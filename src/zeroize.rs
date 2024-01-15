//! Secure zeroization of memory.
//!
//! Provides the [`Zeroize`] trait and implementations for common types.
//! Zeroization uses [`core::ptr::write_volatile`] to ensure the compiler
//! cannot optimize away the clearing of sensitive memory, combined with
//! [`core::sync::atomic::compiler_fence`] to prevent reordering.
//!
//! # Why not just `mem.set(0)`?
//!
//! The compiler is free to elide writes it considers "dead" — if a buffer
//! is about to go out of scope, the compiler may determine the zeroing has
//! no observable effect and remove it entirely. Volatile writes bypass
//! this optimization, guaranteeing the memory is actually overwritten.

use core::ptr::write_volatile;
use core::sync::atomic::{compiler_fence, Ordering};

/// A trait for types that can be securely zeroized.
///
/// Implementing this trait ensures that the type's memory can be safely
/// cleared in a way that the compiler cannot optimize away. This is
/// critical for types containing secrets such as cryptographic keys,
/// passwords, tokens, or any sensitive data that should not persist in
/// memory after it is no longer needed.
///
/// # Implementation notes
///
/// Implementations should use volatile writes to clear memory and should
/// not rely on `ptr::write_bytes` without volatile semantics, as the
/// compiler may optimize those away.
pub trait Zeroize {
    /// Zeroize self, overwriting memory with zeros using volatile writes.
    fn zeroize(&mut self);
}

/// Zeroize a byte slice using volatile writes with compiler fences.
///
/// This is the core zeroization primitive used by all implementations
/// in this crate. The fences ensure that the volatile writes are not
/// reordered before or after the zeroization region.
#[inline]
pub fn zeroize_slice(slice: &mut [u8]) {
    compiler_fence(Ordering::SeqCst);
    for byte in slice.iter_mut() {
        // SAFETY: `byte` is a valid, aligned, mutable reference to a u8
        // within the slice. The write_volatile ensures the compiler
        // cannot elide this write.
        unsafe { write_volatile(byte as *mut u8, 0) };
    }
    compiler_fence(Ordering::SeqCst);
}

/// Zeroize a region of memory given a raw pointer and length.
///
/// # Safety
///
/// The caller must ensure:
/// - `ptr` points to a valid, writable memory region of at least `len` bytes
/// - The memory is not concurrently accessed during this call
/// - `ptr` is properly aligned for `u8`
#[inline]
pub unsafe fn zeroize_ptr(ptr: *mut u8, len: usize) {
    compiler_fence(Ordering::SeqCst);
    for i in 0..len {
        write_volatile(ptr.add(i), 0);
    }
    compiler_fence(Ordering::SeqCst);
}

// === Primitive integer implementations ===

macro_rules! impl_zeroize_int {
    ($($t:ty),*) => {
        $(
            impl Zeroize for $t {
                #[inline]
                fn zeroize(&mut self) {
                    compiler_fence(Ordering::SeqCst);
                    // SAFETY: `self` is a valid, aligned, mutable reference.
                    unsafe { write_volatile(self as *mut $t, 0) };
                    compiler_fence(Ordering::SeqCst);
                }
            }
        )*
    };
}

impl_zeroize_int!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, bool, char, f32, f64);

// === Slice and array implementations ===

impl Zeroize for [u8] {
    #[inline]
    fn zeroize(&mut self) {
        zeroize_slice(self);
    }
}

impl<const N: usize> Zeroize for [u8; N] {
    #[inline]
    fn zeroize(&mut self) {
        zeroize_slice(self);
    }
}

// === Alloc-requiring implementations ===

#[cfg(feature = "alloc")]
impl Zeroize for alloc::vec::Vec<u8> {
    #[inline]
    fn zeroize(&mut self) {
        zeroize_slice(self.as_mut_slice());
        self.clear();
    }
}

#[cfg(feature = "alloc")]
impl Zeroize for alloc::string::String {
    #[inline]
    fn zeroize(&mut self) {
        // SAFETY: We have exclusive access to the string's buffer.
        // The bytes are valid for writing (String stores valid UTF-8,
        // but we're overwriting with zeros which is valid UTF-8).
        unsafe {
            let ptr = self.as_mut_vec().as_mut_ptr();
            let len = self.len();
            zeroize_ptr(ptr, len);
        }
        self.clear();
    }
}

#[cfg(feature = "alloc")]
impl<T: Zeroize> Zeroize for alloc::boxed::Box<T> {
    #[inline]
    fn zeroize(&mut self) {
        // SAFETY: dereferencing the Box gives a valid mutable reference
        // to the contained value.
        (**self).zeroize();
    }
}

#[cfg(feature = "alloc")]
impl<T: Zeroize> Zeroize for alloc::boxed::Box<[T]> {
    #[inline]
    fn zeroize(&mut self) {
        for item in self.iter_mut() {
            item.zeroize();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeroize_u8() {
        let mut val: u8 = 0xFF;
        val.zeroize();
        assert_eq!(val, 0);
    }

    #[test]
    fn zeroize_u32() {
        let mut val: u32 = 0xDEAD_BEEF;
        val.zeroize();
        assert_eq!(val, 0);
    }

    #[test]
    fn zeroize_array() {
        let mut arr = [0x41u8; 16];
        arr.zeroize();
        assert!(arr.iter().all(|&b| b == 0));
    }

    #[test]
    fn zeroize_slice() {
        let mut buf = [0x42u8; 32];
        zeroize_slice(&mut buf);
        assert!(buf.iter().all(|&b| b == 0));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn zeroize_vec() {
        let mut buf: Vec<u8> = vec![0x43; 64];
        buf.zeroize();
        assert!(buf.is_empty());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn zeroize_string() {
        let mut s = String::from("sensitive password data");
        s.zeroize();
        assert!(s.is_empty());
    }

    #[test]
    fn zeroize_ptr() {
        let mut buf = [0x44u8; 8];
        unsafe { zeroize_ptr(buf.as_mut_ptr(), 8) };
        assert!(buf.iter().all(|&b| b == 0));
    }
}
