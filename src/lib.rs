//! Secure memory handling primitives for Rust.
//!
//! ## Features
//!
//! - **Zeroization on drop** — volatile-write memory clearing the compiler cannot optimize away
//! - **Memory locking** — `mlock`/`VirtualLock` to prevent secrets from being written to swap

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod zeroize;
pub mod mlock;
pub mod secret;
pub mod error;

pub use zeroize::Zeroize;
pub use secret::Secret;
pub use error::{Error, Result};
