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
    /// Error type returned by most operations
    type Error: MiddlewareError<Inner = <<Self as Middleware>::Inner as Middleware>::Error>;
    /// The JSON-RPC client type at the bottom of the stack
    type Provider: JsonRpcClient;
    /// The next-lower middleware in the middleware stack
    type Inner: Middleware<Provider = Self::Provider>;

    /// Get a reference to the next-lower middleware in the middleware stack
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

    /// Return the default sender (if any). This will typically be the
    /// connected node's first address, or the address of a Signer in a lower
    /// middleware stack
    fn default_sender(&self) -> Option<Address> {
        self.inner().default_sender()
    }

    /// Returns the current client version using the `web3_clientVersion` RPC.
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

    /// Get the block number
    async fn get_block_number(&self) -> Result<U64, Self::Error> {
        self.inner().get_block_number().await.map_err(MiddlewareError::from_err)
    }

    /// Sends the transaction to the entire Ethereum network and returns the
    /// transaction's hash. This will consume gas from the account that signed
    /// the transaction. This call will fail if no signer is available, and the
    /// RPC node does  not have an unlocked accounts
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

    ////// Ethereum Naming Service
    // The Ethereum Naming Service (ENS) allows easy to remember and use names to
    // be assigned to Ethereum addresses. Any provider operation which takes an address
    // may also take an ENS name.
    //
    // ENS also provides the ability for a reverse lookup, which determines the name for an address
    // if it has been configured.

    /// Returns the address that the `ens_name` resolves to (or None if not configured).
    ///
    /// # Panics
    ///
    /// If the bytes returned from the ENS registrar/resolver cannot be interpreted as
    /// an address. This should theoretically never happen.
    async fn resolve_name(&self, ens_name: &str) -> Result<Address, Self::Error> {
        self.inner().resolve_name(ens_name).await.map_err(MiddlewareError::from_err)
    }

    /// Returns the ENS name the `address` resolves to (or None if not configured).
    ///
    /// # Panics
    ///
    /// If the bytes returned from the ENS registrar/resolver cannot be interpreted as
    /// a string. This should theoretically never happen.
    async fn lookup_address(&self, address: Address) -> Result<String, Self::Error> {
        self.inner().lookup_address(address).await.map_err(MiddlewareError::from_err)
    }

    /// Returns the avatar HTTP link of the avatar that the `ens_name` resolves to (or None
    /// if not configured)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ethers_providers::{Provider, Http, Middleware};
    /// # async fn foo(provider: Provider<Http>) -> Result<(), Box<dyn std::error::Error>> {
    /// let avatar = provider.resolve_avatar("parishilton.eth").await?;
    /// assert_eq!(avatar.to_string(), "https://i.imgur.com/YW3Hzph.jpg");
    /// # Ok(()) }
    /// ```
    ///
    /// # Panics
    ///
    /// If the bytes returned from the ENS registrar/resolver cannot be interpreted as
    /// a string. This should theoretically never happen.
    async fn resolve_avatar(&self, ens_name: &str) -> Result<Url, Self::Error> {
        self.inner().resolve_avatar(ens_name).await.map_err(MiddlewareError::from_err)
    }

    /// Returns the URL (not necesserily HTTP) of the image behind a token.
    ///
    /// # Example
    /// ```no_run
    /// # use ethers_providers::{Provider, Http, Middleware};
    /// use ethers_providers::erc::ERCNFT;
    /// # async fn foo(provider: Provider<Http>) -> Result<(), Box<dyn std::error::Error>> {
    /// let token = "erc721:0xc92ceddfb8dd984a89fb494c376f9a48b999aafc/9018".parse()?;
    /// let token_image = provider.resolve_nft(token).await?;
    /// assert_eq!(
    ///     token_image.to_string(),
    ///     "https://creature.mypinata.cloud/ipfs/QmNwj3aUzXfG4twV3no7hJRYxLLAWNPk6RrfQaqJ6nVJFa/9018.jpg"
    /// );
    /// # Ok(()) }
    /// ```
    ///
    /// # Panics
    ///
    /// If the bytes returned from the ENS registrar/resolver cannot be interpreted as
    /// a string. This should theoretically never happen.
    async fn resolve_nft(&self, token: erc::ERCNFT) -> Result<Url, Self::Error> {
        self.inner().resolve_nft(token).await.map_err(MiddlewareError::from_err)
    }

    /// Fetch a field for the `ens_name` (no None if not configured).
    ///
    /// # Panics
    ///
    /// If the bytes returned from the ENS registrar/resolver cannot be interpreted as
    /// a string. This should theoretically never happen.
    async fn resolve_field(&self, ens_name: &str, field: &str) -> Result<String, Self::Error> {
        self.inner().resolve_field(ens_name, field).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the block at `block_hash_or_number` (transaction hashes only)
    async fn get_block<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<TxHash>>, Self::Error> {
        self.inner().get_block(block_hash_or_number).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the block at `block_hash_or_number` (full transactions included)
    async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<Transaction>>, Self::Error> {
        self.inner()
            .get_block_with_txs(block_hash_or_number)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Gets the block uncle count at `block_hash_or_number`
    async fn get_uncle_count<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<U256, Self::Error> {
        self.inner().get_uncle_count(block_hash_or_number).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the block uncle at `block_hash_or_number` and `idx`
    async fn get_uncle<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
        idx: U64,
    ) -> Result<Option<Block<H256>>, Self::Error> {
        self.inner().get_uncle(block_hash_or_number, idx).await.map_err(MiddlewareError::from_err)
    }

    /// Returns the nonce of the address
    async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        self.inner().get_transaction_count(from, block).await.map_err(MiddlewareError::from_err)
    }

    /// Sends a transaction to a single Ethereum node and return the estimated amount of gas
    /// required (as a U256) to send it This is free, but only an estimate. Providing too little
    /// gas will result in a transaction being rejected (while still consuming all provided
    /// gas).
    async fn estimate_gas(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        self.inner().estimate_gas(tx, block).await.map_err(MiddlewareError::from_err)
    }

    /// Sends the read-only (constant) transaction to a single Ethereum node and return the result
    /// (as bytes) of executing it. This is free, since it does not change any state on the
    /// blockchain.
    async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        self.inner().call(tx, block).await.map_err(MiddlewareError::from_err)
    }

    /// Return current client syncing status. If IsFalse sync is over.
    async fn syncing(&self) -> Result<SyncingStatus, Self::Error> {
        self.inner().syncing().await.map_err(MiddlewareError::from_err)
    }

    /// Returns the currently configured chain id, a value used in replay-protected
    /// transaction signing as introduced by EIP-155.
    async fn get_chainid(&self) -> Result<U256, Self::Error> {
        self.inner().get_chainid().await.map_err(MiddlewareError::from_err)
    }

    /// Returns the network version.
    async fn get_net_version(&self) -> Result<String, Self::Error> {
        self.inner().get_net_version().await.map_err(MiddlewareError::from_err)
    }

    /// Returns the account's balance
    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        self.inner().get_balance(from, block).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the transaction with `transaction_hash`
    async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<Transaction>, Self::Error> {
        self.inner().get_transaction(transaction_hash).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the transaction with block and index
    async fn get_transaction_by_block_and_index<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
        idx: U64,
    ) -> Result<Option<Transaction>, ProviderError> {
        self.inner()
            .get_transaction_by_block_and_index(block_hash_or_number, idx)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Gets the transaction receipt with `transaction_hash`
    async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        self.inner()
            .get_transaction_receipt(transaction_hash)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Returns all receipts for a block.
    ///
    /// Note that this uses the `eth_getBlockReceipts` RPC, which is
    /// non-standard and currently supported by Erigon.
    async fn get_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>, Self::Error> {
        self.inner().get_block_receipts(block).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the current gas price as estimated by the node
    async fn get_gas_price(&self) -> Result<U256, Self::Error> {
        self.inner().get_gas_price().await.map_err(MiddlewareError::from_err)
    }

    /// Gets a heuristic recommendation of max fee per gas and max priority fee per gas for
    /// EIP-1559 compatible transactions.
    async fn estimate_eip1559_fees(
        &self,
        estimator: Option<fn(U256, Vec<Vec<U256>>) -> (U256, U256)>,
    ) -> Result<(U256, U256), Self::Error> {
        self.inner().estimate_eip1559_fees(estimator).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the accounts on the node
    async fn get_accounts(&self) -> Result<Vec<Address>, Self::Error> {
        self.inner().get_accounts().await.map_err(MiddlewareError::from_err)
    }

    /// Send the raw RLP encoded transaction to the entire Ethereum network and returns the
    /// transaction's hash This will consume gas from the account that signed the transaction.
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

    /// Signs data using a specific account. This account needs to be unlocked,
    /// or the middleware stack must contain a `SignerMiddleware`
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

    /// Returns an array (possibly empty) of logs that match the filter
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

    /// Install a new filter on the node.
    ///
    /// This method is hidden because filter lifecycle  should be managed by
    /// the [`FilterWatcher`]
    #[doc(hidden)]
    async fn new_filter(&self, filter: FilterKind<'_>) -> Result<U256, Self::Error> {
        self.inner().new_filter(filter).await.map_err(MiddlewareError::from_err)
    }

    /// Uninstalls a filter.
    ///
    /// This method is hidden because filter lifecycle  should be managed by
    /// the [`FilterWatcher`]
    #[doc(hidden)]
    async fn uninstall_filter<T: Into<U256> + Send + Sync>(
        &self,
        id: T,
    ) -> Result<bool, Self::Error> {
        self.inner().uninstall_filter(id).await.map_err(MiddlewareError::from_err)
    }

    /// Streams event logs matching the filter.
    ///
    /// This function streams via a polling system, by repeatedly dispatching
    /// RPC requests. If possible, prefer using a WS or IPC connection and the
    /// `stream` interface
    async fn watch<'a>(
        &'a self,
        filter: &Filter,
    ) -> Result<FilterWatcher<'a, Self::Provider, Log>, Self::Error> {
        self.inner().watch(filter).await.map_err(MiddlewareError::from_err)
    }

    /// Streams pending transactions.
    ///
    /// This function streams via a polling system, by repeatedly dispatching
    /// RPC requests. If possible, prefer using a WS or IPC connection and the
    /// `stream` interface
    async fn watch_pending_transactions(
        &self,
    ) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
        self.inner().watch_pending_transactions().await.map_err(MiddlewareError::from_err)
    }

    /// Polling method for a filter, which returns an array of logs which occurred since last poll.
    ///
    /// This method must be called with one of the following return types, depending on the filter
    /// type:
    /// - `eth_newBlockFilter`: [`H256`], returns block hashes
    /// - `eth_newPendingTransactionFilter`: [`H256`], returns transaction hashes
    /// - `eth_newFilter`: [`Log`], returns raw logs
    ///
    /// If one of these types is not used, decoding will fail and the method will
    /// return an error.
    ///
    /// [`H256`]: ethers_core::types::H256
    /// [`Log`]: ethers_core::types::Log
    ///
    /// This method is hidden because filter lifecycle  should be managed by
    /// the [`FilterWatcher`]
    #[doc(hidden)]
    async fn get_filter_changes<T, R>(&self, id: T) -> Result<Vec<R>, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        R: Serialize + DeserializeOwned + Send + Sync + Debug,
    {
        self.inner().get_filter_changes(id).await.map_err(MiddlewareError::from_err)
    }

    /// Streams new block hashes
    ///
    /// This function streams via a polling system, by repeatedly dispatching
    /// RPC requests. If possible, prefer using a WS or IPC connection and the
    /// `stream` interface
    async fn watch_blocks(&self) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
        self.inner().watch_blocks().await.map_err(MiddlewareError::from_err)
    }

    /// Returns the deployed code at a given address
    async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        self.inner().get_code(at, block).await.map_err(MiddlewareError::from_err)
    }

    /// Get the storage of an address for a particular slot location
    async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: H256,
        block: Option<BlockId>,
    ) -> Result<H256, Self::Error> {
        self.inner().get_storage_at(from, location, block).await.map_err(MiddlewareError::from_err)
    }

    /// Returns the EIP-1186 proof response
    /// <https://github.com/ethereum/EIPs/issues/1186>
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
    // NOTE: This will eventually need to be enabled by users explicitly because the personal
    // namespace is being deprecated:
    // Issue: https://github.com/ethereum/go-ethereum/issues/25948
    // PR: https://github.com/ethereum/go-ethereum/pull/26390

    /// Sends the given key to the node to be encrypted with the provided
    /// passphrase and stored.
    ///
    /// The key represents a secp256k1 private key and should be 32 bytes.
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

    /// Prompts the node to decrypt the given account from its keystore.
    ///
    /// If the duration provided is `None`, then the account will be unlocked
    /// indefinitely. Otherwise, the account will be unlocked for the provided
    /// number of seconds.
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

    /// Requests adding the given peer, returning a boolean representing
    /// whether or not the peer was accepted for tracking.
    async fn add_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
        self.inner().add_peer(enode_url).await.map_err(MiddlewareError::from_err)
    }

    /// Requests adding the given peer as a trusted peer, which the node will
    /// always connect to even when its peer slots are full.
    async fn add_trusted_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
        self.inner().add_trusted_peer(enode_url).await.map_err(MiddlewareError::from_err)
    }

    /// Returns general information about the node as well as information about the running p2p
    /// protocols (e.g. `eth`, `snap`).
    async fn node_info(&self) -> Result<NodeInfo, Self::Error> {
        self.inner().node_info().await.map_err(MiddlewareError::from_err)
    }

    /// Returns the list of peers currently connected to the node.
    async fn peers(&self) -> Result<Vec<PeerInfo>, Self::Error> {
        self.inner().peers().await.map_err(MiddlewareError::from_err)
    }

    /// Requests to remove the given peer, returning true if the enode was successfully parsed and
    /// the peer was removed.
    async fn remove_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
        self.inner().remove_peer(enode_url).await.map_err(MiddlewareError::from_err)
    }

    /// Requests to remove the given peer, returning a boolean representing whether or not the
    /// enode url passed was validated. A return value of `true` does not necessarily mean that the
    /// peer was disconnected.
    async fn remove_trusted_peer(&self, enode_url: String) -> Result<bool, Self::Error> {
        self.inner().remove_trusted_peer(enode_url).await.map_err(MiddlewareError::from_err)
    }

    // Miner namespace

    /// Starts the miner.
    async fn start_mining(&self) -> Result<(), Self::Error> {
        self.inner().start_mining().await.map_err(MiddlewareError::from_err)
    }

    /// Stop terminates the miner, both at the consensus engine level as well as at
    /// the block creation level.
    async fn stop_mining(&self) -> Result<(), Self::Error> {
        self.inner().stop_mining().await.map_err(MiddlewareError::from_err)
    }

    // Mempool inspection for Geth's API

    /// Returns the details of all transactions currently pending for inclusion in the next
    /// block(s), as well as the ones that are being scheduled for future execution only.
    /// Ref: [Here](https://geth.ethereum.org/docs/rpc/ns-txpool#txpool_content)
    async fn txpool_content(&self) -> Result<TxpoolContent, Self::Error> {
        self.inner().txpool_content().await.map_err(MiddlewareError::from_err)
    }

    /// Returns a summary of all the transactions currently pending for inclusion in the next
    /// block(s), as well as the ones that are being scheduled for future execution only.
    /// Ref: [Here](https://geth.ethereum.org/docs/rpc/ns-txpool#txpool_inspect)
    async fn txpool_inspect(&self) -> Result<TxpoolInspect, Self::Error> {
        self.inner().txpool_inspect().await.map_err(MiddlewareError::from_err)
    }

    /// Returns the number of transactions currently pending for inclusion in the next block(s), as
    /// well as the ones that are being scheduled for future execution only.
    /// Ref: [Here](https://geth.ethereum.org/docs/rpc/ns-txpool#txpool_status)
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

    /// Replays all transactions in a given block (specified by block number) and returns the traces
    /// configured with passed options
    /// Ref:
    /// [Here](https://geth.ethereum.org/docs/interacting-with-geth/rpc/ns-debug#debugtraceblockbynumber)
    async fn debug_trace_block_by_number(
        &self,
        block: Option<BlockNumber>,
        trace_options: GethDebugTracingOptions,
    ) -> Result<Vec<GethTrace>, Self::Error> {
        self.inner()
            .debug_trace_block_by_number(block, trace_options)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Replays all transactions in a given block (specified by block hash) and returns the traces
    /// configured with passed options
    /// Ref:
    /// [Here](https://geth.ethereum.org/docs/interacting-with-geth/rpc/ns-debug#debugtraceblockbyhash)
    async fn debug_trace_block_by_hash(
        &self,
        block: H256,
        trace_options: GethDebugTracingOptions,
    ) -> Result<Vec<GethTrace>, Self::Error> {
        self.inner()
            .debug_trace_block_by_hash(block, trace_options)
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

    /// Executes given calls and returns a number of possible traces for each
    /// call
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

    /// Create a new subscription
    ///
    /// This method is hidden as subscription lifecycles are intended to be
    /// handled by a [`SubscriptionStream`] object.
    #[doc(hidden)]
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

    /// Instruct the RPC to cancel a subscription by its ID
    ///
    /// This method is hidden as subscription lifecycles are intended to be
    /// handled by a [`SubscriptionStream`] object
    #[doc(hidden)]
    async fn unsubscribe<T>(&self, id: T) -> Result<bool, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().unsubscribe(id).await.map_err(MiddlewareError::from_err)
    }

    /// Subscribe to a stream of incoming blocks.
    ///
    /// This function is only available on pubsub clients, such as Websockets
    /// or IPC. For a polling alternative available over HTTP, use
    /// [`Middleware::watch_blocks`]. However, be aware that polling increases
    /// RPC usage drastically.
    async fn subscribe_blocks(
        &self,
    ) -> Result<SubscriptionStream<'_, Self::Provider, Block<TxHash>>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe_blocks().await.map_err(MiddlewareError::from_err)
    }

    /// Subscribe to a stream of pending transaction hashes.
    ///
    /// This function is only available on pubsub clients, such as Websockets
    /// or IPC. For a polling alternative available over HTTP, use
    /// [`Middleware::watch_pending_transactions`]. However, be aware that
    /// polling increases RPC usage drastically.
    async fn subscribe_pending_txs(
        &self,
    ) -> Result<SubscriptionStream<'_, Self::Provider, TxHash>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe_pending_txs().await.map_err(MiddlewareError::from_err)
    }

    /// Subscribe to a stream of pending transaction bodies.
    ///
    /// This function is only available on pubsub clients, such as Websockets
    /// or IPC. For a polling alternative available over HTTP, use
    /// [`Middleware::watch_pending_transactions`]. However, be aware that
    /// polling increases RPC usage drastically.
    ///
    /// Note: This endpoint is compatible only with Geth client version 1.11.0 or later.
    async fn subscribe_full_pending_txs(
        &self,
    ) -> Result<SubscriptionStream<'_, Self::Provider, Transaction>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe_full_pending_txs().await.map_err(MiddlewareError::from_err)
    }

    /// Subscribe to a stream of event logs matchin the provided [`Filter`].
    ///
    /// This function is only available on pubsub clients, such as Websockets
    /// or IPC. For a polling alternative available over HTTP, use
    /// [`Middleware::watch`]. However, be aware that polling increases
    /// RPC usage drastically.
    async fn subscribe_logs<'a>(
        &'a self,
        filter: &Filter,
    ) -> Result<SubscriptionStream<'a, Self::Provider, Log>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe_logs(filter).await.map_err(MiddlewareError::from_err)
    }

    /// Query the node for a [`FeeHistory`] object. This objct contains
    /// information about the EIP-1559 base fee in past blocks, as well as gas
    /// utilization within those blocks.
    ///
    /// See the
    /// [EIP-1559 documentation](https://eips.ethereum.org/EIPS/eip-1559) for
    /// details
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

    /// Querty the node for an EIP-2930 Access List.
    ///
    /// See the
    /// [EIP-2930 documentation](https://eips.ethereum.org/EIPS/eip-2930) for
    /// details
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
/// Celo-specific extension trait
pub trait CeloMiddleware: Middleware {
    /// Get validator BLS public keys
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

#[cfg(feature = "celo")]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T> CeloMiddleware for T where T: Middleware {}
