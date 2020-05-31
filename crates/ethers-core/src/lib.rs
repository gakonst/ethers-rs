//! # Ethereum Related DataTypes
//!
//! This library provides type definitions for Ethereum's main datatypes
//!
//! # Signing an ethereum-prefixed message
//!
//! Signing in Ethereum is done by first prefixing the message with
//! `"\x19Ethereum Signed Message:\n" + message.length`, and then
//! signing the hash of the result.
//!
//! ```rust
//! use ethers_core::types::{PrivateKey, Address};
//!
//! let message = "Some data";
//! let key = PrivateKey::new(&mut rand::thread_rng());
//! let address = Address::from(key);
//!
//! // Sign the message
//! let signature = key.sign(message);
//!
//! // Recover the signer from the message
//! let recovered = signature.recover(message).unwrap();
//!
//! assert_eq!(recovered, address);
//! ```
//!
//! # Utilities
//!
//! The crate provides utilities for launching local Ethereum testnets by using `ganache-cli`
//! via the `GanacheBuilder` struct. In addition, you're able to compile contracts on the
//! filesystem by providing a glob to their path, using the `Solc` struct.
//!
//! # ABI Encoding and Decoding
//!
//! This crate re-exports the [`ethabi`](http://docs.rs/ethabi) crate's functions
//! under the `abi` module
//!
//! # A note about `secp256k1` and `rand`
//!
//! The version of `rand` used in the `secp256k1` crate is not compatible with the
//! latest one in crates at the time of writing (rand version 0.5.1, secp256k1 version 0.17.1)
//! As a result, the RNGs used for generating private keys must use a compatible rand crate
//! version. For convenience, we re-export it so that consumers can use it as `ethers_core::rand`.

/// Ethereum related datatypes
pub mod types;

#[cfg(feature = "abi")]
pub mod abi;

/// Various utilities
pub mod utils;

// re-export the non-standard rand version so that other crates don't use the
// wrong one by accident
pub use rand;

// re-export libsecp
pub use secp256k1;
