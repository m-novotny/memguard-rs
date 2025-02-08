//! Memory locking primitives to prevent swapping of sensitive data.
//!
//! On Unix, this uses `mlock`/`munlock` via direct FFI. On Windows, this
//! uses `VirtualLock`/`VirtualUnlock`. When the `lock` feature is disabled
//! or the platform is unsupported, these functions are no-ops.
//!
//! # Limitations
//!
//! Memory locking prevents pages from being written to swap, but does not
//! prevent other forms of memory exposure (e.g., hibernation, core dumps,
//! or ptrace). For comprehensive protection, consider also disabling core
//! dumps and using `madvise` with `MADV_DONTDUMP` on Unix.
//!
//! # Permissions
//!
//! On Linux, `mlock` requires `CAP_IPC_LOCK` or a sufficiently high
//! `ulimit -l` (RLIMIT_MEMLOCK). The default soft limit is often 64KB.
//! Applications that lock large amounts of memory should raise this limit
//! or grant the capability.

use crate::error::Result;

// === Unix: mlock / munlock via direct FFI (no libc dependency) ===

#[cfg(all(unix, feature = "lock"))]
mod platform {
    use crate::error::{Error, Result};
    use core::ffi::{c_int, c_void};

    extern "C" {
        fn mlock(addr: *const c_void, len: usize) -> c_int;
        fn munlock(addr: *const c_void, len: usize) -> c_int;
    }

    /// Lock a memory region, preventing it from being swapped.
    ///
    /// # Safety
    ///
    /// - `addr` must point to a valid memory region of at least `len` bytes
    /// - The region must remain valid for the duration of the lock
    /// - The address should be page-aligned for some implementations
    #[inline]
    pub unsafe fn lock(addr: *const u8, len: usize) -> Result<()> {
        if len == 0 {
            return Ok(());
        }
        let ret = mlock(addr as *const c_void, len);
        if ret != 0 {
            Err(Error::LockFailed)
        } else {
            Ok(())
        }
    }

    /// Unlock a previously locked memory region.
    ///
    /// # Safety
    ///
    /// - `addr` and `len` must match a previous successful `lock` call
    /// - The region must still be valid
    #[inline]
    pub unsafe fn unlock(addr: *const u8, len: usize) -> Result<()> {
        if len == 0 {
            return Ok(());
        }
        let ret = munlock(addr as *const c_void, len);
        if ret != 0 {
            Err(Error::UnlockFailed)
        } else {
            Ok(())
        }
    }
}

// === Windows: VirtualLock / VirtualUnlock via direct FFI ===

#[cfg(all(windows, feature = "lock"))]
mod platform {
    use crate::error::{Error, Result};

    #[link(name = "kernel32")]
    extern "system" {
        fn VirtualLock(addr: *const core::ffi::c_void, len: usize) -> i32;
        fn VirtualUnlock(addr: *const core::ffi::c_void, len: usize) -> i32;
    }

    /// Lock a memory region, preventing it from being swapped.
    ///
    /// # Safety
    ///
    /// - `addr` must point to a valid memory region of at least `len` bytes
    /// - The region must remain valid for the duration of the lock
    #[inline]
    pub unsafe fn lock(addr: *const u8, len: usize) -> Result<()> {
        if len == 0 {
            return Ok(());
        }
        let ret = VirtualLock(addr as *const core::ffi::c_void, len);
        if ret == 0 {
            Err(Error::LockFailed)
        } else {
            Ok(())
        }
    }

    /// Unlock a previously locked memory region.
    ///
    /// # Safety
    ///
    /// - `addr` and `len` must match a previous successful `lock` call
    /// - The region must still be valid
    #[inline]
    pub unsafe fn unlock(addr: *const u8, len: usize) -> Result<()> {
        if len == 0 {
            return Ok(());
        }
        let ret = VirtualUnlock(addr as *const core::ffi::c_void, len);
        if ret == 0 {
            Err(Error::UnlockFailed)
        } else {
            Ok(())
        }
    }
}

// === No-op fallback for unsupported platforms or when lock feature is off ===

#[cfg(not(any(all(unix, feature = "lock"), all(windows, feature = "lock"))))]
mod platform {
    use crate::error::Result;

    #[inline]
    pub unsafe fn lock(_addr: *const u8, _len: usize) -> Result<()> {
        Ok(())
    }

    #[inline]
    pub unsafe fn unlock(_addr: *const u8, _len: usize) -> Result<()> {
        Ok(())
    }
}

/// Lock a memory region, preventing it from being written to swap.
///
/// This is a thin wrapper around the platform-specific implementation.
///
/// # Safety
///
/// - `addr` must point to a valid, writable memory region of at least `len` bytes
/// - The memory is not concurrently accessed during this call
/// - `addr` is properly aligned for `u8`
#[inline]
pub unsafe fn lock(addr: *const u8, len: usize) -> Result<()> {
    platform::lock(addr, len)
}

/// Unlock a previously locked memory region.
///
/// # Safety
///
/// - `addr` and `len` must match a previous successful `lock` call
/// - The memory region must still be valid
#[inline]
pub unsafe fn unlock(addr: *const u8, len: usize) -> Result<()> {
    platform::unlock(addr, len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn lock_unlock_roundtrip() {
        let mut buf = [0u8; 4096];
        let ptr = buf.as_mut_ptr();
        unsafe {
            // This may fail on systems with low mlock limits, so we
            // only assert if the lock succeeded.
            if lock(ptr, 4096).is_ok() {
                assert!(unlock(ptr, 4096).is_ok());
            }
        }
    }

    #[test]
    fn zero_length_is_ok() {
        let buf: [u8; 0] = [];
        unsafe {
            assert!(lock(buf.as_ptr(), 0).is_ok());
            assert!(unlock(buf.as_ptr(), 0).is_ok());
        }
    }
}
// Note: on macOS, mlock requires the page to be writable and
// the RLIMIT_MEMLOCK default is 64KB, same as Linux.
