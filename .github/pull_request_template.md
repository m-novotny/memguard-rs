## Pull request checklist

- [ ] I have read [CONTRIBUTING.md](../CONTRIBUTING.md)
- [ ] My code follows the style guidelines (`cargo fmt --all -- --check` passes)
- [ ] I have added tests for my changes
- [ ] All tests pass (`cargo test --all-features` passes)
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] I have documented any new `pub` items with `///` doc comments
- [ ] If my PR touches `unsafe` code, I have added a `// SAFETY:` comment explaining the safety invariants
- [ ] My commits follow the imperative mood convention

## Description

Describe your changes here.

## Motivation

Why is this change needed? Reference any relevant issues with `Fixes #N` or `Closes #N`.

## Type of change

- [ ] Bug fix (non-breaking)
- [ ] New feature (non-breaking)
- [ ] Breaking change
- [ ] Documentation only
- [ ] Test improvement

## Security impact

If this PR touches security-sensitive code (zeroization, mlock, constant-time comparison, unsafe blocks), describe the security implications. If not applicable, write "N/A".
