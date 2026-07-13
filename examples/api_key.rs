//! Example: Protecting an API key in memory.
//!
//! Run with: `cargo run --example api_key`

use memguard_rs::Secret;

fn main() {
    // Simulate receiving an API key from user input
    let api_key_bytes: [u8; 32] = {
        let mut arr = [0u8; 32];
        // In real code, this would come from stdin, a file, a vault, etc.
        for (i, byte) in arr.iter_mut().enumerate() {
            *byte = (i as u8).wrapping_mul(7).wrapping_add(0x41);
        }
        arr
    };

    // Wrap it in a Secret — zeroized on drop
    let key = Secret::new(api_key_bytes);

    // Use it only within a closure
    key.expose(|k| {
        println!("API key loaded: {} bytes", k.len());
        println!("First 4 bytes: {:02x?}", &k[..4]);
    });

    // Try to lock it (may fail if mlock limit is too low)
    let locked_key = key.lock();
    match &locked_key {
        Ok(k) => println!("Memory locked — won't appear in swap"),
        Err(_) => println!("Warning: could not lock memory (check ulimit -l)"),
    }

    // When `locked_key` goes out of scope, memory is:
    // 1. Zeroized via volatile writes
    // 2. Unlocked (if locking succeeded)
    // 3. Dropped
    println!("Key will be zeroized when it goes out of scope");
}
