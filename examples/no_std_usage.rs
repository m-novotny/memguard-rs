//! Example: Using memguard-rs in a `no_std` embedded context.
//!
//! This example demonstrates the API patterns for using core primitives
//! (zeroization, constant-time comparison) without an allocator.
//!
//! In a real `no_std` project, add this to Cargo.toml:
//! ```toml
//! [dependencies]
//! memguard-rs = { version = "0.1", default-features = false, features = ["lock"] }
//! ```
//!
//! And use `#![no_std]` at the crate root.

use memguard_rs::{ct_eq, Secret};

fn verify_key(stored: &[u8; 32], received: &[u8; 32]) -> bool {
    let secret = Secret::new(*stored);
    let mut result = false;
    secret.expose(|s| {
        result = ct_eq(s, received);
    });
    // `secret` is zeroized when it goes out of scope
    result
}

fn main() {
    let stored = [0x42u8; 32];
    let received = [0x42u8; 32];
    let wrong = [0x43u8; 32];

    assert!(verify_key(&stored, &received));
    assert!(!verify_key(&stored, &wrong));

    println!("no_std pattern verified");
}
