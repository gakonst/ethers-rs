//! ethers-rs
//!
//! ethers-rs is a port of [ethers-js](github.com/ethers-io/ethers.js) in Rust.

pub mod providers;

pub mod wallet;

/// Ethereum related datatypes
pub mod types;

/// Re-export solc for convenience
pub use solc;

/// JSON-RPC client
mod jsonrpc;

mod utils;
