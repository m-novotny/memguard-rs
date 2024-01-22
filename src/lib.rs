//! Secure memory handling primitives for Rust.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod zeroize;
pub mod mlock;
pub mod error;

pub use zeroize::Zeroize;
pub use error::{Error, Result};
