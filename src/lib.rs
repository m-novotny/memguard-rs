//! # memguard-rs
//!
//! Secure memory handling primitives for Rust.
//!
//! ## Features
//!
//! - **Zeroization on drop** — volatile-write memory clearing the compiler cannot optimize away
//! - **Memory locking** — `mlock`/`VirtualLock` to prevent secrets from being written to swap
//! - **Constant-time comparison** — timing side-channel resistant equality checks for secrets
//! - **Compile-time guarded regions** — const-generic memory regions with type-level size enforcement
//! - **`no_std` compatible** — core primitives work without an allocator
//! - **Zero dependencies** — no transitive dependency surface to audit
//!
//! ## Quick start
//!
//! ```rust
//! use memguard_rs::Secret;
//!
//! let mut key = Secret::new([0u8; 32]);
//!
//! // Access the secret only within a closure — it's not exposed outside
//! key.expose(|k| {
//!     assert_eq!(k.len(), 32);
//! });
//!
//! // Modify in place
//! key.expose_mut(|k| {
//!     k[0] = 0xFF;
//! });
//!
//! // When `key` goes out of scope, its memory is zeroized via volatile writes
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod cmp;
pub mod error;
pub mod guard;
pub mod mlock;
pub mod secret;
pub mod zeroize;

pub use cmp::{ct_eq, ct_select};
pub use error::{Error, Result};
pub use guard::GuardedRegion;
pub use secret::Secret;
pub use zeroize::Zeroize;
