#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(intra_doc_link_resolution_failure)]
//! # Clients for interacting with Ethereum nodes
//!
//! This crate provides asynchronous [Ethereum JSON-RPC](https://github.com/ethereum/wiki/wiki/JSON-RPC)
//! compliant clients.
//!
//! For more documentation on the available calls, refer to the [`Provider`](crate::Provider)
//! struct.
//!
//! # Examples
//!
//! ```no_run
//! use ethers::providers::{Provider, Http};
//! use std::convert::TryFrom;
//!
//! # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = Provider::<Http>::try_from(
//!     "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
//! )?;
//!
//! let block = provider.get_block(100u64).await?;
//! println!("Got block: {}", serde_json::to_string(&block)?);
//!
//! let code = provider.get_code("0x89d24a6b4ccb1b6faa2625fe562bdd9a23260359", None).await?;
//! println!("Got code: {}", serde_json::to_string(&code)?);
//! # Ok(())
//! # }
//! ```
//!
//! # Websockets
//!
//! The crate has support for WebSockets. If none of the provided async runtime
//! features are enabled, you must manually instantiate the WS connection and wrap
//! it with with a [`Ws::new`](method@crate::Ws::new) call.
//!
//! ```ignore
//! use ethers::providers::Ws;
//!
//! let ws = Ws::new(...);
//! ```
//!
//! If you have compiled the library with any of the following features, you may
//! instantiate the websocket instance with the `connect` call and your URL:
//! - `tokio-runtime`: Uses `tokio` as the runtime
//! - `async-std-runtime`: Uses `async-std-runtime`
//!
//! ```no_run
//! # #[cfg(any(
//! #     feature = "tokio-runtime",
//! #     feature = "tokio-tls",
//! #     feature = "async-std-runtime",
//! #     feature = "async-std-tls",
//! # ))]
//! # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
//! # use ethers::providers::Ws;
//! let ws = Ws::connect("ws://localhost:8545").await?;
//! # Ok(())
//! # }
//! ```
//!
//! TLS support is also provided via the following feature flags:
//! - `tokio-tls`
//! - `async-tls`
//!
//! ```no_run
//! # #[cfg(any(
//! #     feature = "tokio-runtime",
//! #     feature = "tokio-tls",
//! #     feature = "async-std-runtime",
//! #     feature = "async-std-tls",
//! # ))]
//! # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
//! # use ethers::providers::Ws;
//! let ws = Ws::connect("wss://localhost:8545").await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Ethereum Name Service
//!
//! The provider may also be used to resolve [Ethereum Name Service](https://ens.domains) (ENS) names
//! to addresses (and vice versa). The default ENS address is [mainnet](https://etherscan.io/address/0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e) and can be overriden by calling the [`ens`](method@crate::Provider::ens) method on the provider.
//!
//! ```no_run
//! # use ethers::providers::{Provider, Http};
//! # use std::convert::TryFrom;
//! # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
//! # let provider = Provider::<Http>::try_from(
//! #     "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
//! # )?;
//! // Resolve ENS name to Address
//! let name = "vitalik.eth";
//! let address = provider.resolve_name(name).await?;
//!
//! // Lookup ENS name given Address
//! let resolved_name = provider.lookup_address(address).await?;
//! assert_eq!(name, resolved_name);
//! # Ok(())
//! # }
//! ```
mod transports;
pub use transports::{Http, Ws};

mod provider;

// ENS support
mod ens;

mod pending_transaction;
pub use pending_transaction::PendingTransaction;

mod stream;
pub use stream::{FilterStream, DEFAULT_POLL_INTERVAL};
// re-export `StreamExt` so that consumers can call `next()` on the `FilterStream`
// without having to import futures themselves
pub use futures_util::StreamExt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Debug};

pub use provider::{Provider, ProviderError};

#[async_trait]
/// Trait which must be implemented by data transports to be used with the Ethereum
/// JSON-RPC provider.
pub trait JsonRpcClient: Send + Sync {
    /// A JSON-RPC Error
    type Error: Error + Into<ProviderError>;

    /// Sends a request with the provided JSON-RPC and parameters serialized as JSON
    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: for<'a> Deserialize<'a>;
}
