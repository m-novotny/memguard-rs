# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✓         |

## Reporting a vulnerability

If you discover a security vulnerability in memguard-rs, please report it
responsibly.

**Do not open a public GitHub issue.**

Instead, email **m.novotny@isaagi.cloud** with:

1. A description of the vulnerability
2. Steps to reproduce (proof of concept, if possible)
3. The potential impact
4. Any suggested mitigations or fixes

You should receive a response within 48 hours. If the vulnerability is
confirmed, a fix will be developed and a security advisory will be
published via GitHub Security Advisories.

## Scope

This crate handles sensitive memory operations. The following are
considered security-relevant:

- **Zeroization bypass** — if the compiler can optimize away volatile writes under any circumstances
- **Memory locking failure** — if `mlock`/`VirtualLock` silently fails without returning an error
- **Constant-time violations** — if comparison functions leak timing information through branches or early returns
- **Drop ordering bugs** — if `Secret<T>` or `GuardedRegion<N>` can fail to zeroize or unlock on drop
- **Soundness holes** — if `unsafe` code violates Rust's aliasing or lifetime rules

## Disclosure timeline

1. **Day 0**: Vulnerability reported via email
2. **Day 1–2**: Acknowledgment and triage
3. **Day 3–7**: Fix developed, tested, and validated
4. **Day 7–14**: Coordinated disclosure via GitHub Security Advisory
5. **Day 14+**: Patch release published

## Security considerations for users

### What this crate protects against

- Secrets remaining in memory after they are no longer needed
- Secrets being written to swap space (with `lock` feature)
- Timing side-channels during secret comparison

### What this crate does NOT protect against

- **Core dumps** — locked memory may still appear in core dumps. Disable core dumps in production (`ulimit -c 0` on Unix)
- **Hibernation** — locked memory may be written to hibernation files on some platforms
- **Speculative execution attacks** — Spectre/Meltdown may bypass memory safety guarantees
- **Physical access** — an attacker with physical access to RAM can read any memory
- **Cold boot attacks** — RAM contents may persist after power loss, even for locked pages
- **Debuggers** — `ptrace` and similar debugging tools can read locked memory

Use defense-in-depth. This crate is one layer, not a complete solution.
