# Contributing to memguard-rs

Thank you for your interest in contributing! This crate handles security-
sensitive memory operations, so contributions require extra care.

## Getting started

1. Fork the repository
2. Create a feature branch: `git checkout -b my-feature`
3. Make your changes
4. Run tests: `cargo test --all-features`
5. Run clippy: `cargo clippy --all-targets --all-features -- -D warnings`
6. Run fmt check: `cargo fmt --all -- --check`
7. Commit with a clear message
8. Open a pull request

## Code review

All PRs go through review. For changes touching `unsafe` code, the
review will focus on:

- **Soundness**: Does the `unsafe` block violate any Rust invariants (aliasing, lifetimes, alignment)?
- **Zeroization correctness**: Can the compiler optimize away the volatile writes?
- **Drop safety**: Can `Drop` implementations panic or double-drop?
- **Constant-time guarantees**: Do comparison functions have any branches that depend on secret data?

## Testing

- All new functionality must have tests
- Tests should cover edge cases (empty slices, zero-length, max values)
- If you add a new platform, test on that platform
- Integration tests go in `tests/`, unit tests go inline with `#[cfg(test)] mod tests`

## Commit style

- Use the imperative mood: "Add constant-time select for slices" not "Added"
- Keep the first line under 72 characters
- Reference issues: "Fix mlock failure on macOS (#12)"

## Style

- Follow `rustfmt` defaults
- No `unwrap()` or `expect()` in library code (tests are fine)
- Document all `pub` items with `///` doc comments
- Document all `unsafe` blocks with a `// SAFETY:` comment explaining why the operation is safe

## License

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this work shall be dual-licensed under the
MIT and Apache 2.0 licenses, without any additional terms or conditions.
