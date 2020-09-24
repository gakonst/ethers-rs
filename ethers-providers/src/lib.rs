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
pub use transports::*;

mod provider;

// ENS support
mod ens;

mod pending_transaction;
pub use pending_transaction::PendingTransaction;

mod stream;
pub use futures_util::StreamExt;
pub use stream::{FilterWatcher, DEFAULT_POLL_INTERVAL};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{Debug, Display},
    future::Future,
    pin::Pin,
};

pub use provider::{FilterKind, Provider, ProviderError};

// Helper type alias
pub(crate) type PinBoxFut<'a, T> = Pin<Box<dyn Future<Output = Result<T, ProviderError>> + 'a>>;

#[async_trait]
/// Trait which must be implemented by data transports to be used with the Ethereum
/// JSON-RPC provider.
pub trait JsonRpcClient: Debug + Send + Sync {
    /// A JSON-RPC Error
    type Error: Error + Into<ProviderError>;

    /// Sends a request with the provided JSON-RPC and parameters serialized as JSON
    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: for<'a> Deserialize<'a>;
}

use ethers_core::types::*;

#[async_trait(?Send)]
pub trait Middleware: Sync + Send + Debug {
    type Error: Display + Debug;
    type Provider: JsonRpcClient;

    async fn get_block_number(&self) -> Result<U64, Self::Error>;

    async fn send_transaction(
        &self,
        tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<TxHash, Self::Error>;

    async fn resolve_name(&self, ens_name: &str) -> Result<Address, Self::Error>;

    async fn lookup_address(&self, address: Address) -> Result<String, Self::Error>;

    async fn get_block<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<TxHash>>, Self::Error>;

    async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<Transaction>>, Self::Error>;

    async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockNumber>,
    ) -> Result<U256, Self::Error>;

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<U256, Self::Error>;

    async fn call(
        &self,
        tx: &TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<Bytes, Self::Error>;

    async fn get_chainid(&self) -> Result<U256, Self::Error>;

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockNumber>,
    ) -> Result<U256, Self::Error>;

    async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<Transaction>, Self::Error>;

    async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<TransactionReceipt>, Self::Error>;

    async fn get_gas_price(&self) -> Result<U256, Self::Error>;

    async fn get_accounts(&self) -> Result<Vec<Address>, Self::Error>;

    async fn send_raw_transaction(&self, tx: &Transaction) -> Result<TxHash, Self::Error>;

    async fn sign<T: Into<Bytes> + Send + Sync>(
        &self,
        data: T,
        from: &Address,
    ) -> Result<Signature, Self::Error>;

    ////// Contract state

    async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, Self::Error>;

    async fn new_filter(&self, filter: FilterKind<'_>) -> Result<U256, Self::Error>;

    async fn uninstall_filter<T: Into<U256> + Send + Sync>(
        &self,
        id: T,
    ) -> Result<bool, Self::Error>;

    async fn watch<'a>(&'a self, filter: &Filter)
        -> Result<FilterWatcher<'a, Self::Provider, Log>, Self::Error>;

    async fn watch_pending_transactions(&self) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error>;

    async fn get_filter_changes<T, R>(&self, id: T) -> Result<Vec<R>, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        R: for<'a> Deserialize<'a> + Send + Sync;

    async fn watch_blocks(&self) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error>;

    async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<BlockNumber>,
    ) -> Result<Bytes, Self::Error>;

    async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: H256,
        block: Option<BlockNumber>,
    ) -> Result<H256, Self::Error>;
}
