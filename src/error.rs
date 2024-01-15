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
        assert_eq!(
            format!("{}", Error::LockFailed),
            "failed to lock memory"
        );
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
