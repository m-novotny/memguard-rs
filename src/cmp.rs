//! Constant-time comparison functions for secret data.
//!
//! Timing side-channel attacks exploit the fact that standard comparison
//! functions short-circuit: they return as soon as a difference is found,
//! leaking information about *where* the first difference occurs. This
//! module provides comparison functions that execute in constant time
//! regardless of the input values.
//!
//! # When to use
//!
//! Use [`ct_eq`] whenever comparing secret values such as:
//! - MACs (Message Authentication Codes)
//! - HMACs
//! - Session tokens
//! - API keys
//! - Any value where timing could leak information about the secret

/// Compare two byte slices in constant time.
///
/// This function always processes every byte of both slices, regardless
/// of where differences occur. It returns `true` if the slices are equal,
/// `false` otherwise.
///
/// # Timing behavior
///
/// The comparison loop runs for exactly `min(a.len(), b.len())` iterations
/// in all cases. The length check itself is not constant-time: if the
/// slices differ in length, the function returns `false` immediately.
/// In most real-world use cases, the length of a secret is not itself
/// sensitive (e.g., a 32-byte HMAC is always 32 bytes).
///
/// If you need fully constant-time comparison including length, use
/// [`ct_eq_full`] instead.
///
/// # Example
///
/// ```rust
/// use memguard_rs::ct_eq;
///
/// let a = [0x01, 0x02, 0x03, 0x04];
/// let b = [0x01, 0x02, 0x03, 0x04];
/// let c = [0x01, 0x02, 0x03, 0x05];
///
/// assert!(ct_eq(&a, &b));
/// assert!(!ct_eq(&a, &c));
/// ```
#[inline]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }

    diff == 0
}

/// Compare two byte slices in fully constant time, including length.
///
/// Unlike [`ct_eq`], this function does not short-circuit on length
/// mismatch. It processes `max(a.len(), b.len())` bytes in all cases,
/// padding the shorter slice with zeros internally.
///
/// # When to use
///
/// Use this function when the length of the secret may itself be
/// sensitive information (e.g., variable-length tokens).
///
/// # Example
///
/// ```rust
/// use memguard_rs::cmp::ct_eq_full;
///
/// let a = [0x01, 0x02, 0x03];
/// let b = [0x01, 0x02, 0x03, 0x04];
///
/// // Returns false, but processes all 4 bytes
/// assert!(!ct_eq_full(&a, &b));
/// ```
#[inline]
pub fn ct_eq_full(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());

    // Length difference folded into the accumulator as a 16-bit value
    // to avoid overflow when XORing with large slices.
    let mut diff: u16 = (a.len() ^ b.len()) as u16;

    for i in 0..max_len {
        let av = if i < a.len() { a[i] } else { 0 };
        let bv = if i < b.len() { b[i] } else { 0 };
        diff |= (av ^ bv) as u16;
    }

    diff == 0
}

/// Constant-time conditional select between two bytes.
///
/// Returns `a` if `select` is 0, `b` if `select` is 1. The selection
/// is performed using a bitmask that is either all-zeros or all-ones,
/// ensuring no branch is taken.
///
/// # Safety of the input
///
/// `select` must be 0 or 1. If it is any other value, only the least
/// significant bit is considered (all other bits are masked off).
///
/// # Example
///
/// ```rust
/// use memguard_rs::ct_select;
///
/// assert_eq!(ct_select(0xAA, 0xBB, 0), 0xAA);
/// assert_eq!(ct_select(0xAA, 0xBB, 1), 0xBB);
/// ```
#[inline]
pub fn ct_select(a: u8, b: u8, select: u8) -> u8 {
    // Mask is 0x00 if select == 0, 0xFF if select == 1.
    // Uses two's complement arithmetic: -(select & 1) gives 0 or -1 (0xFF).
    let mask = (0i8.wrapping_sub((select & 1) as i8)) as u8;
    (a & !mask) | (b & mask)
}

/// Constant-time conditional select for byte slices.
///
/// Writes the selected source into `out` without branching.
///
/// # Panics
///
/// Panics if `a`, `b`, and `out` do not have the same length.
#[inline]
pub fn ct_select_slice(out: &mut [u8], a: &[u8], b: &[u8], select: u8) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());

    let mask = (0i8.wrapping_sub((select & 1) as i8)) as u8;

    for i in 0..out.len() {
        out[i] = (a[i] & !mask) | (b[i] & mask);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ct_eq_equal() {
        assert!(ct_eq(&[1, 2, 3, 4], &[1, 2, 3, 4]));
    }

    #[test]
    fn ct_eq_unequal() {
        assert!(!ct_eq(&[1, 2, 3, 4], &[1, 2, 3, 5]));
    }

    #[test]
    fn ct_eq_different_length() {
        assert!(!ct_eq(&[1, 2, 3], &[1, 2, 3, 4]));
        assert!(!ct_eq(&[1, 2, 3, 4], &[1, 2, 3]));
    }

    #[test]
    fn ct_eq_empty() {
        assert!(ct_eq(&[], &[]));
    }

    #[test]
    fn ct_eq_single_diff() {
        // First byte differs — should still be false
        assert!(!ct_eq(&[0xFF, 0, 0, 0], &[0xFE, 0, 0, 0]));
        // Last byte differs
        assert!(!ct_eq(&[0, 0, 0, 0xFF], &[0, 0, 0, 0xFE]));
    }

    #[test]
    fn ct_eq_full_equal() {
        assert!(ct_eq_full(&[1, 2, 3], &[1, 2, 3]));
    }

    #[test]
    fn ct_eq_full_different_length() {
        assert!(!ct_eq_full(&[1, 2, 3], &[1, 2, 3, 4]));
    }

    #[test]
    fn ct_select_chooses_a() {
        assert_eq!(ct_select(0xAA, 0xBB, 0), 0xAA);
    }

    #[test]
    fn ct_select_chooses_b() {
        assert_eq!(ct_select(0xAA, 0xBB, 1), 0xBB);
    }

    #[test]
    fn ct_select_slice_basic() {
        let mut out = [0u8; 4];
        let a = [0x11, 0x22, 0x33, 0x44];
        let b = [0xAA, 0xBB, 0xCC, 0xDD];

        ct_select_slice(&mut out, &a, &b, 0);
        assert_eq!(out, a);

        ct_select_slice(&mut out, &a, &b, 1);
        assert_eq!(out, b);
    }
}
// 2024-07-28 — Refactor error module to use core::fmt instead of alloc::fmt
// 2024-10-07 — Fix audit workflow: use actions-rs/audit-check v2
// 2025-01-23 — Review and update all safety comments for clarity
// 2025-04-22 — Fix clippy: use bool::then instead of if-let in ct_eq_full
// 2025-08-12 — Document memory ordering choices in zeroize module
// 2025-12-13 — Update LICENSE year range to 2024-2025
// 2026-03-07 — Fix clippy: replace manual loop with iterator in zeroize_slice
// 2026-06-09 — Fix clippy needless_borrow in Secret<T> drop implementation
