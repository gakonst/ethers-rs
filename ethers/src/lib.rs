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
#[cfg_attr(docsrs, doc(cfg(feature = "contract")))]
/// TODO
pub mod contract {
    pub use ethers_contract::*;
}

#[cfg(feature = "providers")]
#[cfg_attr(docsrs, doc(cfg(feature = "providers")))]
/// # Clients for interacting with Ethereum nodes
///
/// This crate provides asynchronous [Ethereum JSON-RPC](https://github.com/ethereum/wiki/wiki/JSON-RPC)
/// compliant clients. The client is network-specific in order to provide ENS support and EIP-155
/// replay protection. If you are testing and do not want to use EIP-155, you may use the `Any`
/// network type and override the provider's ENS address with the `ens` method.
///
/// ```rust
/// use ethers::providers::{HttpProvider, networks::Any};
/// use std::convert::TryFrom;
/// use tokio::runtime::Runtime;
///
/// let provider = HttpProvider::<Any>::try_from(
///     "https://mainnet.infura.io/v3/9408f47dedf04716a03ef994182cf150"
/// ).unwrap();
///
/// // Since this is an async function, we need to run it from an async runtime,
/// // such as `tokio`
/// let mut runtime = Runtime::new().expect("Failed to create Tokio runtime");
/// let block = runtime.block_on(provider.get_block(100u64)).unwrap();
/// println!("Got block: {}", serde_json::to_string(&block).unwrap());
/// ```
///
/// # Ethereum Name Service
///
/// The provider may also be used to resolve [Ethereum Name Service](https://ens.domains) (ENS) names
/// to addresses (and vice versa). The address of the deployed ENS contract per network is specified in
/// the `networks` module. If you want to use mainnet ENS, you should instantiate your provider as
/// follows:
///
/// ```rust
/// # use ethers::providers::{HttpProvider, networks::Mainnet};
/// # use std::convert::TryFrom;
/// # use tokio::runtime::Runtime;
/// # let provider = HttpProvider::<Mainnet>::try_from(
/// #     "https://mainnet.infura.io/v3/9408f47dedf04716a03ef994182cf150"
/// # ).unwrap();
/// # let mut runtime = Runtime::new().expect("Failed to create Tokio runtime");
/// // Resolve ENS name to Address
/// let name = "vitalik.eth";
/// let address = runtime.block_on(provider.resolve_name(name)).unwrap();
/// let address = address.unwrap();
///
/// // Lookup ENS name given Address
/// let resolved_name = runtime.block_on(provider.lookup_address(address)).unwrap();
/// let resolved_name = resolved_name.unwrap();
/// assert_eq!(name, resolved_name);
/// ```
pub mod providers {
    pub use ethers_providers::*;
}

#[cfg(feature = "signers")]
#[cfg_attr(docsrs, doc(cfg(feature = "signers")))]
/// # ethers-signers
///
/// Provides a unified interface for locally signing transactions and interacting
/// with the Ethereum JSON-RPC. You can implement the `Signer` trait to extend
/// functionality to other signers such as Hardware Security Modules, KMS etc.
///
/// ```ignore
/// # use anyhow::Result;
/// # use ethers::{providers::HttpProvider, signers::MainnetWallet, types::TransactionRequest};
/// # use std::convert::TryFrom;
/// # async fn main() -> Result<()> {
/// // connect to the network
/// let provider = HttpProvider::try_from("http://localhost:8545")?;
///
/// // instantiate the wallet and connect it to the provider to get a client
/// let client = "dcf2cbdd171a21c480aa7f53d77f31bb102282b3ff099c78e3118b37348c72f7"
///     .parse::<MainnetWallet>()?
///     .connect(&provider);
///
/// // create a transaction
/// let tx = TransactionRequest::new()
///     .to("vitalik.eth") // this will use ENS
///     .value(10000);
///
/// // send it! (this will resolve the ENS name to an address under the hood)
/// let hash = client.send_transaction(tx, None).await?;
///
/// // get the mined tx
/// let tx = client.get_transaction(hash).await?;
///
/// // get the receipt
/// let receipt = client.get_transaction_receipt(tx.hash).await?;
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
/// # Ethereum types, cryptography and utilities
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
/// under the `abi` module
///
/// # A note about `secp256k1` and `rand`
///
/// The version of `rand` used in the `secp256k1` crate is not compatible with the
/// latest one in crates at the time of writing (rand version 0.5.1, secp256k1 version 0.17.1)
/// As a result, the RNGs used for generating private keys must use a compatible rand crate
/// version. For convenience, we re-export it so that consumers can use it as `ethers_core::rand`.
pub mod core {
    pub use ethers_core::*;
}

// Re-export ethers_core::utils
#[cfg(feature = "core")]
pub use ethers_core::utils;

/// Easy import of frequently used type definitions and traits
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
