#![cfg_attr(docsrs, feature(doc_cfg))]
//! Ethereum types, cryptography and utilities.
//! _It is recommended to use the `utils`, `types` and `abi` re-exports instead of
//! the `core` module to simplify your imports._
//!
//! This library provides type definitions for Ethereum's main datatypes along
//! with other utilities for interacting with the Ethereum ecosystem
//!
//! ## Signing an ethereum-prefixed message
//!
//! Signing in Ethereum is done by first prefixing the message with
//! `"\x19Ethereum Signed Message:\n" + message.length`, and then
//! signing the hash of the result.
//!
//! ```rust
//! use ethers::core::types::{PrivateKey, Address};
//!
//! let message = "Some data";
//! let key = PrivateKey::new(&mut rand::thread_rng());
//! let address = Address::from(&key);
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
//! ## Utilities
//!
//! The crate provides utilities for launching local Ethereum testnets by using `ganache-cli`
//! via the `GanacheBuilder` struct. In addition, you're able to compile contracts on the
//! filesystem by providing a glob to their path, using the `Solc` struct.
//!
//! # ABI Encoding and Decoding
//!
//! This crate re-exports the [`ethabi`](ethabi) crate's functions
//! under the `abi` module, as well as the [`secp256k1`](https://docs.rs/libsecp256k1)
//! and [`rand`](https://docs.rs/rand) crates for convenience.
pub mod types;

pub mod abi;

/// Various utilities
pub mod utils;

// re-export rand to avoid potential confusion when there's rand version mismatches
pub use rand;

// re-export libsecp
pub use secp256k1;
