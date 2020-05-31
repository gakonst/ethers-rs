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

#[cfg(feature = "contract")]
pub mod contract {
    pub use ethers_contract::*;
}

#[cfg(feature = "providers")]
pub mod providers {
    pub use ethers_providers::*;
}

#[cfg(feature = "signers")]
pub mod signers {
    pub use ethers_signers::*;
}

#[cfg(feature = "types")]
pub mod types {
    pub use ethers_types::*;
}
