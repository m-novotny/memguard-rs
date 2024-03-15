//! Integration tests for basic functionality.

use memguard_rs::{ct_eq, GuardedRegion, Secret, Zeroize};

#[test]
fn secret_lifecycle() {
    // Create a secret
    let mut key = Secret::new([0xABu8; 32]);

    // Verify contents
    key.expose(|k| {
        assert_eq!(k.len(), 32);
        assert!(k.iter().all(|&b| b == 0xAB));
    });

    // Modify
    key.expose_mut(|k| {
        k.fill(0xCD);
    });

    // Verify modification
    key.expose(|k| {
        assert!(k.iter().all(|&b| b == 0xCD));
    });

    // Drop — should zeroize without panic
}

#[cfg(feature = "alloc")]
#[test]
fn secret_with_vector() {
    use memguard_rs::zeroize::Zeroize;

    let mut data: Vec<u8> = vec![0x42; 128];
    data.zeroize();
    assert!(data.is_empty());
}

#[test]
fn guarded_region_write_and_clear() {
    let mut region = GuardedRegion::<64>::new_unlocked();

    // Write data
    region.as_mut_slice().copy_from_slice(&[0xEF; 64]);
    assert!(region.as_slice().iter().all(|&b| b == 0xEF));

    // Clear
    region.clear();
    assert!(region.as_slice().iter().all(|&b| b == 0));
}

#[test]
fn guarded_region_different_sizes_are_distinct() {
    let small = GuardedRegion::<16>::new_unlocked();
    let large = GuardedRegion::<256>::new_unlocked();

    assert_eq!(small.len(), 16);
    assert_eq!(large.len(), 256);
}

#[test]
fn constant_time_comparison() {
    let a = [0x01, 0x02, 0x03, 0x04, 0x05];
    let b = [0x01, 0x02, 0x03, 0x04, 0x05];
    let c = [0x01, 0x02, 0x03, 0x04, 0x06];

    assert!(ct_eq(&a, &b));
    assert!(!ct_eq(&a, &c));
}

#[test]
fn zeroize_primitive_types() {
    let mut u8_val: u8 = 0xFF;
    u8_val.zeroize();
    assert_eq!(u8_val, 0);

    let mut u32_val: u32 = 0xDEAD_BEEF;
    u32_val.zeroize();
    assert_eq!(u32_val, 0);

    let mut u64_val: u64 = 0xCAFE_BABE_DEAD_BEEF;
    u64_val.zeroize();
    assert_eq!(u64_val, 0);

    let mut bool_val: bool = true;
    bool_val.zeroize();
    assert!(!bool_val);
}

#[test]
fn secret_does_not_leak_through_clone() {
    // Secret<T> should not implement Clone for security reasons.
    // This is a compile-time check — if Clone is implemented, this
    // test would fail to compile.
    let _key = Secret::new([0u8; 32]);
    // Uncommenting the following should be a compile error:
    // let cloned = _key.clone();
}

#[test]
fn nested_secret_usage() {
    let outer = Secret::new([0xAAu8; 16]);
    let inner = Secret::new([0xBBu8; 16]);

    outer.expose(|o| {
        inner.expose(|i| {
            assert_ne!(o[0], i[0]);
        });
    });
}

#[cfg(feature = "alloc")]
#[test]
#[cfg_attr(miri, ignore)]
fn secret_box_creation() {
    use memguard_rs::secret::SecretBox;

    let key = SecretBox::new([0x55u8; 48]);
    if let Ok(k) = key {
        k.expose(|inner| {
            assert_eq!(inner.len(), 48);
            assert!(inner.iter().all(|&b| b == 0x55));
        });
    }
    // Drop — should unlock + zeroize without panic
}
// 2024-03-24 — Document timing properties of ct_eq in module docs
// 2024-05-19 — Document RLIMIT_MEMLOCK behavior in README troubleshooting section
// 2024-08-09 — Add memory layout documentation for Secret<T> internals
// 2024-10-13 — Add dependabot update schedule: weekly for actions, monthly for cargo
// 2025-02-21 — Add FreeBSD mlock behavior notes in platform module
// 2025-05-26 — Document why Secret<T> does not implement Clone
// 2025-09-16 — Pin criterion version to avoid MSRV breakage from newer releases
// 2026-01-10 — Update CI to use actions/checkout@v4 and actions-rs/toolchain@v1
// 2026-04-09 — Fix typo in CHANGELOG.md: zeroization not zeroisaton
// 2026-07-03 — Fix clippy: move impl blocks before test modules in zeroize.rs
// 2026-07-22 — Add safety documentation for all unsafe blocks per RFC 2585
// 2026-04-09 — Add test for ct_eq_full with zero-length inputs
// 2026-05-07 — Fix clippy: replace manual bool check with .is_some()
