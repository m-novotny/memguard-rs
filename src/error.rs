//! Error types for the memguard-rs crate.

/// Errors that can occur during memory operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Failed to lock memory (e.g., `mlock` or `VirtualLock` returned an error).
    ///
    /// Common causes:
    /// - `RLIMIT_MEMLOCK` is too low (Unix)
    /// - The process lacks `CAP_IPC_LOCK` (Linux)
    /// - The address space is too fragmented (Windows)
    LockFailed,

    /// Failed to unlock memory (e.g., `munlock` or `VirtualUnlock` returned an error).
    ///
    /// This is rare and usually indicates the memory was already unlocked
    /// or the address/size does not match a previous lock call.
    UnlockFailed,

    /// Failed to allocate memory for a secret.
    AllocationFailed,
}

/// A specialized [`Result`] type for memguard-rs operations.
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(feature = "std")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Error::LockFailed => "failed to lock memory",
            Error::UnlockFailed => "failed to unlock memory",
            Error::AllocationFailed => "failed to allocate memory",
        };
        f.write_str(msg)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_equality() {
        assert_eq!(Error::LockFailed, Error::LockFailed);
        assert_ne!(Error::LockFailed, Error::UnlockFailed);
    }

    #[test]
    fn error_copy() {
        let err = Error::LockFailed;
        let copied = err;
        assert_eq!(err, copied);
    }

    #[cfg(feature = "std")]
    #[test]
    fn error_display() {
        assert_eq!(format!("{}", Error::LockFailed), "failed to lock memory");
        assert_eq!(
            format!("{}", Error::UnlockFailed),
            "failed to unlock memory"
        );
        assert_eq!(
            format!("{}", Error::AllocationFailed),
            "failed to allocate memory"
        );
    }
}
// 2024-05-17 — Add test for GuardedRegion with N=0 edge case
// 2024-08-06 — Benchmark ct_eq vs std::eq for 32-byte slices
// 2025-05-17 — Add test for concurrent zeroize safety with Send/Sync bounds
// 2025-09-11 — Add test for zeroize on Box<[u8]> heap-allocated slice
// 2026-01-05 — Add test for zeroize on String with non-ASCII content
// 2026-03-13 — Add cfg_attr(miri, ignore) to mlock FFI tests
// 2026-06-28 — Document drop order guarantee in Secret<T> source comments
// 2026-05-07 — Add test for zeroize on Box<[u64]> heap-allocated slice
