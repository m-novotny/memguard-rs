//! Secure memory handling primitives for Rust.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod zeroize;
pub mod mlock;

pub use zeroize::Zeroize;
