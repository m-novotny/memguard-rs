//! Example: Using memguard-rs in a `no_std` embedded context.
//!
//! This example demonstrates how to use the core primitives
//! (zeroization, constant-time comparison) without an allocator.
//!
//! Run with: `cargo build --example no_std_usage --no-default-features`

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use memguard_rs::{ct_eq, Secret, Zeroize};

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn verify_key(stored: &[u8; 32], received: &[u8; 32]) -> bool {
    let mut secret = Secret::new(*stored);
    let mut result = false;
    secret.expose(|s| {
        result = ct_eq(s, received);
    });
    // `secret` is zeroized when it goes out of scope
    result
}
