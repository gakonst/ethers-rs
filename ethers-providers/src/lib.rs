#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(clippy::type_complexity)]
#![doc = include_str!("../README.md")]
mod transports;
use futures_util::future::join_all;
pub use transports::*;

mod provider;

// ENS support
pub mod ens;

mod pending_transaction;
pub use pending_transaction::PendingTransaction;

mod pending_escalator;
pub use pending_escalator::EscalatingPending;

mod stream;
pub use futures_util::StreamExt;
pub use stream::{interval, FilterWatcher, TransactionStream, DEFAULT_POLL_INTERVAL};

mod pubsub;
pub use pubsub::{PubsubClient, SubscriptionStream};

pub mod erc;

use async_trait::async_trait;
use auto_impl::auto_impl;
use ethers_core::types::transaction::{eip2718::TypedTransaction, eip2930::AccessListWithGasUsed};
use serde::{de::DeserializeOwned, Serialize};
use std::{error::Error, fmt::Debug, future::Future, pin::Pin};
use url::Url;

pub use provider::{FilterKind, Provider, ProviderError};

// feature-enabled support for dev-rpc methods
#[cfg(feature = "dev-rpc")]
pub use provider::dev_rpc::DevRpcMiddleware;

/// A simple gas escalation policy
pub type EscalationPolicy = Box<dyn Fn(U256, usize) -> U256 + Send + Sync>;

// Helper type alias
#[cfg(target_arch = "wasm32")]
pub(crate) type PinBoxFut<'a, T> = Pin<Box<dyn Future<Output = Result<T, ProviderError>> + 'a>>;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type PinBoxFut<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProviderError>> + Send + 'a>>;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
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
        R: DeserializeOwned;
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

/// Structure used in eth_syncing RPC
#[derive(Clone, Debug)]
pub enum SyncingStatus {
    /// When client is synced to highest block, eth_syncing with return string "false"
    IsFalse,
    /// When client is still syncing past blocks we get IsSyncing information.
    IsSyncing { starting_block: U256, current_block: U256, highest_block: U256 },
}

/// A middleware allows customizing requests send and received from an ethereum node.
///
/// Writing a middleware is as simple as:
/// 1. implementing the [`inner`](crate::Middleware::inner) method to point to the next layer in the
/// "middleware onion", 2. implementing the [`FromErr`](crate::FromErr) trait on your middleware's
/// error type 3. implementing any of the methods you want to override
///
/// ```rust
/// use ethers_providers::{Middleware, FromErr};
/// use ethers_core::types::{U64, TransactionRequest, U256, transaction::eip2718::TypedTransaction};
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
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[auto_impl(&, Box, Arc)]
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

    /// Fill necessary details of a transaction for dispatch
    ///
    /// This function is defined on providers to behave as follows:
    /// 1. populate the `from` field with the default sender
    /// 2. resolve any ENS names in the tx `to` field
    /// 3. Estimate gas usage _without_ access lists
    /// 4. Estimate gas usage _with_ access lists
    /// 5. Enable access lists IFF they are cheaper
    /// 6. Poll and set legacy or 1559 gas prices
    /// 7. Set the chain_id with the provider's, if not already set
    ///
    /// It does NOT set the nonce by default.
    /// It MAY override the gas amount set by the user, if access lists are
    /// cheaper.
    ///
    /// Middleware are encouraged to override any values _before_ delegating
    /// to the inner implementation AND/OR modify the values provided by the
    /// default implementation _after_ delegating.
    ///
    /// E.g. a middleware wanting to double gas prices should consider doing so
    /// _after_ delegating and allowing the default implementation to poll gas.
    async fn fill_transaction(
        &self,
        tx: &mut TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<(), Self::Error> {
        self.inner().fill_transaction(tx, block).await.map_err(FromErr::from)
    }

    async fn get_block_number(&self) -> Result<U64, Self::Error> {
        self.inner().get_block_number().await.map_err(FromErr::from)
    }

    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        self.inner().send_transaction(tx, block).await.map_err(FromErr::from)
    }

    /// Send a transaction with a simple escalation policy.
    ///
    /// `policy` should be a boxed function that maps `original_gas_price`
    /// and `number_of_previous_escalations` -> `new_gas_price`.
    ///
    /// e.g. `Box::new(|start, escalation_index| start * 1250.pow(escalations) /
    /// 1000.pow(escalations))`
    async fn send_escalating<'a>(
        &'a self,
        tx: &TypedTransaction,
        escalations: usize,
        policy: EscalationPolicy,
    ) -> Result<EscalatingPending<'a, Self::Provider>, Self::Error> {
        let mut original = tx.clone();
        self.fill_transaction(&mut original, None).await?;

        // set the nonce, if no nonce is found
        if original.nonce().is_none() {
            let nonce =
                self.get_transaction_count(tx.from().copied().unwrap_or_default(), None).await?;
            original.set_nonce(nonce);
        }

        let gas_price = original.gas_price().expect("filled");
        let sign_futs: Vec<_> = (0..escalations)
            .map(|i| {
                let new_price = policy(gas_price, i);
                let mut r = original.clone();
                r.set_gas_price(new_price);
                r
            })
            .map(|req| async move {
                self.sign_transaction(&req, self.default_sender().unwrap_or_default())
                    .await
                    .map(|sig| req.rlp_signed(&sig))
            })
            .collect();

        // we reverse for convenience. Ensuring that we can always just
        // `pop()` the next tx off the back later
        let mut signed = join_all(sign_futs).await.into_iter().collect::<Result<Vec<_>, _>>()?;
        signed.reverse();

        Ok(EscalatingPending::new(self.provider(), signed))
    }

    async fn resolve_name(&self, ens_name: &str) -> Result<Address, Self::Error> {
        self.inner().resolve_name(ens_name).await.map_err(FromErr::from)
    }

    async fn lookup_address(&self, address: Address) -> Result<String, Self::Error> {
        self.inner().lookup_address(address).await.map_err(FromErr::from)
    }

    async fn resolve_avatar(&self, ens_name: &str) -> Result<Url, Self::Error> {
        self.inner().resolve_avatar(ens_name).await.map_err(FromErr::from)
    }

    async fn resolve_nft(&self, token: erc::ERCNFT) -> Result<Url, Self::Error> {
        self.inner().resolve_nft(token).await.map_err(FromErr::from)
    }

    async fn resolve_field(&self, ens_name: &str, field: &str) -> Result<String, Self::Error> {
        self.inner().resolve_field(ens_name, field).await.map_err(FromErr::from)
    }

    async fn get_block<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<TxHash>>, Self::Error> {
        self.inner().get_block(block_hash_or_number).await.map_err(FromErr::from)
    }

    async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<Transaction>>, Self::Error> {
        self.inner().get_block_with_txs(block_hash_or_number).await.map_err(FromErr::from)
    }

    async fn get_uncle_count<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<U256, Self::Error> {
        self.inner().get_uncle_count(block_hash_or_number).await.map_err(FromErr::from)
    }

    async fn get_uncle<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
        idx: U64,
    ) -> Result<Option<Block<H256>>, Self::Error> {
        self.inner().get_uncle(block_hash_or_number, idx).await.map_err(FromErr::from)
    }

    async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        self.inner().get_transaction_count(from, block).await.map_err(FromErr::from)
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

    async fn syncing(&self) -> Result<SyncingStatus, Self::Error> {
        self.inner().syncing().await.map_err(FromErr::from)
    }

    async fn get_chainid(&self) -> Result<U256, Self::Error> {
        self.inner().get_chainid().await.map_err(FromErr::from)
    }

    async fn get_net_version(&self) -> Result<U64, Self::Error> {
        self.inner().get_net_version().await.map_err(FromErr::from)
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        self.inner().get_balance(from, block).await.map_err(FromErr::from)
    }

    async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<Transaction>, Self::Error> {
        self.inner().get_transaction(transaction_hash).await.map_err(FromErr::from)
    }

    async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        self.inner().get_transaction_receipt(transaction_hash).await.map_err(FromErr::from)
    }

    async fn get_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>, Self::Error> {
        self.inner().get_block_receipts(block).await.map_err(FromErr::from)
    }

    async fn get_gas_price(&self) -> Result<U256, Self::Error> {
        self.inner().get_gas_price().await.map_err(FromErr::from)
    }

    async fn estimate_eip1559_fees(
        &self,
        estimator: Option<fn(U256, Vec<Vec<U256>>) -> (U256, U256)>,
    ) -> Result<(U256, U256), Self::Error> {
        self.inner().estimate_eip1559_fees(estimator).await.map_err(FromErr::from)
    }

    async fn get_accounts(&self) -> Result<Vec<Address>, Self::Error> {
        self.inner().get_accounts().await.map_err(FromErr::from)
    }

    async fn send_raw_transaction<'a>(
        &'a self,
        tx: Bytes,
    ) -> Result<PendingTransaction<'a, Self::Provider>, Self::Error> {
        self.inner().send_raw_transaction(tx).await.map_err(FromErr::from)
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

    /// Sign a transaction via RPC call
    async fn sign_transaction(
        &self,
        tx: &TypedTransaction,
        from: Address,
    ) -> Result<Signature, Self::Error> {
        self.inner().sign_transaction(tx, from).await.map_err(FromErr::from)
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
        self.inner().uninstall_filter(id).await.map_err(FromErr::from)
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
        self.inner().watch_pending_transactions().await.map_err(FromErr::from)
    }

    async fn get_filter_changes<T, R>(&self, id: T) -> Result<Vec<R>, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        R: Serialize + DeserializeOwned + Send + Sync + Debug,
    {
        self.inner().get_filter_changes(id).await.map_err(FromErr::from)
    }

    async fn watch_blocks(&self) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
        self.inner().watch_blocks().await.map_err(FromErr::from)
    }

    async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        self.inner().get_code(at, block).await.map_err(FromErr::from)
    }

    async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: H256,
        block: Option<BlockId>,
    ) -> Result<H256, Self::Error> {
        self.inner().get_storage_at(from, location, block).await.map_err(FromErr::from)
    }

    async fn get_proof<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        locations: Vec<H256>,
        block: Option<BlockId>,
    ) -> Result<EIP1186ProofResponse, Self::Error> {
        self.inner().get_proof(from, locations, block).await.map_err(FromErr::from)
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
        self.inner().trace_call(req, trace_type, block).await.map_err(FromErr::from)
    }

    async fn trace_call_many<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        req: Vec<(T, Vec<TraceType>)>,
        block: Option<BlockNumber>,
    ) -> Result<Vec<BlockTrace>, Self::Error> {
        self.inner().trace_call_many(req, block).await.map_err(FromErr::from)
    }

    /// Traces a call to `eth_sendRawTransaction` without making the call, returning the traces
    async fn trace_raw_transaction(
        &self,
        data: Bytes,
        trace_type: Vec<TraceType>,
    ) -> Result<BlockTrace, Self::Error> {
        self.inner().trace_raw_transaction(data, trace_type).await.map_err(FromErr::from)
    }

    /// Replays a transaction, returning the traces
    async fn trace_replay_transaction(
        &self,
        hash: H256,
        trace_type: Vec<TraceType>,
    ) -> Result<BlockTrace, Self::Error> {
        self.inner().trace_replay_transaction(hash, trace_type).await.map_err(FromErr::from)
    }

    /// Replays all transactions in a block returning the requested traces for each transaction
    async fn trace_replay_block_transactions(
        &self,
        block: BlockNumber,
        trace_type: Vec<TraceType>,
    ) -> Result<Vec<BlockTrace>, Self::Error> {
        self.inner().trace_replay_block_transactions(block, trace_type).await.map_err(FromErr::from)
    }

    /// Returns traces created at given block
    async fn trace_block(&self, block: BlockNumber) -> Result<Vec<Trace>, Self::Error> {
        self.inner().trace_block(block).await.map_err(FromErr::from)
    }

    /// Return traces matching the given filter
    async fn trace_filter(&self, filter: TraceFilter) -> Result<Vec<Trace>, Self::Error> {
        self.inner().trace_filter(filter).await.map_err(FromErr::from)
    }

    /// Returns trace at the given position
    async fn trace_get<T: Into<U64> + Send + Sync>(
        &self,
        hash: H256,
        index: Vec<T>,
    ) -> Result<Trace, Self::Error> {
        self.inner().trace_get(hash, index).await.map_err(FromErr::from)
    }

    /// Returns all traces of a given transaction
    async fn trace_transaction(&self, hash: H256) -> Result<Vec<Trace>, Self::Error> {
        self.inner().trace_transaction(hash).await.map_err(FromErr::from)
    }

    // Parity namespace

    /// Returns all receipts for that block. Must be done on a parity node.
    async fn parity_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>, Self::Error> {
        self.inner().parity_block_receipts(block).await.map_err(FromErr::from)
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
        self.inner().subscribe_pending_txs().await.map_err(FromErr::from)
    }

    async fn subscribe_logs<'a>(
        &'a self,
        filter: &Filter,
    ) -> Result<SubscriptionStream<'a, Self::Provider, Log>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe_logs(filter).await.map_err(FromErr::from)
    }

    async fn fee_history<T: Into<U256> + serde::Serialize + Send + Sync>(
        &self,
        block_count: T,
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
        self.inner().create_access_list(tx, block).await.map_err(FromErr::from)
    }
}

#[cfg(feature = "celo")]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait CeloMiddleware: Middleware {
    async fn get_validators_bls_public_keys<T: Into<BlockId> + Send + Sync>(
        &self,
        block_id: T,
    ) -> Result<Vec<String>, ProviderError> {
        self.provider().get_validators_bls_public_keys(block_id).await.map_err(FromErr::from)
    }
}

pub use test_provider::{GOERLI, MAINNET, RINKEBY, ROPSTEN};

/// Pre-instantiated Infura HTTP clients which rotate through multiple API keys
/// to prevent rate limits
pub mod test_provider {
    use super::*;
    use crate::Http;
    use once_cell::sync::Lazy;
    use std::{convert::TryFrom, iter::Cycle, slice::Iter, sync::Mutex};

    // List of infura keys to rotate through so we don't get rate limited
    const INFURA_KEYS: &[&str] = &[
        "6770454bc6ea42c58aac12978531b93f",
        "7a8769b798b642f6933f2ed52042bd70",
        "631fd9a6539644088297dc605d35fff3",
        "16a8be88795540b9b3903d8de0f7baa5",
        "f4a0bdad42674adab5fc0ac077ffab2b",
        "5c812e02193c4ba793f8c214317582bd",
    ];

    pub static RINKEBY: Lazy<TestProvider> =
        Lazy::new(|| TestProvider::new(INFURA_KEYS, "rinkeby"));
    pub static MAINNET: Lazy<TestProvider> =
        Lazy::new(|| TestProvider::new(INFURA_KEYS, "mainnet"));
    pub static GOERLI: Lazy<TestProvider> = Lazy::new(|| TestProvider::new(INFURA_KEYS, "goerli"));
    pub static ROPSTEN: Lazy<TestProvider> =
        Lazy::new(|| TestProvider::new(INFURA_KEYS, "ropsten"));

    #[derive(Debug)]
    pub struct TestProvider {
        network: String,
        keys: Mutex<Cycle<Iter<'static, &'static str>>>,
    }

    impl TestProvider {
        pub fn new(keys: &'static [&'static str], network: &str) -> Self {
            Self { keys: Mutex::new(keys.iter().cycle()), network: network.to_owned() }
        }

        pub fn provider(&self) -> Provider<Http> {
            let url = format!(
                "https://{}.infura.io/v3/{}",
                self.network,
                self.keys.lock().unwrap().next().unwrap()
            );
            Provider::try_from(url.as_str()).unwrap()
        }

        #[cfg(feature = "ws")]
        pub async fn ws(&self) -> Provider<crate::Ws> {
            let url = format!(
                "wss://{}.infura.io/ws/v3/{}",
                self.network,
                self.keys.lock().unwrap().next().unwrap()
            );
            Provider::connect(url.as_str()).await.unwrap()
        }
    }
}
