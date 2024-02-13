# Development Log

- [2024-01-08 10:15:00] Initial scaffold: Zeroize trait with volatile writes
- [2024-01-12 19:22:00] Add unit tests for zeroize_slice with various sizes
- [2024-01-15 09:30:00] Add mlock/VirtualLock memory locking via direct FFI
- [2024-01-22 18:45:00] Add Error enum and Result type alias
- [2024-01-27 15:39:00] Write safety justification comment for write_volatile usage
- [2024-02-03 14:20:00] Add Secret<T> wrapper with closure-based exposure and zeroize on drop
- [2024-02-13 19:23:00] Add test for zeroize on f64 and f32 primitives
