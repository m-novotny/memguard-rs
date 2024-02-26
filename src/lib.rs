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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod zeroize;
pub mod mlock;
pub mod secret;
pub mod cmp;
pub mod guard;
pub mod error;

pub use zeroize::Zeroize;
pub use secret::Secret;
pub use cmp::{ct_eq, ct_select};
pub use guard::GuardedRegion;
pub use error::{Error, Result};
