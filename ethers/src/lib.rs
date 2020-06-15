#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![deny(intra_doc_link_resolution_failure)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # ethers-rs
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
//! ```
//! # #[allow(unused)]
//! use ethers::prelude::*;
//! ```
//!
//! Examples on how you can use the types imported by the prelude can be found in
//! the [`examples` directory of the repository](https://github.com/gakonst/ethers-rs)
//! and in the `tests/` directories of each crate.
//!
//! # Quick explanation of each module in ascending order of abstraction
//!
//! ## `core`
//!
//! Contains all the [necessary data structures](core/types/index.html) for interacting
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
//! [transactions](core/types/struct.TransactionRequest.html) where the `data` field contains
//! the [function's
//! selector](https://ethereum.stackexchange.com/questions/72363/what-is-a-function-selector)
//! along with the arguments of the called function. This module provides the
//! [`Contract`] and [`ContractFactory`] abstractions so that you do not have to worry about that.
//! It also provides typesafe bindings via the [`abigen`] macro and the [`Abigen` builder].
//!
//! [`Provider`]: providers/struct.Provider.html
//!
//! [`Wallet`]: signers/struct.Wallet.html
//! [`Client`]: signers/struct.Client.html
//!
//! [`ContractFactory`]: contract/struct.ContractFactory.html
//! [`Contract`]: contract/struct.Contract.html
//! [`abigen`]: contract/macro.abigen.html
//! [`Abigen` builder]: contract/struct.Abigen.html
//!
//! [`utils`]: core/utils/index.html
//! [`abi`]: core/abi/index.html
//! [`types`]: core/types/index.html

#[cfg(feature = "contract")]
#[cfg_attr(docsrs, doc(cfg(feature = "contract")))]
// This is copied over from the crate-level docs, is there a better way to do this?
/// Type-safe abstractions for interacting with Ethereum smart contracts
///
/// Interacting with a smart contract requires broadcasting carefully crafted
/// [transactions](core/types/struct.TransactionRequest.html) where the `data` field contains
/// the [function's
/// selector](https://ethereum.stackexchange.com/questions/72363/what-is-a-function-selector)
/// along with the arguments of the called function. This module provides the
/// [`Contract`] and [`ContractFactory`] abstractions so that you do not have to worry about that.
/// It also provides typesafe bindings via the [`abigen`] macro and the [`Abigen` builder].
///
/// [`ContractFactory`]: struct.ContractFactory.html
/// [`Contract`]: struct.Contract.html
/// [`abigen`]: macro.abigen.html
/// [`Abigen` builder]: struct.Abigen.html
pub mod contract {
    pub use ethers_contract::*;
}

#[cfg(feature = "providers")]
#[cfg_attr(docsrs, doc(cfg(feature = "providers")))]
/// # Clients for interacting with Ethereum nodes
///
/// This crate provides asynchronous [Ethereum JSON-RPC](https://github.com/ethereum/wiki/wiki/JSON-RPC)
/// compliant clients.
///
/// For more documentation on the available calls, refer to the [`Provider`](struct.Provider.html)
/// struct.
///
/// # Examples
///
/// ```no_run
/// use ethers::providers::{Provider, Http};
/// use std::convert::TryFrom;
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Provider::<Http>::try_from(
///     "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
/// )?;
///
/// let block = provider.get_block(100u64).await?;
/// println!("Got block: {}", serde_json::to_string(&block)?);
///
/// let code = provider.get_code("0x89d24a6b4ccb1b6faa2625fe562bdd9a23260359", None).await?;
/// println!("Got code: {}", serde_json::to_string(&code)?);
/// # Ok(())
/// # }
/// ```
///
/// # Ethereum Name Service
///
/// The provider may also be used to resolve [Ethereum Name Service](https://ens.domains) (ENS) names
/// to addresses (and vice versa). The default ENS address is [mainnet](https://etherscan.io/address/0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e) and can be overriden by calling the [`ens`](struct.Provider.html#method.ens) method on the provider.
///
/// ```no_run
/// # use ethers::providers::{Provider, Http};
/// # use std::convert::TryFrom;
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// # let provider = Provider::<Http>::try_from(
/// #     "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
/// # )?;
/// // Resolve ENS name to Address
/// let name = "vitalik.eth";
/// let address = provider.resolve_name(name).await?;
///
/// // Lookup ENS name given Address
/// let resolved_name = provider.lookup_address(address).await?;
/// assert_eq!(name, resolved_name);
/// # Ok(())
/// # }
/// ```
pub mod providers {
    pub use ethers_providers::*;
}

#[cfg(feature = "signers")]
#[cfg_attr(docsrs, doc(cfg(feature = "signers")))]
/// Provides a unified interface for locally signing transactions and interacting
/// with the Ethereum JSON-RPC. You can implement the `Signer` trait to extend
/// functionality to other signers such as Hardware Security Modules, KMS etc.
///
/// ```no_run
/// # use ethers::{
///     providers::{Http, Provider},
///     signers::Wallet,
///     core::types::TransactionRequest
/// };
/// # use std::convert::TryFrom;
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// // connect to the network
/// let provider = Provider::<Http>::try_from("http://localhost:8545")?;
///
/// // instantiate the wallet and connect it to the provider to get a client
/// let client = "dcf2cbdd171a21c480aa7f53d77f31bb102282b3ff099c78e3118b37348c72f7"
///     .parse::<Wallet>()?
///     .connect(provider);
///
/// // create a transaction
/// let tx = TransactionRequest::new()
///     .to("vitalik.eth") // this will use ENS
///     .value(10000);
///
/// // send it! (this will resolve the ENS name to an address under the hood)
/// let pending_tx = client.send_transaction(tx, None).await?;
///
/// // get the receipt
/// let receipt = pending_tx.await?;
///
/// // get the mined tx
/// let tx = client.get_transaction(receipt.transaction_hash).await?;
///
/// println!("{}", serde_json::to_string(&tx)?);
/// println!("{}", serde_json::to_string(&receipt)?);
///
/// # Ok(())
/// # }
pub mod signers {
    pub use ethers_signers::*;
}

#[cfg(feature = "core")]
#[cfg_attr(docsrs, doc(cfg(feature = "core")))]
/// Ethereum types, cryptography and utilities.
/// _It is recommended to use the `utils`, `types` and `abi` re-exports instead of
/// the `core` module to simplify your imports._
///
/// This library provides type definitions for Ethereum's main datatypes along
/// with other utilities for interacting with the Ethereum ecosystem
///
/// ## Signing an ethereum-prefixed message
///
/// Signing in Ethereum is done by first prefixing the message with
/// `"\x19Ethereum Signed Message:\n" + message.length`, and then
/// signing the hash of the result.
///
/// ```rust
/// use ethers::core::types::{PrivateKey, Address};
///
/// let message = "Some data";
/// let key = PrivateKey::new(&mut rand::thread_rng());
/// let address = Address::from(&key);
///
/// // Sign the message
/// let signature = key.sign(message);
///
/// // Recover the signer from the message
/// let recovered = signature.recover(message).unwrap();
///
/// assert_eq!(recovered, address);
/// ```
///
/// ## Utilities
///
/// The crate provides utilities for launching local Ethereum testnets by using `ganache-cli`
/// via the `GanacheBuilder` struct. In addition, you're able to compile contracts on the
/// filesystem by providing a glob to their path, using the `Solc` struct.
///
/// # ABI Encoding and Decoding
///
/// This crate re-exports the [`ethabi`](http://docs.rs/ethabi) crate's functions
/// under the `abi` module, as well as the [`secp256k1`](https://docs.rs/libsecp256k1)
/// and [`rand`](https://docs.rs/rand) crates for convenience.
pub mod core {
    pub use ethers_core::*;
}

// Re-export ethers_core::utils/types/abi
#[cfg(feature = "core")]
pub use ethers_core::abi;
#[cfg(feature = "core")]
pub use ethers_core::types;
#[cfg(feature = "core")]
pub use ethers_core::utils;

/// Easy imports of frequently used type definitions and traits
pub mod prelude {
    #[cfg(feature = "contract")]
    pub use ethers_contract::*;

    #[cfg(feature = "providers")]
    pub use ethers_providers::*;

    #[cfg(feature = "signers")]
    pub use ethers_signers::*;

    #[cfg(feature = "core")]
    pub use ethers_core::types::*;
}
