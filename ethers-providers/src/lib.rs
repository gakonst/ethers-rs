#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(broken_intra_doc_links)]
#![allow(clippy::type_complexity)]
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
//! The crate has support for WebSockets via Tokio.
//!
//! ```
//! # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
//! # use ethers::providers::Ws;
//! let ws = Ws::connect("ws://localhost:8545").await?;
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
pub use stream::{interval, FilterWatcher, TransactionStream, DEFAULT_POLL_INTERVAL};

mod pubsub;
pub use pubsub::{PubsubClient, SubscriptionStream};

use async_trait::async_trait;
use auto_impl::auto_impl;
use ethers_core::types::transaction::{eip2718::TypedTransaction, eip2930::AccessListWithGasUsed};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{error::Error, fmt::Debug, future::Future, pin::Pin};

pub use provider::{FilterKind, Provider, ProviderError};

// Helper type alias
pub(crate) type PinBoxFut<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProviderError>> + Send + 'a>>;

#[async_trait]
#[auto_impl(&, Box, Arc)]
/// Trait which must be implemented by data transports to be used with the Ethereum
/// JSON-RPC provider.
pub trait JsonRpcClient: Debug + Send + Sync {
    /// A JSON-RPC Error
    type Error: Error + Into<ProviderError>;

    /// Sends a request with the provided JSON-RPC and parameters serialized as JSON
    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: Serialize + DeserializeOwned;
}

use ethers_core::types::*;
pub trait FromErr<T> {
    fn from(src: T) -> Self;
}

/// Calls the future if `item` is None, otherwise returns a `futures::ok`
pub async fn maybe<F, T, E>(item: Option<T>, f: F) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
{
    if let Some(item) = item {
        futures_util::future::ok(item).await
    } else {
        f.await
    }
}

#[async_trait]
#[auto_impl(&, Box, Arc)]
/// A middleware allows customizing requests send and received from an ethereum node.
///
/// Writing a middleware is as simple as:
/// 1. implementing the [`inner`](crate::Middleware::inner) method to point to the next layer in the "middleware onion",
/// 2. implementing the [`FromErr`](crate::FromErr) trait on your middleware's error type
/// 3. implementing any of the methods you want to override
///
/// ```rust
/// use ethers::{providers::{Middleware, FromErr}, types::{U64, TransactionRequest, U256, transaction::eip2718::TypedTransaction}};
/// use thiserror::Error;
/// use async_trait::async_trait;
///
/// #[derive(Debug)]
/// struct MyMiddleware<M>(M);
///
/// #[derive(Error, Debug)]
/// pub enum MyError<M: Middleware> {
///     #[error("{0}")]
///     MiddlewareError(M::Error),
///
///     // Add your middleware's specific errors here
/// }
///
/// impl<M: Middleware> FromErr<M::Error> for MyError<M> {
///     fn from(src: M::Error) -> MyError<M> {
///         MyError::MiddlewareError(src)
///     }
/// }
///
/// #[async_trait]
/// impl<M> Middleware for MyMiddleware<M>
/// where
///     M: Middleware,
/// {
///     type Error = MyError<M>;
///     type Provider = M::Provider;
///     type Inner = M;
///
///     fn inner(&self) -> &M {
///         &self.0
///     }
///
///     /// Overrides the default `get_block_number` method to always return 0
///     async fn get_block_number(&self) -> Result<U64, Self::Error> {
///         Ok(U64::zero())
///     }
///
///     /// Overrides the default `estimate_gas` method to log that it was called,
///     /// before forwarding the call to the next layer.
///     async fn estimate_gas(&self, tx: &TypedTransaction) -> Result<U256, Self::Error> {
///         println!("Estimating gas...");
///         self.inner().estimate_gas(tx).await.map_err(FromErr::from)
///     }
/// }
/// ```
pub trait Middleware: Sync + Send + Debug {
    type Error: Sync + Send + Error + FromErr<<Self::Inner as Middleware>::Error>;
    type Provider: JsonRpcClient;
    type Inner: Middleware<Provider = Self::Provider>;

    /// The next middleware in the stack
    fn inner(&self) -> &Self::Inner;

    /// The HTTP or Websocket provider.
    fn provider(&self) -> &Provider<Self::Provider> {
        self.inner().provider()
    }

    fn default_sender(&self) -> Option<Address> {
        self.inner().default_sender()
    }

    async fn client_version(&self) -> Result<String, Self::Error> {
        self.inner().client_version().await.map_err(FromErr::from)
    }

    /// Helper for filling a transaction
    async fn fill_transaction(
        &self,
        tx: &mut TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<(), Self::Error> {
        let tx_clone = tx.clone();

        // TODO: Maybe deduplicate the code in a nice way
        match tx {
            TypedTransaction::Legacy(ref mut inner) => {
                if let Some(NameOrAddress::Name(ref ens_name)) = inner.to {
                    let addr = self.resolve_name(ens_name).await?;
                    inner.to = Some(addr.into());
                };

                if inner.from.is_none() {
                    inner.from = self.default_sender();
                }

                let (gas_price, gas) = futures_util::try_join!(
                    maybe(inner.gas_price, self.get_gas_price()),
                    maybe(inner.gas, self.estimate_gas(&tx_clone)),
                )?;
                inner.gas = Some(gas);
                inner.gas_price = Some(gas_price);
            }
            TypedTransaction::Eip2930(inner) => {
                if let Ok(lst) = self.create_access_list(&tx_clone, block).await {
                    inner.access_list = lst.access_list;
                }

                if let Some(NameOrAddress::Name(ref ens_name)) = inner.tx.to {
                    let addr = self.resolve_name(ens_name).await?;
                    inner.tx.to = Some(addr.into());
                };

                if inner.tx.from.is_none() {
                    inner.tx.from = self.default_sender();
                }

                let (gas_price, gas) = futures_util::try_join!(
                    maybe(inner.tx.gas_price, self.get_gas_price()),
                    maybe(inner.tx.gas, self.estimate_gas(&tx_clone)),
                )?;
                inner.tx.gas = Some(gas);
                inner.tx.gas_price = Some(gas_price);
            }
            TypedTransaction::Eip1559(inner) => {
                if let Ok(lst) = self.create_access_list(&tx_clone, block).await {
                    inner.access_list = lst.access_list;
                }

                if let Some(NameOrAddress::Name(ref ens_name)) = inner.to {
                    let addr = self.resolve_name(ens_name).await?;
                    inner.to = Some(addr.into());
                };

                if inner.from.is_none() {
                    inner.from = self.default_sender();
                }

                let (max_priority_fee_per_gas, max_fee_per_gas, gas) = futures_util::try_join!(
                    // TODO: Replace with algorithms using eth_feeHistory
                    maybe(inner.max_priority_fee_per_gas, self.get_gas_price()),
                    maybe(inner.max_fee_per_gas, self.get_gas_price()),
                    maybe(inner.gas, self.estimate_gas(&tx_clone)),
                )?;
                inner.gas = Some(gas);
                inner.max_fee_per_gas = Some(max_fee_per_gas);
                inner.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
            }
        };

        Ok(())
    }

    async fn get_block_number(&self) -> Result<U64, Self::Error> {
        self.inner().get_block_number().await.map_err(FromErr::from)
    }

    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
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
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        self.inner()
            .get_transaction_count(from, block)
            .await
            .map_err(FromErr::from)
    }

    async fn estimate_gas(&self, tx: &TypedTransaction) -> Result<U256, Self::Error> {
        self.inner().estimate_gas(tx).await.map_err(FromErr::from)
    }

    async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        self.inner().call(tx, block).await.map_err(FromErr::from)
    }

    async fn get_chainid(&self) -> Result<U256, Self::Error> {
        self.inner().get_chainid().await.map_err(FromErr::from)
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
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

    async fn get_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>, Self::Error> {
        self.inner()
            .get_block_receipts(block)
            .await
            .map_err(FromErr::from)
    }

    async fn get_gas_price(&self) -> Result<U256, Self::Error> {
        self.inner().get_gas_price().await.map_err(FromErr::from)
    }

    async fn get_accounts(&self) -> Result<Vec<Address>, Self::Error> {
        self.inner().get_accounts().await.map_err(FromErr::from)
    }

    async fn send_raw_transaction<'a>(
        &'a self,
        tx: Bytes,
    ) -> Result<PendingTransaction<'a, Self::Provider>, Self::Error> {
        self.inner()
            .send_raw_transaction(tx)
            .await
            .map_err(FromErr::from)
    }

    /// This returns true if either the middleware stack contains a `SignerMiddleware`, or the
    /// JSON-RPC provider has an unlocked key that can sign using the `eth_sign` call. If none of
    /// the above conditions are met, then the middleware stack is not capable of signing data.
    async fn is_signer(&self) -> bool {
        self.inner().is_signer().await
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
        R: Serialize + DeserializeOwned + Send + Sync + Debug,
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
        block: Option<BlockId>,
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
        block: Option<BlockId>,
    ) -> Result<H256, Self::Error> {
        self.inner()
            .get_storage_at(from, location, block)
            .await
            .map_err(FromErr::from)
    }

    // Mempool inspection for Geth's API

    async fn txpool_content(&self) -> Result<TxpoolContent, Self::Error> {
        self.inner().txpool_content().await.map_err(FromErr::from)
    }

    async fn txpool_inspect(&self) -> Result<TxpoolInspect, Self::Error> {
        self.inner().txpool_inspect().await.map_err(FromErr::from)
    }

    async fn txpool_status(&self) -> Result<TxpoolStatus, Self::Error> {
        self.inner().txpool_status().await.map_err(FromErr::from)
    }

    // Parity `trace` support

    /// Executes the given call and returns a number of possible traces for it
    async fn trace_call<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        req: T,
        trace_type: Vec<TraceType>,
        block: Option<BlockNumber>,
    ) -> Result<BlockTrace, Self::Error> {
        self.inner()
            .trace_call(req, trace_type, block)
            .await
            .map_err(FromErr::from)
    }

    /// Traces a call to `eth_sendRawTransaction` without making the call, returning the traces
    async fn trace_raw_transaction(
        &self,
        data: Bytes,
        trace_type: Vec<TraceType>,
    ) -> Result<BlockTrace, Self::Error> {
        self.inner()
            .trace_raw_transaction(data, trace_type)
            .await
            .map_err(FromErr::from)
    }

    /// Replays a transaction, returning the traces
    async fn trace_replay_transaction(
        &self,
        hash: H256,
        trace_type: Vec<TraceType>,
    ) -> Result<BlockTrace, Self::Error> {
        self.inner()
            .trace_replay_transaction(hash, trace_type)
            .await
            .map_err(FromErr::from)
    }

    /// Replays all transactions in a block returning the requested traces for each transaction
    async fn trace_replay_block_transactions(
        &self,
        block: BlockNumber,
        trace_type: Vec<TraceType>,
    ) -> Result<Vec<BlockTrace>, Self::Error> {
        self.inner()
            .trace_replay_block_transactions(block, trace_type)
            .await
            .map_err(FromErr::from)
    }

    /// Returns traces created at given block
    async fn trace_block(&self, block: BlockNumber) -> Result<Vec<Trace>, Self::Error> {
        self.inner().trace_block(block).await.map_err(FromErr::from)
    }

    /// Return traces matching the given filter
    async fn trace_filter(&self, filter: TraceFilter) -> Result<Vec<Trace>, Self::Error> {
        self.inner()
            .trace_filter(filter)
            .await
            .map_err(FromErr::from)
    }

    /// Returns trace at the given position
    async fn trace_get<T: Into<U64> + Send + Sync>(
        &self,
        hash: H256,
        index: Vec<T>,
    ) -> Result<Trace, Self::Error> {
        self.inner()
            .trace_get(hash, index)
            .await
            .map_err(FromErr::from)
    }

    /// Returns all traces of a given transaction
    async fn trace_transaction(&self, hash: H256) -> Result<Vec<Trace>, Self::Error> {
        self.inner()
            .trace_transaction(hash)
            .await
            .map_err(FromErr::from)
    }

    // Parity namespace

    /// Returns all receipts for that block. Must be done on a parity node.
    async fn parity_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>, Self::Error> {
        self.inner()
            .parity_block_receipts(block)
            .await
            .map_err(FromErr::from)
    }

    async fn subscribe<T, R>(
        &self,
        params: T,
    ) -> Result<SubscriptionStream<'_, Self::Provider, R>, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send + Sync,
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe(params).await.map_err(FromErr::from)
    }

    async fn unsubscribe<T>(&self, id: T) -> Result<bool, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().unsubscribe(id).await.map_err(FromErr::from)
    }

    async fn subscribe_blocks(
        &self,
    ) -> Result<SubscriptionStream<'_, Self::Provider, Block<TxHash>>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe_blocks().await.map_err(FromErr::from)
    }

    async fn subscribe_pending_txs(
        &self,
    ) -> Result<SubscriptionStream<'_, Self::Provider, TxHash>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner()
            .subscribe_pending_txs()
            .await
            .map_err(FromErr::from)
    }

    async fn subscribe_logs<'a>(
        &'a self,
        filter: &Filter,
    ) -> Result<SubscriptionStream<'a, Self::Provider, Log>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner()
            .subscribe_logs(filter)
            .await
            .map_err(FromErr::from)
    }

    async fn fee_history(
        &self,
        block_count: u64,
        last_block: BlockNumber,
        reward_percentiles: &[f64],
    ) -> Result<FeeHistory, Self::Error> {
        self.inner()
            .fee_history(block_count, last_block, reward_percentiles)
            .await
            .map_err(FromErr::from)
    }

    async fn create_access_list(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<AccessListWithGasUsed, Self::Error> {
        self.inner()
            .create_access_list(tx, block)
            .await
            .map_err(FromErr::from)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeeHistory {
    pub base_fee_per_gas: Vec<U256>,
    pub gas_used_ratio: Vec<f64>,
    pub oldest_block: u64,
    pub reward: Vec<Vec<U256>>,
}

#[cfg(feature = "celo")]
#[async_trait]
pub trait CeloMiddleware: Middleware {
    async fn get_validators_bls_public_keys<T: Into<BlockId> + Send + Sync>(
        &self,
        block_id: T,
    ) -> Result<Vec<String>, ProviderError> {
        self.provider()
            .get_validators_bls_public_keys(block_id)
            .await
            .map_err(FromErr::from)
    }
}
