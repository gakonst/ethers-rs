#[cfg(feature = "ipc")]
use std::path::Path;

use std::{borrow::Cow, error, fmt, sync::Arc};

use serde::{Deserialize, Serialize};

use ethers_core::types::{
    transaction::eip2718::TypedTransaction, Address, Block, Bytes, FeeHistory, Log, Transaction,
    TransactionReceipt, TransactionRequest, H256, U256, U64,
};

#[cfg(all(unix, feature = "ipc"))]
use crate::connection::ipc::{Ipc, IpcError};
use crate::{
    batch::{BatchCall, BatchError},
    connection::{self, noop::Noop, ConnectionError},
    jsonrpc as rpc,
    sub::SubscriptionStream,
    types::{BlockNumber, Filter, SyncStatus, TransactionCall},
    CallParams, Connection, DuplexConnection, RpcCall,
};

/// A provider for Ethereum JSON-RPC API calls.
///
/// This type provides type-safe bindings to all RPC calls defined in the
/// [JSON-RPC API specification](https://eth.wiki/json-rpc/API).
#[derive(Clone, Copy)]
pub struct Provider<C> {
    /// The provider's wrapped connection.
    pub connection: C,
}

impl<C> Provider<C> {
    /// Returns a new [`Provider`] wrapping the given `connection`.
    pub const fn new(connection: C) -> Self {
        Self { connection }
    }
}

impl<C: Connection + 'static> Provider<C> {
    /// Borrows the underlying [`Connection`] and returns a new provider that
    /// can be cheaply cloned and copied.
    pub const fn borrow(&self) -> Provider<&'_ C> {
        let connection = &self.connection;
        Provider { connection }
    }
}

impl<'a, C: Connection> Provider<&'a C> {
    /// Converts this [`Provider`] into one using a [`Connection`] trait object.
    pub fn to_dyn(self) -> Provider<&'a dyn Connection> {
        let connection = self.connection as _;
        Provider { connection }
    }
}

impl<C: Connection + 'static> Provider<Arc<C>> {
    /// Converts the [`Provider`] into one using a [`Connection`] trait object.
    pub fn to_dyn(self) -> Provider<Arc<dyn Connection>> {
        let connection = self.connection as _;
        Provider { connection }
    }
}

impl Provider<Noop> {
    /// Creates a new [`Noop`] connection
    /// provider.
    pub const fn noop() -> Self {
        Self { connection: Noop }
    }
}

#[cfg(all(unix, feature = "ipc"))]
impl Provider<Arc<Ipc>> {
    /// Attempts to establish a connection with the IPC socket at the given
    /// `path`.
    ///
    /// # Errors
    ///
    /// This fails, if the file at `path` is not a valid IPC socket.    
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self, IpcError> {
        let connection = Ipc::connect(path).await?;
        Ok(Self { connection: Arc::new(connection) })
    }
}

impl Provider<Arc<dyn Connection>> {
    /// Attempts to connect to any of the available connections based on the
    /// given `path`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ethers_connections::Provider;
    ///
    /// # async fn example_connect() -> Result<(), Box<dyn std::error::Error>> {
    /// // connects via HTTP
    /// let provider = Provider::connect("http://localhost:8545").await?;
    /// // connect via websocket
    /// let provider = Provider::connect("ws://localhost:8546").await?;
    /// // connects to a local IPC socket
    /// let provider = Provider::connect("ipc:///home/user/.ethereum/geth.ipc").await?;
    /// let provider = Provider::connect("/home/user/.ethereum/geth.ipc").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///  
    /// Fails, if the selected connection can not be established.
    ///
    /// # Panics
    ///
    /// Panics, if a connection is selected which has not been feature-enabled
    /// at compile time, e.g., if a HTTP url is given but the `http` cargo
    /// feature is not enabled.
    pub async fn connect(path: &str) -> Result<Self, Box<ConnectionError>> {
        let connection: Arc<dyn Connection> = if path.starts_with("http") {
            #[cfg(feature = "http")]
            {
                let http =
                    connection::http::Http::new(path).map_err(ConnectionError::connection)?;
                Arc::new(http)
            }
            #[cfg(not(feature = "http"))]
            {
                panic!("path starts with http/https, but `http` cargo feature is not enabled");
            }
        } else if path.starts_with("ws") {
            #[cfg(feature = "ws")]
            {
                todo!("...")
            }
            #[cfg(not(feature = "ws"))]
            {
                panic!("path starts with ws/wss, but `ws` cargo feature is not enabled");
            }
        } else {
            #[cfg(feature = "ipc")]
            {
                // the path is allowed start with "ipc://"
                let ipc = connection::ipc::Ipc::connect(path.trim_start_matches("ipc://"))
                    .await
                    .map_err(ConnectionError::connection)?;
                Arc::new(ipc)
            }
            #[cfg(not(feature = "ipc"))]
            {
                todo!("ipc path detected, but `ipc` cargo feature is not enabled");
            }
        };

        #[allow(unreachable_code)]
        Ok(Self { connection })
    }
}

impl Provider<Arc<dyn DuplexConnection>> {
    /// Attempts to connect to any of the available connections based on the
    /// given `path`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ethers_connections::{Provider};
    ///
    /// # async fn example_connect_duplex() -> Result<(), Box<dyn std::error::Error>> {
    /// // connect via websocket
    /// let provider = Provider::connect_duplex("ws://localhost:8546").await?;
    /// // connects to a local IPC socket
    /// let provider = Provider::connect_duplex("ipc:///home/user/.ethereum/geth.ipc").await?;
    /// let provider = Provider::connect_duplex("/home/user/.ethereum/geth.ipc").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Fails, if the selected connection can not be established.
    ///
    /// # Panics
    ///
    /// Panics, if a connection is selected which has not been feature-enabled
    /// at compile time, e.g., if a HTTP url is given but the `http` cargo
    /// feature is not enabled.
    pub async fn connect_duplex(path: &str) -> Result<Self, Box<ConnectionError>> {
        let connection: Arc<dyn DuplexConnection> = if path.starts_with("http") {
            panic!("path starts with http/https, but http does not support duplex connections");
        } else if path.starts_with("ws") {
            #[cfg(feature = "ws")]
            {
                todo!("...")
            }
            #[cfg(not(feature = "ws"))]
            {
                panic!("path starts with ws/wss, but `ws` cargo feature is not enabled");
            }
        } else {
            #[cfg(feature = "ipc")]
            {
                // the path is allowed start with "ipc://"
                let ipc = connection::ipc::Ipc::connect(path.trim_start_matches("ipc://"))
                    .await
                    .map_err(ConnectionError::connection)?;
                Arc::new(ipc)
            }
            #[cfg(not(feature = "ipc"))]
            {
                todo!("ipc path detected, but `ipc` cargo feature is not enabled");
            }
        };

        Ok(Self { connection })
    }
}

impl<C: Connection> Provider<C> {
    /// Send a batch request and await its reponse.
    ///
    /// The order of responses always matches the order in which the input
    /// requests are given to this method.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example_batch() {
    /// # let provider = Provider { connection: Noop };
    /// # let address = ethrs::types::Address::zero();
    /// let r0 = provider.get_balance(&address, "latest".into());
    /// let r1 = provider.get_gas_price();
    /// // `send_balance` accepts tuples of heterogeneous RPC calls
    /// if let Ok((balance, gas_price)) = provider.send_batch((r0, r1)).await {
    ///     # drop(balance);
    ///     # drop(gas_price);
    ///     // ...
    /// }
    ///
    /// // ... or Vecs, slices, arrays of RPC calls with the same output type
    /// let r0 = provider.get_balance(&address, "earliest".into());
    /// let r1 = provider.get_balance(&address, "latest".into());
    /// let r2 = provider.get_balance(&address, "pending".into());
    ///
    /// if let Ok(balances) = provider.send_batch(vec![r0, r1, r2]).await {
    ///     # drop(balances);
    ///     // ...
    /// }
    /// # }
    /// ```
    pub async fn send_batch_request<B>(&self, batch: B) -> Result<B::Output, BatchError>
    where
        B: BatchCall,
    {
        batch.send_batch(&self.connection).await
    }

    /// Returns an object with data about the sync or
    /// [`Synced`](SyncStatus::Synced).
    ///
    /// ```ignore
    /// async fn get_syncing(&self) -> Result<SyncStatus, Box<ProviderError>>;
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// # use ethrs::{connection::noop::Noop, types::SyncStatus};
    /// use ethrs::Provider;
    ///
    /// # async fn example_syncing() {
    /// # let provider = Provider { connection: Noop };
    /// if let Ok(SyncStatus::Synced) = provider.get_syncing().await {
    ///     println!("node is synced");
    /// }
    /// # }
    /// ```
    pub fn get_syncing(&self) -> RpcCall<&C, SyncStatus> {
        self.prepare_rpc_call("eth_syncing", ())
    }

    /// Returns the client coinbase address.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_coinbase(&self) -> Result<Address, Box<ProviderError>>;
    /// ```
    pub fn get_coinbase(&self) -> RpcCall<&C, Address> {
        self.prepare_rpc_call("eth_coinbase", ())
    }

    /// Returns the current price per gas in wei.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_gas_price(&self) -> Result<U256, Box<ProviderError>>;
    /// ```
    pub fn get_gas_price(&self) -> RpcCall<&C, U256> {
        self.prepare_rpc_call("eth_gasPrice", ())
    }

    /// Returns a list of addresses owned by the client.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_accounts(&self) -> Result<Vec<Address>, Box<ProviderError>>;
    /// ```
    pub fn get_accounts(&self) -> RpcCall<&C, Box<[Address]>> {
        self.prepare_rpc_call("eth_getAccounts", ())
    }

    /// Returns `true` if the client is actively mining new blocks.
    ///
    /// The function signature is equivalent to
    ///
    /// ```ignore
    /// async fn get_mining(&self) -> Result<bool, Box<ProviderError>>;
    /// ```
    pub fn get_mining(&self) -> RpcCall<&C, bool> {
        self.prepare_rpc_call("eth_mining", ())
    }

    /// Returns the number of most recent block.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_block_number(&self) -> Result<u64, Box<ProviderError>>;
    /// ```
    pub fn get_block_number(&self) -> RpcCall<&C, u64> {
        self.prepare_rpc_call("eth_blockNumber", ())
    }

    /// Returns the balance of the account of given address.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_balance(
    ///     &self,
    ///     address: &Address,
    ///     block: &BlockNumber
    /// ) -> Result<U256, Box<ProviderError>>;
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// # let provider = Provider { connection: Noop };
    /// # async fn example_balance() {
    /// # let address = ethrs::types::Address::zero();
    /// // default block is "latest"
    /// if let Ok(balance) = provider.get_balance(&address, Default::default()).await {
    ///     println!("account balance is {balance:?}");
    /// }
    /// # }
    /// ```
    pub fn get_balance(&self, address: &Address, block: Option<BlockNumber>) -> RpcCall<&C, U256> {
        match block {
            Some(block) => self.prepare_rpc_call("eth_getBalance", (address, block)),
            None => self.prepare_rpc_call("eth_getBalance", [address]),
        }
    }

    /// Returns the value from a storage position at a given address.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_storage_at(
    ///     &self,
    ///     address: &Address,
    ///     pos: &U256,
    ///     block: BlockNumber,
    /// ) -> Result<U256, Box<ProviderError>>;
    /// ```
    pub fn get_storage_at(
        &self,
        address: &Address,
        pos: &U256,
        block: BlockNumber,
    ) -> RpcCall<&C, U256> {
        self.prepare_rpc_call("eth_getStorageAt", (address, pos, block))
    }

    /// Returns the number of transactions sent from an address.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_transaction_count(
    ///     &self,
    ///     address: &Address,
    ///     block: BlockNumber
    /// ) -> Result<U256, Box<ProviderError>>;
    /// ```
    pub fn get_transaction_count(
        &self,
        address: &Address,
        block: BlockNumber,
    ) -> RpcCall<&C, U256> {
        self.prepare_rpc_call("eth_getTransactionCount", (address, block))
    }

    /// Returns code at a given address.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_code(
    ///     &self,
    ///     address: &Address,
    /// ) -> Result<Bytes, Box<ProviderError>>;
    /// ```
    pub fn get_code(&self, address: &Address) -> RpcCall<&C, Bytes> {
        self.prepare_rpc_call("eth_getCode", [address])
    }

    /// Executes a new message call immidiately without creating a transaction
    /// on the block chain.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn call(
    ///     &self,
    ///     txn: &TransactionCall
    /// ) -> Result<Bytes, Box<ProviderError>>;
    /// ```
    pub fn call(&self, txn: &TransactionCall) -> RpcCall<&C, Bytes> {
        self.prepare_rpc_call("eth_call", [txn])
    }

    /// Signs the given `message` using the account at `address`.
    ///
    /// The sign method calculates an Ethereum specific signature with:
    /// `sign(keccak256("\x19Ethereum Signed Message:\n" + len(message) + message)))`.
    ///
    /// By adding a prefix to the message makes the calculated signature
    /// recognisable as an Ethereum specific signature.
    /// This prevents misuse where a malicious DApp can sign arbitrary data
    /// (e.g. transaction) and use the signature to impersonate the victim.
    ///
    /// **Note** the address to sign with must be unlocked.
    ///
    /// The function signature is equivalent to
    ///
    /// ```
    /// async fn sign(
    ///     &self,
    ///     address: &Address,
    ///     message: &Bytes,
    /// ) -> Result<Bytes, Box<ProviderError>>;
    /// ```
    pub fn sign(&self, address: &Address, message: &Bytes) -> RpcCall<&C, Bytes> {
        self.prepare_rpc_call("eth_sign", (address, message))
    }

    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn sign_transaction(
    ///     &self,
    ///     txn: &TransactionRequest
    /// ) -> Result<Bytes, Box<ProviderError>>;
    /// ```
    pub fn sign_transaction(&self, txn: &TransactionRequest) -> RpcCall<&C, Bytes> {
        self.prepare_rpc_call("eth_signTransaction", [txn])
    }

    /// Equivalent to:
    ///
    /// ```ignore
    /// pub async fn send_transaction(
    ///     &self,
    ///     txn: &TransactionRequest
    /// ) -> Result<H256, Box<ProviderError>>
    /// ```
    pub fn send_transaction(&self, txn: &TransactionRequest) -> RpcCall<&C, H256> {
        self.prepare_rpc_call("eth_sendTransaction", [txn])
    }

    /// Equivalent to:
    ///
    /// ```ignore
    /// pub async fn send_raw_transaction(
    ///     &self,
    ///     data: &Bytes
    /// ) -> Result<H256, Box<ProviderError>>
    /// ```
    pub fn send_raw_transaction(&self, data: Bytes) -> RpcCall<&C, H256> {
        self.prepare_rpc_call("eth_sendRawTransaction", [data])
    }

    /// Generates and returns an estimate of how much gas is necessary to allow
    /// the transaction to complete.
    ///
    /// The transaction will not be added to the blockchain.
    /// **Note** that the estimate may be significantly more than the amount of
    /// gas actually used by the transaction, for a variety of reasons including
    /// EVM mechanics and node performance.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// pub async fn estimate_gas(
    ///     &self,
    ///     txn: &ByTransactionCalltes
    /// ) -> Result<U256, Box<ProviderError>>
    /// ```
    pub fn estimate_gas(&self, txn: &TransactionCall) -> RpcCall<&C, U256> {
        self.prepare_rpc_call("eth_estimateGas", [txn])
    }

    /// Returns a collection of historical gas information from which you can
    /// decide what to submit as your `max_fee_per_gas` and `max_priority_fee_per_gas`.
    /// This method was introduced with [EIP-1559](https://blog.alchemy.com/blog/eip-1559).
    ///
    /// # Parameters
    ///
    /// - `block_count` - The numberof blocks in the requested range. Between 1
    ///   and 1024 blocks can be requested in a single query. Less than the
    ///   requested number may be returned if not all blocks are available.
    /// - `newest_block` - The highest block in the requested range.
    /// - `reward_percentiles` - (optional)
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// # use ethers_core::types::Address;
    /// # use ethers_connections::connections::noop;
    /// use ethers_connections::{Connection, Provider};
    ///
    /// # async fn examples_get_code() {
    /// # let build_provider = || Provider::new(Arc::new(noop::Noop)).into_dyn();
    /// let provider: Provider<Arc<dyn Connection>> = build_provider();
    /// let res = provider.fee_history(4, "latest".into(), Some(&[25, 75]).await;
    /// # assert!(res.is_err());
    /// # }
    /// ```
    pub fn fee_history(
        &self,
        block_count: u64,
        newest_block: BlockNumber,
        reward_percentiles: Option<&[u8]>,
    ) -> RpcCall<&C, FeeHistory> {
        match reward_percentiles {
            Some(reward_percentiles) => self.prepare_rpc_call(
                "eth_feeHistory",
                (block_count, newest_block, reward_percentiles),
            ),
            None => self.prepare_rpc_call("eth_feeHistory", (block_count, newest_block)),
        }
    }

    /// Returns the block with the given `hash` with only the hashes of all
    /// included transactions.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_block_by_hash(
    ///    &self,
    ///    hash: &H256,
    /// ) -> Result<Option<Block<H256>>, Box<ProviderError>>;
    /// ```
    pub fn get_block_by_hash(&self, hash: &H256) -> RpcCall<&C, Option<Block<H256>>> {
        self.prepare_rpc_call("eth_getBlockByHash", (hash, false))
    }

    /// Returns the block with the given `hash` with all included transactions.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_block_by_hash_with_txns(
    ///    &self,
    ///    hash: &H256,
    /// ) -> Result<Option<Block<Transaction>>, Box<ProviderError>>;
    /// ```
    pub fn get_block_by_hash_with_txns(
        &self,
        hash: &H256,
    ) -> RpcCall<&C, Option<Block<Transaction>>> {
        self.prepare_rpc_call("eth_getBlockByHash", (hash, true))
    }

    /// Returns the block with the given `block` number (or tag) with all
    /// included transactions.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_block_by_number(
    ///    &self,
    ///    block: BlockNumber,
    /// ) -> Result<Option<Block<H256>>, Box<ProviderError>>;
    /// ```
    pub fn get_block_by_number(&self, block: BlockNumber) -> RpcCall<&C, Option<Block<H256>>> {
        self.prepare_rpc_call("eth_getBlockByNumber", (block, false))
    }

    /// Returns the block with the given `block` number (or tag) with only the
    /// hashes of all included transactions.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_block_by_number_with_txns(
    ///    &self,
    ///    block: BlockNumber,
    /// ) -> Result<Option<Block<Transaction>>, Box<ProviderError>>;
    /// ```
    pub fn get_block_by_number_with_txns(
        &self,
        block: BlockNumber,
    ) -> RpcCall<&C, Option<Block<Transaction>>> {
        self.prepare_rpc_call("eth_getBlockByNumber", (block, true))
    }

    pub fn get_transaction_by_hash<'a>(
        &'a self,
        hash: &H256,
    ) -> RpcCall<&'a C, Option<Transaction>> {
        self.prepare_rpc_call("eth_getTransactionByHash", [hash])
    }

    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn get_transaction_by_block_hash_and_index(
    ///     &self,
    ///     hash: &H256,
    ///     index: u64,
    /// ) -> Result<Option<Transaction>, Box<ProviderError>>;
    /// ```
    pub fn get_transaction_by_block_hash_and_index(
        &self,
        hash: &H256,
        index: u64,
    ) -> RpcCall<&C, Option<Transaction>> {
        self.prepare_rpc_call("eth_getTransactionByBlockHashAndIndex", (hash, U64::from(index)))
    }

    pub fn get_transaction_by_block_number_and_index(
        &self,
        block: BlockNumber,
        index: u64,
    ) -> RpcCall<&C, Option<Transaction>> {
        self.prepare_rpc_call("eth_getTransactionByBlockNumberAndIndex", (block, U64::from(index)))
    }

    /// Returns the receipt of a transaction by transaction hash.
    ///
    /// **Note** That the receipt is not available for pending transactions.
    pub fn get_transaction_receipt(&self, hash: &H256) -> RpcCall<&C, Option<TransactionReceipt>> {
        self.prepare_rpc_call("eth_getTransactionReceipt", [hash])
    }

    /// Returns the number of uncles in a block from a block matching the given
    /// block `hash`.
    pub fn get_uncle_count_by_block_hash(&self, hash: &H256) -> RpcCall<&C, U256> {
        self.prepare_rpc_call("eth_getUncleCountByBlockHash", [hash])
    }

    /// Returns the number of uncles in a block from a block matching the given
    /// `block` number.
    pub fn get_uncle_count_by_block_number(&self, block: BlockNumber) -> RpcCall<&C, U256> {
        self.prepare_rpc_call("eth_getUncleCountByBlockNumber", [block])
    }

    /// Installs a new `filter` that can be polled for state changes (logs).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ethers_core::types::{Address, H256};
    /// use ethers_connections::types::Filter;
    ///
    /// # async fn example_filter() {
    /// # let provider = ethers_connections::Provider::noop();
    ///
    /// let filter = Filter::new()
    ///     .from_block("latest".into())
    ///     .to_block("pending".into())
    ///     .address(vec![Address::zero()])
    ///     .event("Transfer(uint256)")
    ///     .topic1(H256::zero().into())
    ///     .topic2([H256::zero, H256::zero()].into())
    ///     .topic3(vec![H256::zero(), H256::zero(), H256::zero()].into());
    ///
    /// if let Ok(id) = provider.new_filter(&filter).await {
    ///     println!("installed filter with ID {id}");
    /// }
    /// }
    /// ```
    pub fn install_log_filter(&self, filter: &Filter) -> RpcCall<&C, U256> {
        self.prepare_rpc_call("eth_newFilter", [filter])
    }

    /// Polls the installed log filter with `id` for all new logs matching the
    /// installed filter criteria since the last time it was last polled.
    pub fn get_log_filter_changes(&self, id: &U256) -> RpcCall<&C, Vec<Log>> {
        self.prepare_rpc_call("eth_getFilterChanges", [id])
    }

    /// Installs a new filter that can be polled for the hashes of newly
    /// arrived blocks.
    pub fn install_block_filter(&self) -> RpcCall<&C, H256> {
        self.prepare_rpc_call("eth_newBlockFilter", ())
    }

    /// Polls the installed block filter with `id` for all new block hashes
    /// since the last time it was last polled.
    pub fn get_block_filter_changes(&self, id: &U256) -> RpcCall<&C, Vec<H256>> {
        self.prepare_rpc_call("eth_getFilterChanges", [id])
    }

    /// Installs a new filter that can be polled for the hashes of newly
    /// arrived pending transactions.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn install_pending_transactions_filter(&self) -> Result<U256, Box<ProviderError>>;
    /// ```
    pub fn install_pending_transactions_filter(&self) -> RpcCall<&C, U256> {
        self.prepare_rpc_call("eth_newPendingTransactionsFilter", ())
    }

    /// Polls the installed block filter with `id` for all new pending
    /// transaction hashes since the last time it was last polled.
    ///
    /// The function signature is equivalent to
    ///
    /// ```ignore
    /// async fn get_pending_transactions_filter_changes(
    ///     &self,
    ///     id: &U256,
    /// ) -> Result<Vec<H256>, Box<ProviderError>>;
    /// ```
    pub fn get_pending_transactions_filter_changes(&self, id: &U256) -> RpcCall<&C, Vec<H256>> {
        self.prepare_rpc_call("eth_getFilterChanges", [id])
    }

    /// Uninstalls a filter with a given `id`.
    ///
    /// The function signature is equivalent to
    ///
    /// ```
    /// pub async fn uninstall_filter(&self, id: &U256) -> Result<bool, Box<ProviderError>>;
    /// ```
    pub fn uninstall_filter(&self, id: &U256) -> RpcCall<&C, bool> {
        self.prepare_rpc_call("eth_uninstallFilter", [id])
    }

    /// Prepares an [`RpcCall`] for the given parameters using this provider's
    /// [`Connection`].
    pub fn prepare_rpc_call<T, R>(&self, method: &'static str, params: T) -> RpcCall<&C, R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let id = self.connection.request_id();
        RpcCall::new(&self.connection, CallParams::new(id, method, params))
    }
}

impl<C: DuplexConnection + Clone> Provider<C> {
    /// Installs a subscription for new blocks.
    ///
    /// # Examples
    ///
    /// ```
    /// use ethers_connections::{Connection, Provider};
    /// # use ethers_connections::connections::noop;
    ///
    /// # async fn example_new_heads() -> Result<(), Box<dyn std::error::Error>> {
    /// # let provider = Provider::new(noop::Noop);
    /// // let provider = ...;
    /// let mut stream = provider.subscribe_blocks().await?;
    /// while let Some(_) = stream.recv().await {
    ///     println!("new block received");
    /// }
    ///
    /// // subscription must be explicitly unsubscribed from.
    /// stream.unsubscribe().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_blocks(
        &self,
    ) -> Result<SubscriptionStream<Block<H256>, C>, Box<ProviderError>> {
        self.subscribe(["newHeads"]).await
    }

    /// Installs a subscription for new pending transaction hashes.
    pub async fn subscribe_pending_transactions(
        &self,
    ) -> Result<SubscriptionStream<H256, C>, Box<ProviderError>> {
        self.subscribe(["pendingTransactions"]).await
    }

    /// Installs a subscription with the given `params`.
    pub async fn subscribe<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        params: T,
    ) -> Result<SubscriptionStream<R, C>, Box<ProviderError>> {
        let provider = self.clone();
        let id: U256 = provider.prepare_rpc_call("eth_subscribe", params).await?;
        let rx = provider
            .connection
            .subscribe(id)
            .await
            .map_err(|err| err.to_provider_err())?
            .expect("invalid subscription id");

        Ok(SubscriptionStream::new(id, provider, rx))
    }
}

impl<C: DuplexConnection> Provider<C> {
    /// Unsubscribes from the subscription with the given `id`.
    pub async fn unsubscribe(&self, id: U256) -> Result<bool, Box<ProviderError>> {
        let ok: bool = self.prepare_rpc_call("eth_unsubscribe", [id]).await?;
        self.connection.unsubscribe(id).map_err(|err| err.to_provider_err())?;
        Ok(ok)
    }
}

/// An error that occurred during the operation of a [`Provider`].
#[derive(Debug)]
pub struct ProviderError {
    /// The kind of the occurred error.
    pub kind: ErrorKind,
    /// The additional context to the error (empty string if none).
    pub(crate) context: Cow<'static, str>,
}

impl ProviderError {
    /// Returns the error's additional context.
    pub fn context(&self) -> Option<&str> {
        if self.context.is_empty() {
            None
        } else {
            Some(self.context.as_ref())
        }
    }

    pub fn is_insufficient_funds(&self) -> bool {
        self.as_jsonrpc().map(|err| err.message.contains("insufficient funds")).unwrap_or(false)
    }

    pub fn is_nonce_too_low(&self) -> bool {
        self.as_jsonrpc().map(|err| &*err.message == "nonce too low").unwrap_or(false)
    }

    pub fn is_replacement_underpriced(&self) -> bool {
        self.as_jsonrpc()
            .map(|err| err.message.contains("replacement transaction underpriced"))
            .unwrap_or(false)
    }

    pub fn as_jsonrpc(&self) -> Option<&rpc::JsonRpcError> {
        match &self.kind {
            ErrorKind::Connection(err) => match err {
                ConnectionError::JsonRpc(err) => Some(err),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn with_ctx(
        mut self: Box<Self>,
        context: impl Into<Cow<'static, str>>,
    ) -> Box<Self> {
        self.context = context.into();
        self
    }

    pub(crate) fn json(err: serde_json::Error) -> Box<Self> {
        Box::new(Self { kind: ErrorKind::Json(err), context: "".into() })
    }
}

impl error::Error for ProviderError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Json(err) => Some(err),
            ErrorKind::Connection(err) => Some(&*err),
        }
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = &self.kind;
        match self.context() {
            Some(ctx) => write!(f, "{ctx}: {kind}"),
            None => write!(f, "{kind}"),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    /// The error returned when parsing the raw response into the expected type
    /// fails.
    Json(serde_json::Error),
    Connection(ConnectionError),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(err) => write!(f, "failed to parse JSON response to expected type: {err}"),
            Self::Connection(err) => write!(f, "{err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ethers_core::types::Address;

    use crate::{connection::noop, Connection, DuplexConnection, Provider};

    #[test]
    fn object_safety() {
        crate::block_on(async move {
            let provider = Provider::new(noop::Noop);
            let res = provider.get_block_number().await;
            assert!(res.is_err());

            // dyn Connection
            let provider: Provider<Arc<dyn Connection>> = Provider::new(Arc::new(noop::Noop));
            let res = provider.get_block_number().await;
            assert!(res.is_err());

            // dyn DuplexConnection
            let provider: Provider<Arc<dyn DuplexConnection>> = Provider::new(Arc::new(noop::Noop));
            let res = provider.get_block_number().await;
            assert!(res.is_err());

            let res = provider.subscribe_blocks().await;
            assert!(res.is_err());
        });
    }

    #[test]
    fn block_number() {
        crate::block_on(async {
            let provider = Provider::new(noop::Noop);
            let address = Address::zero();

            let _ = provider.get_balance(&address, None).await;
            let _ = provider.get_balance(&address, Some("earliest".into())).await;
            let _ = provider.get_balance(&address, Some("latest".into())).await;
            let _ = provider.get_balance(&address, Some("pending".into())).await;
            let _ = provider.get_balance(&address, Some(0xcafe.into())).await;
        });
    }
}
