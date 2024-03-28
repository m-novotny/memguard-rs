//! Integration tests for constant-time comparison functions.

use memguard_rs::cmp::{ct_eq, ct_eq_full, ct_select, ct_select_slice};
use memguard_rs::ct_eq as crate_ct_eq;

#[test]
fn ct_eq_identical_slices() {
    for len in [0, 1, 4, 16, 32, 64, 128, 256, 1024] {
        let a = vec![0x42u8; len];
        let b = vec![0x42u8; len];
        assert!(ct_eq(&a, &b), "failed for len={}", len);
    }
}

#[test]
fn ct_eq_single_bit_difference() {
    let a = [0u8; 32];
    let mut b = [0u8; 32];

    for bit_pos in 0..256 {
        let byte = bit_pos / 8;
        let bit = bit_pos % 8;
        b[byte] = 1 << bit;

        assert!(!ct_eq(&a, &b), "failed at bit {}", bit_pos);

        b[byte] = 0;
    }
}

#[test]
fn ct_eq_reexport_matches_module() {
    let a = [1, 2, 3, 4];
    let b = [1, 2, 3, 4];
    assert_eq!(crate_ct_eq(&a, &b), ct_eq(&a, &b));
}

#[test]
fn ct_eq_full_equal_lengths() {
    assert!(ct_eq_full(&[1, 2, 3], &[1, 2, 3]));
    assert!(ct_eq_full(&[], &[]));
    assert!(ct_eq_full(&[0xFF], &[0xFF]));
}

#[test]
fn ct_eq_full_unequal_lengths() {
    assert!(!ct_eq_full(&[1, 2], &[1, 2, 3]));
    assert!(!ct_eq_full(&[1, 2, 3], &[1, 2]));
    assert!(!ct_eq_full(&[], &[1]));
    assert!(!ct_eq_full(&[1], &[]));
}

#[test]
fn ct_eq_full_unequal_content_same_length() {
    assert!(!ct_eq_full(&[1, 2, 3], &[1, 2, 4]));
}

#[test]
fn ct_select_all_bit_patterns() {
    for a in [0x00, 0x55, 0xAA, 0xFF] {
        for b in [0x00, 0x55, 0xAA, 0xFF] {
            assert_eq!(ct_select(a, b, 0), a, "select({}, {}, 0)", a, b);
            assert_eq!(ct_select(a, b, 1), b, "select({}, {}, 1)", a, b);
        }
    }
}

#[test]
fn ct_select_ignores_high_bits() {
    // Only the LSB matters
    assert_eq!(ct_select(0xAA, 0xBB, 0), 0xAA);
    assert_eq!(ct_select(0xAA, 0xBB, 2), 0xAA); // even → select a
    assert_eq!(ct_select(0xAA, 0xBB, 3), 0xBB); // odd → select b
    assert_eq!(ct_select(0xAA, 0xBB, 255), 0xBB); // odd → select b
}

#[test]
fn ct_select_slice_roundtrip() {
    let a = [0x11u8; 32];
    let b = [0x22u8; 32];
    let mut out = [0u8; 32];

    ct_select_slice(&mut out, &a, &b, 0);
    assert_eq!(out, a);

    ct_select_slice(&mut out, &a, &b, 1);
    assert_eq!(out, b);
}

#[test]
fn ct_eq_large_input() {
    let a = vec![0x42u8; 100_000];
    let b = vec![0x42u8; 100_000];
    assert!(ct_eq(&a, &b));

    let mut c = b.clone();
    c[99_999] = 0x43;
    assert!(!ct_eq(&a, &c));
}
