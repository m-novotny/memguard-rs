//! Compile-time enforced memory safety boundaries using const generics.
//!
//! The [`GuardedRegion<N>`] type represents a fixed-size memory region
//! that is:
//!
//! 1. **Locked with `mlock`** on creation (when the `lock` feature is enabled)
//! 2. **Zeroized on drop** via volatile writes
//! 3. **Size-checked at compile time** using const generics
//!
//! The const generic parameter `N` enforces the region size at compile
//! time, preventing accidental mixing of regions with different sizes.
//!
//! # Example
//!
//! ```rust
//! use memguard_rs::GuardedRegion;
//!
//! // Create a 32-byte zeroized-on-drop region
//! let mut region = GuardedRegion::<32>::new_unlocked();
//!
//! // Write sensitive data
//! region.as_mut_slice().copy_from_slice(&[0xAB; 32]);
//!
//! // Read it back
//! assert_eq!(region.as_slice()[0], 0xAB);
//!
//! // When `region` drops, memory is zeroized
//! ```

use crate::error::Result;
use crate::zeroize::{zeroize_slice, Zeroize};

/// A fixed-size memory region that is locked and zeroized on drop.
///
/// The region is backed by a stack-allocated array of size `N`. On
/// creation, `mlock` is called to prevent the region from being swapped
/// (when the `lock` feature is enabled). On drop, the region is
/// zeroized using volatile writes and then unlocked.
///
/// # Const generic safety
///
/// The size `N` is part of the type, so two `GuardedRegion<N>` values
/// with different `N` are different types and cannot be confused at
/// compile time.
pub struct GuardedRegion<const N: usize> {
    data: [u8; N],
    locked: bool,
}

impl<const N: usize> GuardedRegion<N> {
    /// Create a new locked, zero-initialized memory region.
    ///
    /// The region is initialized to all zeros and then locked with
    /// `mlock`/`VirtualLock` to prevent swapping.
    ///
    /// # Errors
    ///
    /// Returns an error if the OS refuses to lock the
    /// memory (e.g., due to `RLIMIT_MEMLOCK` being too low).
    #[inline]
    pub fn new() -> Result<Self> {
        let mut region = Self {
            data: [0u8; N],
            locked: false,
        };

        if N > 0 {
            let ptr = region.data.as_ptr();
            // SAFETY: `ptr` points to valid, properly aligned memory
            // of exactly `N` bytes. The region is owned by `self` and
            // will remain valid until drop.
            unsafe {
                crate::mlock::lock(ptr, N)?;
            }
            region.locked = true;
        }

        Ok(region)
    }

    /// Create a new region without locking memory.
    ///
    /// The region is still zeroized on drop, but its memory is not
    /// locked with `mlock`. Useful for environments where `mlock` is
    /// unavailable or when locking is handled at a higher level.
    #[inline]
    pub fn new_unlocked() -> Self {
        Self {
            data: [0u8; N],
            locked: false,
        }
    }

    /// Create a region pre-filled with data, then locked.
    ///
    /// This minimizes the exposure window compared to creating an empty
    /// region and copying data into it afterward.
    ///
    /// # Errors
    ///
    /// Returns an error if the OS refuses to lock the memory.
    #[inline]
    pub fn from_bytes(data: &[u8; N]) -> Result<Self> {
        let mut region = Self::new()?;
        region.as_mut_slice().copy_from_slice(data);
        Ok(region)
    }

    /// Create a region pre-filled with data, without locking.
    #[inline]
    pub fn from_bytes_unlocked(data: &[u8; N]) -> Self {
        let mut region = Self::new_unlocked();
        region.as_mut_slice().copy_from_slice(data);
        region
    }

    /// Access the region as an immutable byte slice.
    #[inline]
    pub fn as_slice(&self) -> &[u8; N] {
        &self.data
    }

    /// Access the region as a mutable byte slice.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8; N] {
        &mut self.data
    }

    /// Get the size of the region in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        N
    }

    /// Check if the region is empty (size 0).
    #[inline]
    pub fn is_empty(&self) -> bool {
        N == 0
    }

    /// Check whether the region's memory is locked.
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Manually zeroize the region's contents.
    ///
    /// This is called automatically on drop, but can be called earlier
    /// to clear the region while it is still in use.
    #[inline]
    pub fn clear(&mut self) {
        self.zeroize();
    }
}

impl<const N: usize> Zeroize for GuardedRegion<N> {
    #[inline]
    fn zeroize(&mut self) {
        zeroize_slice(&mut self.data);
    }
}

impl<const N: usize> Default for GuardedRegion<N> {
    fn default() -> Self {
        Self::new_unlocked()
    }
}

impl<const N: usize> Drop for GuardedRegion<N> {
    fn drop(&mut self) {
        // Zeroize first.
        self.zeroize();

        // Then unlock if we locked.
        if self.locked {
            let ptr = self.data.as_ptr();
            // SAFETY: same pointer and size passed to `lock` in `new`.
            let _ = unsafe { crate::mlock::unlock(ptr, N) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_new_unlocked() {
        let region = GuardedRegion::<16>::new_unlocked();
        assert!(!region.is_locked());
        assert_eq!(region.len(), 16);
        assert!(!region.is_empty());
        assert!(region.as_slice().iter().all(|&b| b == 0));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn region_new_locked() {
        // Lock may fail on systems with low mlock limits.
        if let Ok(region) = GuardedRegion::<4096>::new() {
            assert!(region.is_locked());
        }
    }

    #[test]
    fn region_write_read() {
        let mut region = GuardedRegion::<8>::new_unlocked();
        region.as_mut_slice().copy_from_slice(&[0xAB; 8]);
        assert!(region.as_slice().iter().all(|&b| b == 0xAB));
    }

    #[test]
    fn region_clear() {
        let mut region = GuardedRegion::<4>::new_unlocked();
        region.as_mut_slice().copy_from_slice(&[0xFF; 4]);
        region.clear();
        assert!(region.as_slice().iter().all(|&b| b == 0));
    }

    #[test]
    fn region_drops_clean() {
        {
            let mut region = GuardedRegion::<32>::new_unlocked();
            region.as_mut_slice().copy_from_slice(&[0x42; 32]);
        }
        // Reaching here means drop didn't panic.
    }

    #[test]
    fn region_zero_size() {
        let region = GuardedRegion::<0>::new_unlocked();
        assert!(region.is_empty());
        assert_eq!(region.len(), 0);
    }

    #[test]
    fn region_default() {
        let region = GuardedRegion::<64>::default();
        assert!(!region.is_locked());
        assert_eq!(region.len(), 64);
    }

    #[test]
    fn region_from_bytes() {
        let data = [0xABu8; 16];
        let region = GuardedRegion::<16>::from_bytes_unlocked(&data);
        assert_eq!(region.as_slice(), &data);
    }

    #[test]
    fn region_from_bytes_large() {
        let data = [0xCDu8; 256];
        let region = GuardedRegion::<256>::from_bytes_unlocked(&data);
        assert_eq!(region.as_slice(), &data);
    }
}
// 2024-03-04 — Refactor zeroize macro to reduce code duplication
// 2024-04-22 — Add cross-compilation targets for ARM and aarch64 in CI
// 2024-07-18 — Add no_std compatibility test in CI
// 2024-09-24 — Document unsafe invariants in mlock platform modules
// 2025-01-16 — Add test for zeroize on large 4096-byte array
// 2025-04-21 — Add categories field to Cargo.toml for crates.io discoverability
// 2025-08-01 — Add test for ct_eq_full with very large slices (8192 bytes)
// 2025-12-03 — Add year-end review notes in CHANGELOG.md unreleased section
// 2026-03-06 — Add test for GuardedRegion::new() failure on locked-down system
// 2026-05-02 — Document compiler fence necessity in zeroize safety comments
