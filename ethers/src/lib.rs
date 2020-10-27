#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

//! # Complete Ethereum & Celo library and wallet implementation.
//!
//! > ethers-rs is a port of [ethers-js](github.com/ethers-io/ethers.js) in Rust.
//!
//! _Note: All examples using `await` are assuming that they are called from inside an `async`
//! function which is run in an async runtime. You are free to use any runtime and executor of
//! your preference._
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
//! the [`examples` directory of the repository](https://github.com/gakonst/ethers-rs/tree/master/ethers/examples)
//! and in the `tests/` directories of each crate.
//!
//! # Quick explanation of each module in ascending order of abstraction
//!
//! ## `core`
//!
//! Contains all the [necessary data structures](core::types) for interacting
//! with Ethereum, along with cryptographic utilities for signing and verifying
//! ECDSA signatures on `secp256k1`. Bindings to the solidity compiler and `ganache-cli`
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
//! For security reasons, you typically do not want your private keys to be stored on the nodes.
//! This module provides a [`Wallet`] type for loading a private key which can be connected with a
//! [`Provider`] to produce a [`Client`]. The [`Client`] type is the object via which we recommend
//! users with local private keys to use when interacting with Ethereum.
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
//! [`Provider`]: providers::Provider
//!
//! [`Wallet`]: signers::Wallet
//! [`Client`]: signers::Client
//!
//! [`ContractFactory`]: contract::ContractFactory
//! [`Contract`]: contract::Contract
//! [`abigen`]: ./contract/macro.abigen.html
//! [`Abigen` builder]: contract::Abigen
//!
//! [`utils`]: core::utils
//! [`abi`]: core::abi
//! [`types`]: core::types

#[cfg(feature = "contract")]
pub use ethers_contract as contract;

#[cfg(feature = "providers")]
pub use ethers_providers as providers;

#[cfg(feature = "signers")]
pub use ethers_signers as signers;

#[cfg(feature = "core")]
pub use ethers_core as core;

#[cfg(feature = "middleware")]
pub use ethers_middleware as middleware;

// Re-export ethers_core::utils/types/abi
// We hide these docs so that the rustdoc links send the visitor
// to the corresponding crate, instead of the re-export
#[doc(hidden)]
#[cfg(feature = "core")]
pub use ethers_core::abi;
#[doc(hidden)]
#[cfg(feature = "core")]
pub use ethers_core::types;
#[doc(hidden)]
#[cfg(feature = "core")]
pub use ethers_core::utils;

/// Easy imports of frequently used type definitions and traits
#[doc(hidden)]
pub mod prelude {
    #[cfg(feature = "contract")]
    pub use ethers_contract::*;

    #[cfg(feature = "providers")]
    pub use ethers_providers::*;

    #[cfg(feature = "signers")]
    pub use ethers_signers::*;

    #[cfg(feature = "middleware")]
    pub use ethers_middleware::*;

    #[cfg(feature = "core")]
    pub use ethers_core::types::*;
}
