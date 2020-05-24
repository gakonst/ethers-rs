//! ethers-rs
//!
//! ethers-rs is a port of [ethers-js](github.com/ethers-io/ethers.js) in Rust.
//!
//! # Quickstart
//!
//! ## Sending Ether
//!
//! ## Checking the state of the blockchain
//!
//! ## Deploying and interacting with a smart contract
//!
//! ## Watching on-chain events
//!
//! More examples can be found in the [`examples` directory of the
//! repositry](https://github.com/gakonst/ethers-rs)

pub mod providers;
pub use providers::HttpProvider;

pub mod signers;

pub use signers::{AnyWallet, MainnetWallet, Signer};

/// Ethereum related datatypes
pub mod types;

/// Re-export solc for convenience
pub use solc;

mod utils;
