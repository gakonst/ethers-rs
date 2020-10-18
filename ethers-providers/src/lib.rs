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
//! use ethers::providers::{Provider, Http, Middleware};
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
//! # use ethers::providers::{Provider, Http, Middleware};
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
pub use stream::{interval, FilterWatcher, DEFAULT_POLL_INTERVAL};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Debug, future::Future, pin::Pin};

pub use provider::{FilterKind, Provider, ProviderError};

// Helper type alias
pub(crate) type PinBoxFut<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProviderError>> + Send + 'a>>;

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
pub trait FromErr<T> {
    fn from(src: T) -> Self;
}

#[async_trait]
pub trait Middleware: Sync + Send + Debug {
    type Error: Send + Error + FromErr<<Self::Inner as Middleware>::Error>;
    type Provider: JsonRpcClient;
    type Inner: Middleware<Provider = Self::Provider>;

    fn inner(&self) -> &Self::Inner;

    async fn get_block_number(&self) -> Result<U64, Self::Error> {
        self.inner().get_block_number().await.map_err(FromErr::from)
    }

    async fn send_transaction(
        &self,
        tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<TxHash, Self::Error> {
        self.inner()
            .send_transaction(tx, block)
            .await
            .map_err(FromErr::from)
    }

    async fn resolve_name(&self, ens_name: &str) -> Result<Address, Self::Error> {
        self.inner()
            .resolve_name(ens_name)
            .await
            .map_err(FromErr::from)
    }

    async fn lookup_address(&self, address: Address) -> Result<String, Self::Error> {
        self.inner()
            .lookup_address(address)
            .await
            .map_err(FromErr::from)
    }

    async fn get_block<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<TxHash>>, Self::Error> {
        self.inner()
            .get_block(block_hash_or_number)
            .await
            .map_err(FromErr::from)
    }

    async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<Transaction>>, Self::Error> {
        self.inner()
            .get_block_with_txs(block_hash_or_number)
            .await
            .map_err(FromErr::from)
    }

    async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockNumber>,
    ) -> Result<U256, Self::Error> {
        self.inner()
            .get_transaction_count(from, block)
            .await
            .map_err(FromErr::from)
    }

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<U256, Self::Error> {
        self.inner().estimate_gas(tx).await.map_err(FromErr::from)
    }

    async fn call(
        &self,
        tx: &TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<Bytes, Self::Error> {
        self.inner().call(tx, block).await.map_err(FromErr::from)
    }

    async fn get_chainid(&self) -> Result<U256, Self::Error> {
        self.inner().get_chainid().await.map_err(FromErr::from)
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockNumber>,
    ) -> Result<U256, Self::Error> {
        self.inner()
            .get_balance(from, block)
            .await
            .map_err(FromErr::from)
    }

    async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<Transaction>, Self::Error> {
        self.inner()
            .get_transaction(transaction_hash)
            .await
            .map_err(FromErr::from)
    }

    async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        self.inner()
            .get_transaction_receipt(transaction_hash)
            .await
            .map_err(FromErr::from)
    }

    async fn get_gas_price(&self) -> Result<U256, Self::Error> {
        self.inner().get_gas_price().await.map_err(FromErr::from)
    }

    async fn get_accounts(&self) -> Result<Vec<Address>, Self::Error> {
        self.inner().get_accounts().await.map_err(FromErr::from)
    }

    async fn send_raw_transaction(&self, tx: &Transaction) -> Result<TxHash, Self::Error> {
        self.inner()
            .send_raw_transaction(tx)
            .await
            .map_err(FromErr::from)
    }

    async fn sign<T: Into<Bytes> + Send + Sync>(
        &self,
        data: T,
        from: &Address,
    ) -> Result<Signature, Self::Error> {
        self.inner().sign(data, from).await.map_err(FromErr::from)
    }

    ////// Contract state

    async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, Self::Error> {
        self.inner().get_logs(filter).await.map_err(FromErr::from)
    }

    async fn new_filter(&self, filter: FilterKind<'_>) -> Result<U256, Self::Error> {
        self.inner().new_filter(filter).await.map_err(FromErr::from)
    }

    async fn uninstall_filter<T: Into<U256> + Send + Sync>(
        &self,
        id: T,
    ) -> Result<bool, Self::Error> {
        self.inner()
            .uninstall_filter(id)
            .await
            .map_err(FromErr::from)
    }

    async fn watch<'a>(
        &'a self,
        filter: &Filter,
    ) -> Result<FilterWatcher<'a, Self::Provider, Log>, Self::Error> {
        self.inner().watch(filter).await.map_err(FromErr::from)
    }

    async fn watch_pending_transactions(
        &self,
    ) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
        self.inner()
            .watch_pending_transactions()
            .await
            .map_err(FromErr::from)
    }

    async fn get_filter_changes<T, R>(&self, id: T) -> Result<Vec<R>, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        R: for<'a> Deserialize<'a> + Send + Sync,
    {
        self.inner()
            .get_filter_changes(id)
            .await
            .map_err(FromErr::from)
    }

    async fn watch_blocks(&self) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
        self.inner().watch_blocks().await.map_err(FromErr::from)
    }

    async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<BlockNumber>,
    ) -> Result<Bytes, Self::Error> {
        self.inner()
            .get_code(at, block)
            .await
            .map_err(FromErr::from)
    }

    async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: H256,
        block: Option<BlockNumber>,
    ) -> Result<H256, Self::Error> {
        self.inner()
            .get_storage_at(from, location, block)
            .await
            .map_err(FromErr::from)
    }

    fn pending_transaction(&self, tx_hash: TxHash) -> PendingTransaction<'_, Self::Provider> {
        self.inner().pending_transaction(tx_hash)
    }

    async fn txpool_content(&self) -> Result<TxpoolContent, Self::Error> {
        self.inner().txpool_content().await.map_err(FromErr::from)
    }

    async fn txpool_inspect(&self) -> Result<TxpoolInspect, Self::Error> {
        self.inner().txpool_inspect().await.map_err(FromErr::from)
    }

    async fn txpool_status(&self) -> Result<TxpoolStatus, Self::Error> {
        self.inner().txpool_status().await.map_err(FromErr::from)
    }
}
