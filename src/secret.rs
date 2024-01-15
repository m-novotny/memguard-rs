//! Secret wrapper type that zeroizes on drop and optionally locks memory.
//!
//! The [`Secret<T>`] type wraps a value containing sensitive data and
//! ensures it is securely zeroized when dropped. Access to the inner
//! value is only possible through scoped closures, preventing accidental
//! exposure through copying or logging.
//!
//! # Design
//!
//! `Secret` uses [`core::mem::ManuallyDrop`] internally to ensure that
//! the value is zeroized *before* its destructor runs, and to prevent
//! double-drops if the zeroization step panics.
//!
//! # Example
//!
//! ```rust
//! use memguard_rs::Secret;
//!
//! // Wrap a cryptographic key
//! let key = Secret::new([0u8; 32]);
//!
//! // Read the key only within a closure
//! key.expose(|k| {
//!     println!("Key length: {}", k.len());
//! });
//!
//! // The key's memory is zeroized when it goes out of scope
//! ```

use crate::error::Result;
use crate::zeroize::Zeroize;
use core::mem::ManuallyDrop;

/// A wrapper that zeroizes its contents on drop.
///
/// `Secret<T>` ensures that the wrapped value is securely cleared from
/// memory using volatile writes when it goes out of scope. The inner
/// value is only accessible through the [`expose`](Secret::expose) and
/// [`expose_mut`](Secret::expose_mut) methods, which take a closure to
/// limit the scope in which the secret is accessible.
///
/// `Secret` does not implement `Clone`, `Debug`, `PartialEq`, or `Display`
/// to prevent accidental leakage of the secret through logging or
/// comparison side-channels.
pub struct Secret<T: Zeroize> {
    inner: ManuallyDrop<T>,
    locked: bool,
}

impl<T: Zeroize> Secret<T> {
    /// Create a new `Secret` wrapping the given value.
    ///
    /// The value is not locked with `mlock` — call [`lock`](Secret::lock)
    /// to prevent the memory from being swapped.
    #[inline]
    pub fn new(val: T) -> Self {
        Self {
            inner: ManuallyDrop::new(val),
            locked: false,
        }
    }

    /// Lock the wrapped value's memory with `mlock`/`VirtualLock`.
    ///
    /// This prevents the memory from being written to swap. On platforms
    /// where locking is unavailable or the `lock` feature is disabled,
    /// this is a no-op.
    ///
    /// # Errors
    ///
    /// Returns [`Error::LockFailed`](crate::Error::LockFailed) if the
    /// OS refuses to lock the memory (e.g., due to resource limits).
    #[inline]
    pub fn lock(mut self) -> Result<Self> {
        let size = core::mem::size_of::<T>();
        if size == 0 {
            return Ok(self);
        }
        let ptr = &*self.inner as *const T as *const u8;
        // SAFETY: `ptr` points to valid memory of `size` bytes. The
        // value is owned by `self` and will remain valid until drop.
        unsafe {
            crate::mlock::lock(ptr, size)?;
        }
        self.locked = true;
        Ok(self)
    }

    /// Borrow the secret for read access within a closure.
    ///
    /// The secret value is only accessible inside the closure, limiting
    /// the window in which it is exposed. The closure must not copy the
    /// secret or store references that outlive the closure.
    #[inline]
    pub fn expose<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
        f(&self.inner)
    }

    /// Borrow the secret for write access within a closure.
    ///
    /// Like [`expose`](Secret::expose), but allows mutation of the
    /// inner value.
    #[inline]
    pub fn expose_mut<R, F: FnOnce(&mut T) -> R>(&mut self, f: F) -> R {
        f(&mut self.inner)
    }

    /// Check whether the secret's memory is locked.
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        // First, unlock if we locked the memory.
        if self.locked {
            let size = core::mem::size_of::<T>();
            if size > 0 {
                let ptr = &*self.inner as *const T as *const u8;
                // SAFETY: same pointer and size passed to `lock`.
                let _ = unsafe { crate::mlock::unlock(ptr, size) };
            }
        }

        // Zeroize the inner value's memory via volatile writes.
        // SAFETY: `inner` is still valid — ManuallyDrop prevents the
        // automatic destructor from running, so we have exclusive access.
        unsafe {
            (&mut *self.inner).zeroize();
        }

        // Now drop the (zeroized) inner value.
        // SAFETY: We haven't dropped `inner` yet, and it's been zeroized.
        unsafe {
            ManuallyDrop::drop(&mut self.inner);
        }
    }
}

// === SecretBox: heap-allocated locked secret ===

#[cfg(feature = "alloc")]
mod boxed {
    use super::*;
    use alloc::boxed::Box;

    /// A heap-allocated secret that is locked with `mlock` and zeroized on drop.
    ///
    /// `SecretBox<T>` is the boxed equivalent of [`Secret<T>`]. It is
    /// useful for secrets that are too large for the stack or whose
    /// size is determined at runtime.
    ///
    /// # Example
    ///
    /// ```rust
    /// use memguard_rs::secret::SecretBox;
    ///
    /// let key = SecretBox::new([0u8; 32]).unwrap();
    /// key.expose(|k| {
    ///     assert_eq!(k.len(), 32);
    /// });
    /// ```
    pub struct SecretBox<T: Zeroize> {
        inner: ManuallyDrop<Box<T>>,
        locked: bool,
    }

    impl<T: Zeroize> SecretBox<T> {
        /// Allocate a new boxed secret and lock its memory.
        ///
        /// # Errors
        ///
        /// Returns [`Error::LockFailed`](crate::Error::LockFailed) if
        /// the OS refuses to lock the memory.
        #[inline]
        pub fn new(val: T) -> Result<Self> {
            let bx = Box::new(val);
            let size = core::mem::size_of::<T>();
            let mut secret = Self {
                inner: ManuallyDrop::new(bx),
                locked: false,
            };

            if size > 0 {
                let ptr = &*secret.inner as *const T as *const u8;
                // SAFETY: `ptr` points to valid heap memory of `size`
                // bytes, owned by the Box.
                match unsafe { crate::mlock::lock(ptr, size) } {
                    Ok(()) => secret.locked = true,
                    Err(e) => {
                        // Drop the box without zeroizing first since the
                        // value was just created and may not contain
                        // sensitive data yet.
                        unsafe { ManuallyDrop::drop(&mut secret.inner) };
                        return Err(e);
                    }
                }
            }

            Ok(secret)
        }

        /// Borrow the secret for read access within a closure.
        #[inline]
        pub fn expose<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
            f(&self.inner)
        }

        /// Borrow the secret for write access within a closure.
        #[inline]
        pub fn expose_mut<R, F: FnOnce(&mut T) -> R>(&mut self, f: F) -> R {
            f(&mut self.inner)
        }

        /// Check whether the secret's memory is locked.
        #[inline]
        pub fn is_locked(&self) -> bool {
            self.locked
        }
    }

    impl<T: Zeroize> Drop for SecretBox<T> {
        fn drop(&mut self) {
            if self.locked {
                let size = core::mem::size_of::<T>();
                if size > 0 {
                    let ptr = &*self.inner as *const T as *const u8;
                    let _ = unsafe { crate::mlock::unlock(ptr, size) };
                }
            }

            // Zeroize the contents.
            // SAFETY: inner is still valid.
            unsafe {
                (&mut *self.inner).zeroize();
            }

            // Drop the Box (which frees the heap allocation).
            // SAFETY: not yet dropped.
            unsafe {
                ManuallyDrop::drop(&mut self.inner);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn secretbox_basic() {
            let key = SecretBox::new([0xABu8; 32]).unwrap();
            key.expose(|k| {
                assert!(k.iter().all(|&b| b == 0xAB));
            });
        }
    }
}

#[cfg(feature = "alloc")]
pub use boxed::SecretBox;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_basic() {
        let mut key = Secret::new([0xFFu8; 16]);
        key.expose(|k| {
            assert!(k.iter().all(|&b| b == 0xFF));
        });
        key.expose_mut(|k| {
            k[0] = 0x00;
        });
        key.expose(|k| {
            assert_eq!(k[0], 0x00);
            assert_eq!(k[1], 0xFF);
        });
    }

    #[test]
    fn secret_drops_without_panic() {
        {
            let _key = Secret::new([0x42u8; 32]);
        }
        // If we get here without panicking, the drop worked.
    }

    #[test]
    fn secret_lock_unlock() {
        let key = Secret::new([0u8; 4096]);
        // Lock may fail on systems with low mlock limits — that's ok.
        if let Ok(locked) = key.lock() {
            assert!(locked.is_locked());
            // Drop happens here, should unlock + zeroize without panic.
        }
    }
}
