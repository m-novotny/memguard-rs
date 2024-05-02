# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-01-15

### Added

- `Zeroize` trait with implementations for all primitive integer types, `bool`, `char`, `f32`, `f64`, byte slices, arrays, `Vec<u8>`, `String`, and `Box<T>`
- `zeroize_slice` and `zeroize_ptr` functions using volatile writes with compiler fences
- `mlock`/`munlock` FFI bindings for Unix (via `extern "C"`)
- `VirtualLock`/`VirtualUnlock` FFI bindings for Windows (via `extern "system"`)
- No-op fallback for platforms where memory locking is unavailable
- `Secret<T>` wrapper type with closure-based exposure and zeroization on drop
- `SecretBox<T>` heap-allocated locked secret (requires `alloc` feature)
- `GuardedRegion<const N: usize>` const-generic locked memory region
- `ct_eq` — constant-time byte slice comparison
- `ct_eq_full` — fully constant-time comparison including length
- `ct_select` / `ct_select_slice` — constant-time conditional select
- `Error` enum with `LockFailed`, `UnlockFailed`, `AllocationFailed` variants
- `no_std` support — core primitives work without an allocator
- Feature flags: `std` (default), `alloc`, `lock` (default)
- Zero external dependencies
- CI workflow testing on Ubuntu, macOS, and Windows with stable, beta, and nightly Rust
- `SECURITY.md` policy
- `CONTRIBUTING.md` guidelines
- Dual MIT/Apache-2.0 license
