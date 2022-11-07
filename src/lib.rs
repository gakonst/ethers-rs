#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

//! # Complete Ethereum & Celo library and wallet implementation.
//!
//! > ethers-rs is a port of [ethers-js](https://github.com/ethers-io/ethers.js) in Rust.
//!
//! ## Quickstart: `prelude`
//!
//! A prelude is provided which imports all the important data types and traits for you. Use this
//! when you want to quickly bootstrap a new project.
//!
//! ```no_run
//! # #[allow(unused)]
//! use ethers::prelude::*;
//! ```
//!
//! Examples on how you can use the types imported by the prelude can be found in
//! the [`examples` directory of the repository](https://github.com/gakonst/ethers-rs/tree/master/examples)
//! and in the `tests/` directories of each crate.
//!
//! # Quick explanation of each module in ascending order of abstraction
//!
//! ## `core`
//!
//! Contains all the [necessary data structures](core::types) for interacting
//! with Ethereum, along with cryptographic utilities for signing and verifying
//! ECDSA signatures on `secp256k1`. Bindings to the Solidity compiler, Anvil and `ganache-cli`
//! are also provided as helpers. To simplify your imports, consider using the re-exported
//! modules described in the next subsection.
//!
//! ## `utils`, `types`, `abi`
//!
//! These are re-exports of the [`utils`], [`types`] and [`abi`] modules from the `core` crate
//!
//! ## `providers`
//!
//! Ethereum nodes expose RPC endpoints (by default at `localhost:8545`). You can connect
//! to them by using the [`Provider`]. The provider instance
//! allows you to issue requests to the node which involve querying the state of Ethereum or
//! broadcasting transactions with unlocked accounts on the node.
//!
//! ## `signers`
//!
//! This module provides a [`Signer`] trait which can be used for signing messages
//! or transactions. A [`Wallet`] type is implemented which can be used with a
//! raw private key, or a YubiHSM2. We also provide Ledger support.
//!
//! ## `contract`
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
//! It also provides typesafe bindings via the [`abigen`] macro and the [`Abigen` builder].
//!
//! ## `middleware`
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
//! [`abigen`]: ./contract/macro.abigen.html
//! [`Abigen` builder]: contract::Abigen
//! [`utils`]: core::utils
//! [`abi`]: core::abi
//! [`types`]: core::types

/// Address book consisting of frequently used contracts
pub mod addressbook {
    pub use ethers_addressbook::*;
}

#[doc = include_str!("../assets/CONTRACT_README.md")]
pub mod contract {
    pub use ethers_contract::*;
}

#[doc = include_str!("../assets/CORE_README.md")]
pub mod core {
    pub use ethers_core::*;
}

#[doc = include_str!("../assets/PROVIDERS_README.md")]
pub mod providers {
    pub use ethers_providers::*;
}

#[doc = include_str!("../assets/MIDDLEWARE_README.md")]
pub mod middleware {
    pub use ethers_middleware::*;
}

#[doc = include_str!("../assets/SIGNERS_README.md")]
pub mod signers {
    pub use ethers_signers::*;
}

#[cfg(feature = "ethers-solc")]
#[doc = include_str!("../assets/SOLC_README.md")]
pub mod solc {
    pub use ethers_solc::*;
}

/// Etherscan bindings
pub mod etherscan {
    pub use ethers_etherscan::*;
}

pub use crate::core::{abi, types, utils};

/// Easy imports of frequently used type definitions and traits
#[doc(hidden)]
pub mod prelude {
    pub use super::addressbook::*;

    pub use super::contract::*;

    pub use super::core::{types::*, *};

    pub use super::middleware::*;

    pub use super::providers::*;

    pub use super::signers::*;

    #[cfg(feature = "ethers-solc")]
    pub use super::solc::*;

    pub use super::etherscan::*;
}
