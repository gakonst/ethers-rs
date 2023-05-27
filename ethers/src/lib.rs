//! # ethers-rs
//!
//! A complete Ethereum and Celo Rust library.
//!
//! ## Quickstart: `prelude`
//!
//! A prelude is provided which imports all the important data types and traits for you. Use this
//! when you want to quickly bootstrap a new project.
//!
//! ```rust
//! use ethers::prelude::*;
//! ```
//!
//! Examples on how you can use the types imported by the prelude can be found in the
//! [`examples` directory of the repository](https://github.com/gakonst/ethers-rs/tree/master/examples)
//! and in the `tests/` directories of each crate.
//!
//! ## Modules
//!
//! The following paragraphs are a quick explanation of each module in ascending order of
//! abstraction. More details can be found in [the book](https://gakonst.com/ethers-rs).
//!
//! ### `core`
//!
//! Contains all the [necessary data structures](core::types) for interacting with Ethereum, along
//! with cryptographic utilities for signing and verifying ECDSA signatures on `secp256k1`. Bindings
//! to the Solidity compiler, Anvil and Ganace are also provided as helpers.
//! To simplify your imports, consider using the re-exported modules described in the next
//! subsection.
//!
//! ### `utils`, `types`, `abi`
//!
//! These are re-exports of the [`utils`], [`types`] and [`abi`] modules from the [`core`] crate.
//!
//! ### `providers`
//!
//! Contains the [`Provider`] struct, an abstraction of a connection to the Ethereum network, which
//! alongside the [`Middleware`] trait provides a concise, consistent interface to standard Ethereum
//! node functionality,
//!
//! ### `signers`
//!
//! Provides a [`Signer`] trait which can be used for signing messages or transactions. A [`Wallet`]
//! type is implemented which can be used with a raw private key or a YubiHSM2. Ledger and Trezor
//! support are also provided.
//!
//! ### `contract`
//!
//! Interacting with Ethereum is not restricted to sending or receiving funds. It also involves
//! using smart contracts, which can be thought of as programs with persistent storage.
//!
//! Interacting with a smart contract requires broadcasting carefully crafted
//! [transactions](core::types::TransactionRequest) where the `data` field contains
//! the [function's
//! selector](https://ethereum.stackexchange.com/questions/72363/what-is-a-function-selector)
//! along with the arguments of the called function. This module provides the
//! [`Contract`] and [`ContractFactory`] abstractions so that you do not have to worry about that.
//!
//! It also provides typesafe bindings via the [`abigen`] macro and the [`Abigen`] builder.
//!
//! ### `middleware`
//!
//! In order to keep the ethers architecture as modular as possible, providers define a
//! [`Middleware`] trait which defines all the methods to interact with an Ethereum node. By
//! implementing the middleware trait, you are able to override the default behavior of methods and
//! do things such as using other gas oracles, escalating your transactions' gas prices, or signing
//! your transactions with a [`Signer`]. The middleware architecture allows users to either use one
//! of the existing middleware, or they are free to write on of their own.
//!
//! [`Provider`]: providers::Provider
//! [`Middleware`]: providers::Middleware
//! [`Wallet`]: signers::Wallet
//! [`Signer`]: signers::Signer
//! [`ContractFactory`]: contract::ContractFactory
//! [`Contract`]: contract::Contract
//! [`abigen`]: contract::abigen
//! [`Abigen`]: contract::Abigen
//! [`utils`]: core::utils
//! [`abi`]: core::abi
//! [`types`]: core::types

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(test(no_crate_inject, attr(deny(rust_2018_idioms), allow(dead_code, unused_variables))))]

#[doc(inline)]
pub use ethers_addressbook as addressbook;
#[doc(inline)]
pub use ethers_contract as contract;
#[doc(inline)]
pub use ethers_core as core;
#[doc(inline)]
pub use ethers_etherscan as etherscan;
#[doc(inline)]
pub use ethers_middleware as middleware;
#[doc(inline)]
pub use ethers_providers as providers;
#[doc(inline)]
pub use ethers_signers as signers;
#[doc(inline)]
#[cfg(feature = "ethers-solc")]
pub use ethers_solc as solc;

#[doc(inline)]
pub use ethers_core::{abi, types, utils};

/// Easy imports of frequently used type definitions and traits.
#[doc(hidden)]
#[allow(unknown_lints, ambiguous_glob_reexports)]
pub mod prelude {
    pub use super::addressbook::contract;

    pub use super::contract::*;

    pub use super::core::{types::*, *};

    pub use super::etherscan::*;

    pub use super::middleware::*;

    pub use super::providers::*;

    pub use super::signers::*;

    #[cfg(feature = "ethers-solc")]
    pub use super::solc::*;
}

// For macro expansions only, not public API.
#[doc(hidden)]
#[allow(unused_extern_crates)]
extern crate self as ethers;
