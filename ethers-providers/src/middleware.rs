use async_trait::async_trait;
use auto_impl::auto_impl;
use ethers_core::types::{
    transaction::{eip2718::TypedTransaction, eip2930::AccessListWithGasUsed},
    *,
};
use futures_util::future::join_all;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use url::Url;

use crate::{
    erc, EscalatingPending, EscalationPolicy, FilterKind, FilterWatcher, JsonRpcClient, LogQuery,
    MiddlewareError, NodeInfo, PeerInfo, PendingTransaction, Provider, ProviderError, PubsubClient,
    SubscriptionStream,
};

/// A middleware allows customizing requests send and received from an ethereum node.
///
/// Writing a middleware is as simple as:
/// 1. implementing the [`inner`](crate::Middleware::inner) method to point to the next layer in the
/// "middleware onion", 2. implementing the
/// [`MiddlewareError`](crate::MiddlewareError) trait on your middleware's
/// error type 3. implementing any of the methods you want to override
///
/// ```
/// use ethers_providers::{Middleware, MiddlewareError};
/// use ethers_core::types::{U64, TransactionRequest, U256, transaction::eip2718::TypedTransaction, BlockId};
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
/// impl<M: Middleware> MiddlewareError for MyError<M> {
///     type Inner = M::Error;
///
///     fn from_err(src: M::Error) -> MyError<M> {
///         MyError::MiddlewareError(src)
///     }
///
///     fn as_inner(&self) -> Option<&Self::Inner> {
///         match self {
///             MyError::MiddlewareError(e) => Some(e),
///             _ => None,
///         }
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
///     async fn estimate_gas(&self, tx: &TypedTransaction, block: Option<BlockId>) -> Result<U256, Self::Error> {
///         println!("Estimating gas...");
///         self.inner().estimate_gas(tx, block).await.map_err(MiddlewareError::from_err)
///     }
/// }
/// ```
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[auto_impl(&, Box, Arc)]
pub trait Middleware: Sync + Send + Debug {
    type Error: MiddlewareError<Inner = <<Self as Middleware>::Inner as Middleware>::Error>;
    type Provider: JsonRpcClient;
    type Inner: Middleware<Provider = Self::Provider>;

    /// The next middleware in the stack
    fn inner(&self) -> &Self::Inner;

    /// Convert a provider error into the associated error type by successively
    /// converting it to every intermediate middleware error
    fn convert_err(p: ProviderError) -> Self::Error {
        Self::Error::from_provider_err(p)
    }

    /// The HTTP or Websocket provider.
    fn provider(&self) -> &Provider<Self::Provider> {
        self.inner().provider()
    }

    fn default_sender(&self) -> Option<Address> {
        self.inner().default_sender()
    }

    async fn client_version(&self) -> Result<String, Self::Error> {
        self.inner().client_version().await.map_err(MiddlewareError::from_err)
    }

    /// Fill necessary details of a transaction for dispatch
    ///
    /// This function is defined on providers to behave as follows:
    /// 1. populate the `from` field with the default sender
    /// 2. resolve any ENS names in the tx `to` field
    /// 3. Estimate gas usage
    /// 4. Poll and set legacy or 1559 gas prices
    /// 5. Set the chain_id with the provider's, if not already set
    ///
    /// It does NOT set the nonce by default.
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
        self.inner().fill_transaction(tx, block).await.map_err(MiddlewareError::from_err)
    }

    async fn get_block_number(&self) -> Result<U64, Self::Error> {
        self.inner().get_block_number().await.map_err(MiddlewareError::from_err)
    }

    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        self.inner().send_transaction(tx, block).await.map_err(MiddlewareError::from_err)
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
        self.inner().resolve_name(ens_name).await.map_err(MiddlewareError::from_err)
    }

    async fn lookup_address(&self, address: Address) -> Result<String, Self::Error> {
        self.inner().lookup_address(address).await.map_err(MiddlewareError::from_err)
    }

    async fn resolve_avatar(&self, ens_name: &str) -> Result<Url, Self::Error> {
        self.inner().resolve_avatar(ens_name).await.map_err(MiddlewareError::from_err)
    }

    async fn resolve_nft(&self, token: erc::ERCNFT) -> Result<Url, Self::Error> {
        self.inner().resolve_nft(token).await.map_err(MiddlewareError::from_err)
    }

    async fn resolve_field(&self, ens_name: &str, field: &str) -> Result<String, Self::Error> {
        self.inner().resolve_field(ens_name, field).await.map_err(MiddlewareError::from_err)
    }

    async fn get_block<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<TxHash>>, Self::Error> {
        self.inner().get_block(block_hash_or_number).await.map_err(MiddlewareError::from_err)
    }

    async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<Transaction>>, Self::Error> {
        self.inner()
            .get_block_with_txs(block_hash_or_number)
            .await
            .map_err(MiddlewareError::from_err)
    }

    async fn get_uncle_count<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<U256, Self::Error> {
        self.inner().get_uncle_count(block_hash_or_number).await.map_err(MiddlewareError::from_err)
    }

    async fn get_uncle<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
        idx: U64,
    ) -> Result<Option<Block<H256>>, Self::Error> {
        self.inner().get_uncle(block_hash_or_number, idx).await.map_err(MiddlewareError::from_err)
    }

    async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        self.inner().get_transaction_count(from, block).await.map_err(MiddlewareError::from_err)
    }

    async fn estimate_gas(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        self.inner().estimate_gas(tx, block).await.map_err(MiddlewareError::from_err)
    }

    async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        self.inner().call(tx, block).await.map_err(MiddlewareError::from_err)
    }

    async fn syncing(&self) -> Result<SyncingStatus, Self::Error> {
        self.inner().syncing().await.map_err(MiddlewareError::from_err)
    }

    async fn get_chainid(&self) -> Result<U256, Self::Error> {
        self.inner().get_chainid().await.map_err(MiddlewareError::from_err)
    }

    async fn get_net_version(&self) -> Result<String, Self::Error> {
        self.inner().get_net_version().await.map_err(MiddlewareError::from_err)
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        self.inner().get_balance(from, block).await.map_err(MiddlewareError::from_err)
    }

    async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<Transaction>, Self::Error> {
        self.inner().get_transaction(transaction_hash).await.map_err(MiddlewareError::from_err)
    }

    async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        self.inner()
            .get_transaction_receipt(transaction_hash)
            .await
            .map_err(MiddlewareError::from_err)
    }

    async fn get_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>, Self::Error> {
        self.inner().get_block_receipts(block).await.map_err(MiddlewareError::from_err)
    }

    async fn get_gas_price(&self) -> Result<U256, Self::Error> {
        self.inner().get_gas_price().await.map_err(MiddlewareError::from_err)
    }

    async fn estimate_eip1559_fees(
        &self,
        estimator: Option<fn(U256, Vec<Vec<U256>>) -> (U256, U256)>,
    ) -> Result<(U256, U256), Self::Error> {
        self.inner().estimate_eip1559_fees(estimator).await.map_err(MiddlewareError::from_err)
    }

    async fn get_accounts(&self) -> Result<Vec<Address>, Self::Error> {
        self.inner().get_accounts().await.map_err(MiddlewareError::from_err)
    }

    async fn send_raw_transaction<'a>(
        &'a self,
        tx: Bytes,
    ) -> Result<PendingTransaction<'a, Self::Provider>, Self::Error> {
        self.inner().send_raw_transaction(tx).await.map_err(MiddlewareError::from_err)
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
        self.inner().sign(data, from).await.map_err(MiddlewareError::from_err)
    }

    /// Sign a transaction via RPC call
    async fn sign_transaction(
        &self,
        tx: &TypedTransaction,
        from: Address,
    ) -> Result<Signature, Self::Error> {
        self.inner().sign_transaction(tx, from).await.map_err(MiddlewareError::from_err)
    }

    ////// Contract state

    async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, Self::Error> {
        self.inner().get_logs(filter).await.map_err(MiddlewareError::from_err)
    }

    /// Returns a stream of logs are loaded in pages of given page size
    fn get_logs_paginated<'a>(
        &'a self,
        filter: &Filter,
        page_size: u64,
    ) -> LogQuery<'a, Self::Provider> {
        self.inner().get_logs_paginated(filter, page_size)
    }

    async fn new_filter(&self, filter: FilterKind<'_>) -> Result<U256, Self::Error> {
        self.inner().new_filter(filter).await.map_err(MiddlewareError::from_err)
    }

    async fn uninstall_filter<T: Into<U256> + Send + Sync>(
        &self,
        id: T,
    ) -> Result<bool, Self::Error> {
        self.inner().uninstall_filter(id).await.map_err(MiddlewareError::from_err)
    }

    async fn watch<'a>(
        &'a self,
        filter: &Filter,
    ) -> Result<FilterWatcher<'a, Self::Provider, Log>, Self::Error> {
        self.inner().watch(filter).await.map_err(MiddlewareError::from_err)
    }

    async fn watch_pending_transactions(
        &self,
    ) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
        self.inner().watch_pending_transactions().await.map_err(MiddlewareError::from_err)
    }

    async fn get_filter_changes<T, R>(&self, id: T) -> Result<Vec<R>, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        R: Serialize + DeserializeOwned + Send + Sync + Debug,
    {
        self.inner().get_filter_changes(id).await.map_err(MiddlewareError::from_err)
    }

    async fn watch_blocks(&self) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
        self.inner().watch_blocks().await.map_err(MiddlewareError::from_err)
    }

    async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        self.inner().get_code(at, block).await.map_err(MiddlewareError::from_err)
    }

    async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: H256,
        block: Option<BlockId>,
    ) -> Result<H256, Self::Error> {
        self.inner().get_storage_at(from, location, block).await.map_err(MiddlewareError::from_err)
    }

    async fn get_proof<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        locations: Vec<H256>,
        block: Option<BlockId>,
    ) -> Result<EIP1186ProofResponse, Self::Error> {
        self.inner().get_proof(from, locations, block).await.map_err(MiddlewareError::from_err)
    }

    /// Returns an indication if this node is currently mining.
    async fn mining(&self) -> Result<bool, Self::Error> {
        self.inner().mining().await.map_err(MiddlewareError::from_err)
    }

    // Personal namespace

    async fn import_raw_key(
        &self,
        private_key: Bytes,
        passphrase: String,
    ) -> Result<Address, Self::Error> {
        self.inner()
            .import_raw_key(private_key, passphrase)
            .await
            .map_err(MiddlewareError::from_err)
    }

    async fn unlock_account<T: Into<Address> + Send + Sync>(
        &self,
        account: T,
        passphrase: String,
        duration: Option<u64>,
    ) -> Result<bool, Self::Error> {
        self.inner()
            .unlock_account(account, passphrase, duration)
            .await
            .map_err(MiddlewareError::from_err)
    }

    // Admin namespace

    async fn add_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
        self.inner().add_peer(enode_url).await.map_err(MiddlewareError::from_err)
    }

    async fn add_trusted_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
        self.inner().add_trusted_peer(enode_url).await.map_err(MiddlewareError::from_err)
    }

    async fn node_info(&self) -> Result<NodeInfo, Self::Error> {
        self.inner().node_info().await.map_err(MiddlewareError::from_err)
    }

    async fn peers(&self) -> Result<Vec<PeerInfo>, Self::Error> {
        self.inner().peers().await.map_err(MiddlewareError::from_err)
    }

    async fn remove_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
        self.inner().remove_peer(enode_url).await.map_err(MiddlewareError::from_err)
    }

    async fn remove_trusted_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
        self.inner().remove_trusted_peer(enode_url).await.map_err(MiddlewareError::from_err)
    }

    // Miner namespace

    /// Starts the miner with the given number of threads. If threads is nil, the number of workers
    /// started is equal to the number of logical CPUs that are usable by this process. If mining
    /// is already running, this method adjust the number of threads allowed to use and updates the
    /// minimum price required by the transaction pool.
    async fn start_mining(&self, threads: Option<usize>) -> Result<(), Self::Error> {
        self.inner().start_mining(threads).await.map_err(MiddlewareError::from_err)
    }

    /// Stop terminates the miner, both at the consensus engine level as well as at
    /// the block creation level.
    async fn stop_mining(&self) -> Result<(), Self::Error> {
        self.inner().stop_mining().await.map_err(MiddlewareError::from_err)
    }

    // Mempool inspection for Geth's API

    async fn txpool_content(&self) -> Result<TxpoolContent, Self::Error> {
        self.inner().txpool_content().await.map_err(MiddlewareError::from_err)
    }

    async fn txpool_inspect(&self) -> Result<TxpoolInspect, Self::Error> {
        self.inner().txpool_inspect().await.map_err(MiddlewareError::from_err)
    }

    async fn txpool_status(&self) -> Result<TxpoolStatus, Self::Error> {
        self.inner().txpool_status().await.map_err(MiddlewareError::from_err)
    }

    // Geth `trace` support

    /// After replaying any previous transactions in the same block,
    /// Replays a transaction, returning the traces configured with passed options
    async fn debug_trace_transaction(
        &self,
        tx_hash: TxHash,
        trace_options: GethDebugTracingOptions,
    ) -> Result<GethTrace, Self::Error> {
        self.inner()
            .debug_trace_transaction(tx_hash, trace_options)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Executes the given call and returns a number of possible traces for it
    async fn debug_trace_call<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        req: T,
        block: Option<BlockId>,
        trace_options: GethDebugTracingCallOptions,
    ) -> Result<GethTrace, Self::Error> {
        self.inner()
            .debug_trace_call(req, block, trace_options)
            .await
            .map_err(MiddlewareError::from_err)
    }

    // Parity `trace` support

    /// Executes the given call and returns a number of possible traces for it
    async fn trace_call<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        req: T,
        trace_type: Vec<TraceType>,
        block: Option<BlockNumber>,
    ) -> Result<BlockTrace, Self::Error> {
        self.inner().trace_call(req, trace_type, block).await.map_err(MiddlewareError::from_err)
    }

    async fn trace_call_many<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        req: Vec<(T, Vec<TraceType>)>,
        block: Option<BlockNumber>,
    ) -> Result<Vec<BlockTrace>, Self::Error> {
        self.inner().trace_call_many(req, block).await.map_err(MiddlewareError::from_err)
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
            .map_err(MiddlewareError::from_err)
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
            .map_err(MiddlewareError::from_err)
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
            .map_err(MiddlewareError::from_err)
    }

    /// Returns traces created at given block
    async fn trace_block(&self, block: BlockNumber) -> Result<Vec<Trace>, Self::Error> {
        self.inner().trace_block(block).await.map_err(MiddlewareError::from_err)
    }

    /// Return traces matching the given filter
    async fn trace_filter(&self, filter: TraceFilter) -> Result<Vec<Trace>, Self::Error> {
        self.inner().trace_filter(filter).await.map_err(MiddlewareError::from_err)
    }

    /// Returns trace at the given position
    async fn trace_get<T: Into<U64> + Send + Sync>(
        &self,
        hash: H256,
        index: Vec<T>,
    ) -> Result<Trace, Self::Error> {
        self.inner().trace_get(hash, index).await.map_err(MiddlewareError::from_err)
    }

    /// Returns all traces of a given transaction
    async fn trace_transaction(&self, hash: H256) -> Result<Vec<Trace>, Self::Error> {
        self.inner().trace_transaction(hash).await.map_err(MiddlewareError::from_err)
    }

    // Parity namespace

    /// Returns all receipts for that block. Must be done on a parity node.
    async fn parity_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>, Self::Error> {
        self.inner().parity_block_receipts(block).await.map_err(MiddlewareError::from_err)
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
        self.inner().subscribe(params).await.map_err(MiddlewareError::from_err)
    }

    async fn unsubscribe<T>(&self, id: T) -> Result<bool, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().unsubscribe(id).await.map_err(MiddlewareError::from_err)
    }

    async fn subscribe_blocks(
        &self,
    ) -> Result<SubscriptionStream<'_, Self::Provider, Block<TxHash>>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe_blocks().await.map_err(MiddlewareError::from_err)
    }

    async fn subscribe_pending_txs(
        &self,
    ) -> Result<SubscriptionStream<'_, Self::Provider, TxHash>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe_pending_txs().await.map_err(MiddlewareError::from_err)
    }

    async fn subscribe_logs<'a>(
        &'a self,
        filter: &Filter,
    ) -> Result<SubscriptionStream<'a, Self::Provider, Log>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe_logs(filter).await.map_err(MiddlewareError::from_err)
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
            .map_err(MiddlewareError::from_err)
    }

    async fn create_access_list(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<AccessListWithGasUsed, Self::Error> {
        self.inner().create_access_list(tx, block).await.map_err(MiddlewareError::from_err)
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
        self.provider()
            .get_validators_bls_public_keys(block_id)
            .await
            .map_err(MiddlewareError::from_err)
    }
}
