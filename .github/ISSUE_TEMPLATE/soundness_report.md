---
name: Soundness report
about: Report a soundness issue in unsafe code
title: "[SOUNDNESS] "
labels: soundness
assignees: m-novotny
---

## Soundness concern

Describe the soundness issue. A soundness issue is any case where `unsafe` code violates Rust's safety guarantees (aliasing, lifetimes, alignment, uninitialized memory).

## Location

Which file and line(s) contain the concern?

## Why this is unsound

Explain why the `unsafe` block is incorrect. Reference the Rustonomicon or Rust reference if applicable.

## Suggested fix

How should this be fixed?

## Verification

Has this been verified with `miri`? If so, include the output.
