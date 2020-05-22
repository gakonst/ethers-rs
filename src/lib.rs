//! ethers-rs
//!
//! ethers-rs is a port of [ethers-js](github.com/ethers-io/ethers.js) in Rust.

mod network;

pub mod providers;

pub mod wallet;

pub mod primitives;

mod jsonrpc;

/// Re-export solc
pub use solc;
