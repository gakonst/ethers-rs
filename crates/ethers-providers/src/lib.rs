//! # Clients for interacting with Ethereum nodes
//!
//! This crate provides asynchronous [Ethereum JSON-RPC](https://github.com/ethereum/wiki/wiki/JSON-RPC)
//! compliant clients. The client is network-specific in order to provide ENS support and EIP-155
//! replay protection. If you are testing and do not want to use EIP-155, you may use the `Any`
//! network type and override the provider's ENS address with the `ens` method.
//!
//! ```rust
//! use ethers_providers::{HttpProvider, networks::Any};
//! use std::convert::TryFrom;
//! use tokio::runtime::Runtime;
//!
//! let provider = HttpProvider::<Any>::try_from(
//!     "https://mainnet.infura.io/v3/9408f47dedf04716a03ef994182cf150"
//! ).unwrap();
//!
//! // Since this is an async function, we need to run it from an async runtime,
//! // such as `tokio`
//! let mut runtime = Runtime::new().expect("Failed to create Tokio runtime");
//! let block = runtime.block_on(provider.get_block(100u64)).unwrap();
//! println!("Got block: {}", serde_json::to_string(&block).unwrap());
//! ```
//!
//! # Ethereum Name Service
//!
//! The provider may also be used to resolve [Ethereum Name Service](https://ens.domains) (ENS) names
//! to addresses (and vice versa). The address of the deployed ENS contract per network is specified in
//! the `networks` module. If you want to use mainnet ENS, you should instantiate your provider as
//! follows:
//!
//! ```rust
//! # use ethers_providers::{HttpProvider, networks::Mainnet};
//! # use std::convert::TryFrom;
//! # use tokio::runtime::Runtime;
//! # let provider = HttpProvider::<Mainnet>::try_from(
//! #     "https://mainnet.infura.io/v3/9408f47dedf04716a03ef994182cf150"
//! # ).unwrap();
//! # let mut runtime = Runtime::new().expect("Failed to create Tokio runtime");
//! // Resolve ENS name to Address
//! let name = "vitalik.eth";
//! let address = runtime.block_on(provider.resolve_name(name)).unwrap();
//! let address = address.unwrap();
//!
//! // Lookup ENS name given Address
//! let resolved_name = runtime.block_on(provider.lookup_address(address)).unwrap();
//! let resolved_name = resolved_name.unwrap();
//! assert_eq!(name, resolved_name);
//! ```

mod http;
mod provider;

pub mod networks;

// ENS support
mod ens;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Debug};

pub use provider::Provider;

/// An HTTP provider for interacting with an Ethereum-compatible blockchain
pub type HttpProvider<N> = Provider<http::Provider, N>;

#[async_trait]
/// Trait which must be implemented by data transports to be used with the Ethereum
/// JSON-RPC provider.
pub trait JsonRpcClient: Debug {
    /// A JSON-RPC Error
    type Error: Error;

    /// Sends a request with the provided JSON-RPC and parameters serialized as JSON
    async fn request<T: Serialize + Send + Sync, R: for<'a> Deserialize<'a>>(
        &self,
        method: &str,
        params: Option<T>,
    ) -> Result<R, Self::Error>;
}
