use std::{convert::TryFrom, fmt::Debug, time::Duration};

use async_trait::async_trait;
use eyre::Result;
use hex::FromHex;
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use thiserror::Error;
use url::{ParseError, Url};
use std::fmt;

use ethers_core::{
    abi::{self, Detokenize, ParamType},
    types::{
        Address,
        Block, BlockId, BlockNumber, BlockTrace, Bytes, Filter, H256, Log, NameOrAddress,
        Selector, Signature, Trace, TraceFilter, TraceType, transaction::{eip2718::TypedTransaction, eip2930::AccessListWithGasUsed}, Transaction,
        TransactionReceipt, TxHash, TxpoolContent, TxpoolInspect, TxpoolStatus, U256, U64,
    },
    utils,
};

use crate::{
    ens,
    FeeHistory, PendingTransaction, pubsub::{PubsubClient, SubscriptionStream}, stream::{DEFAULT_POLL_INTERVAL, FilterWatcher},
};
use futures_core::Future;
use std::sync::atomic::{AtomicU64, Ordering};


#[async_trait]
pub trait JsonRpcProvider: Send + Sync {

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R>
        where
            Self: Sized,
            T: Debug + Serialize + Send + Sync,
            R: Serialize + DeserializeOwned {
        todo!()
    }

    fn default_sender(&self) -> Option<Address> {None}

    ////// Blockchain Status
    //
    // Functions for querying the state of the blockchain

    /// Returns the current client version using the `web3_clientVersion` RPC.
    async fn client_version(&self) -> Result<String> where Self: Sized {
        self.request("web3_clientVersion", ()).await
    }

    /// Helper for filling a transaction
    async fn fill_transaction(
        &self,
        tx: &mut TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<()>  where Self: Sized {
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

                let gas = maybe(inner.gas, self.estimate_gas(&tx_clone)).await?;
                inner.gas = Some(gas);

                if inner.max_fee_per_gas.is_none() || inner.max_priority_fee_per_gas.is_none() {
                    let (max_fee_per_gas, max_priority_fee_per_gas) =
                        self.estimate_eip1559_fees(None).await?;
                    if inner.max_fee_per_gas.is_none() {
                        inner.max_fee_per_gas = Some(max_fee_per_gas);
                    }
                    if inner.max_priority_fee_per_gas.is_none() {
                        inner.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
                    }
                }
            }
        };

        Ok(())
    }

    /// Gets the latest block number via the `eth_BlockNumber` API
    async fn get_block_number(&self) -> Result<U64> where Self: Sized {
        self.request("eth_blockNumber", ()).await
    }

    /// Gets the block at `block_hash_or_number` (transaction hashes only)
    async fn get_block<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<TxHash>>> where Self: Sized {
        self.get_block_gen(block_hash_or_number.into(), false).await
    }

    /// Gets the block at `block_hash_or_number` (full transactions included)
    async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<Transaction>>> where Self: Sized {
        self.get_block_gen(block_hash_or_number.into(), true).await
    }

    /// Gets the block uncle count at `block_hash_or_number`
    async fn get_uncle_count<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<U256> where Self: Sized {
        let id = block_hash_or_number.into();
        Ok(match id {
            BlockId::Hash(hash) => {
                let hash = utils::serialize(&hash);
                self.request("eth_getUncleCountByBlockHash", [hash]).await?
            }
            BlockId::Number(num) => {
                let num = utils::serialize(&num);
                self.request("eth_getUncleCountByBlockNumber", [num])
                    .await?
            }
        })
    }

    /// Gets the block uncle at `block_hash_or_number` and `idx`
    async fn get_uncle<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
        idx: U64,
    ) -> Result<Option<Block<H256>>> where Self: Sized {
        let blk_id = block_hash_or_number.into();
        let idx = utils::serialize(&idx);
        Ok(match blk_id {
            BlockId::Hash(hash) => {
                let hash = utils::serialize(&hash);
                self.request("eth_getUncleByBlockHashAndIndex", [hash, idx])
                    .await?
            }
            BlockId::Number(num) => {
                let num = utils::serialize(&num);
                self.request("eth_getUncleByBlockNumberAndIndex", [num, idx])
                    .await?
            }
        })
    }

    /// Gets the transaction with `transaction_hash`
    async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<Transaction>> where Self: Sized {
        let hash = transaction_hash.into();
        self.request("eth_getTransactionByHash", [hash]).await
    }

    /// Gets the transaction receipt with `transaction_hash`
    async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<TransactionReceipt>> where Self: Sized {
        let hash = transaction_hash.into();
        self.request("eth_getTransactionReceipt", [hash]).await
    }

    /// Returns all receipts for a block.
    ///
    /// Note that this uses the `eth_getBlockReceipts` RPC, which is
    /// non-standard and currently supported by Erigon.
    async fn get_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>> where Self: Sized {
        self.request("eth_getBlockReceipts", [block.into()]).await
    }

    /// Gets the current gas price as estimated by the node
    async fn get_gas_price(&self) -> Result<U256> where Self: Sized {
        self.request("eth_gasPrice", ()).await
    }

    /// Gets a heuristic recommendation of max fee per gas and max priority fee per gas for
    /// EIP-1559 compatible transactions.
    async fn estimate_eip1559_fees(
        &self,
        estimator: Option<fn(U256, Vec<Vec<U256>>) -> (U256, U256)>,
    ) -> Result<(U256, U256)> where Self: Sized {
        let base_fee_per_gas = self
            .get_block(BlockNumber::Latest)
            .await?
            .ok_or_else(|| ProviderError::CustomError("Latest block not found".into()))?
            .base_fee_per_gas
            .ok_or_else(|| ProviderError::CustomError("EIP-1559 not activated".into()))?;

        let fee_history = self
            .fee_history(
                utils::EIP1559_FEE_ESTIMATION_PAST_BLOCKS,
                BlockNumber::Latest,
                &[utils::EIP1559_FEE_ESTIMATION_REWARD_PERCENTILE],
            )
            .await?;

        // use the provided fee estimator function, or fallback to the default implementation.
        let (max_fee_per_gas, max_priority_fee_per_gas) = if let Some(es) = estimator {
            es(base_fee_per_gas, fee_history.reward)
        } else {
            utils::eip1559_default_estimator(base_fee_per_gas, fee_history.reward)
        };

        Ok((max_fee_per_gas, max_priority_fee_per_gas))
    }

    /// Gets the accounts on the node
    async fn get_accounts(&self) -> Result<Vec<Address>> where Self: Sized {
        self.request("eth_accounts", ()).await
    }

    /// Returns the nonce of the address
    async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256> where Self: Sized {
        let from = match from.into() {
            NameOrAddress::Name(ens_name) => self.resolve_name(&ens_name).await?,
            NameOrAddress::Address(addr) => addr,
        };

        let from = utils::serialize(&from);
        let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));
        self.request("eth_getTransactionCount", [from, block]).await
    }

    /// Returns the account's balance
    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256> where Self: Sized {
        let from = match from.into() {
            NameOrAddress::Name(ens_name) => self.resolve_name(&ens_name).await?,
            NameOrAddress::Address(addr) => addr,
        };

        let from = utils::serialize(&from);
        let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));
        self.request("eth_getBalance", [from, block]).await
    }

    /// Returns the currently configured chain id, a value used in replay-protected
    /// transaction signing as introduced by EIP-155.
    async fn get_chainid(&self) -> Result<U256> where Self: Sized {
        self.request("eth_chainId", ()).await
    }

    ////// Contract Execution
    //
    // These are relatively low-level calls. The Contracts API should usually be used instead.

    /// Sends the read-only (constant) transaction to a single Ethereum node and return the result (as bytes) of executing it.
    /// This is free, since it does not change any state on the blockchain.
    async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<Bytes> where Self: Sized {
        let tx = utils::serialize(tx);
        let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));
        self.request("eth_call", [tx, block]).await
    }

    /// Sends a transaction to a single Ethereum node and return the estimated amount of gas required (as a U256) to send it
    /// This is free, but only an estimate. Providing too little gas will result in a transaction being rejected
    /// (while still consuming all provided gas).
    async fn estimate_gas(&self, tx: &TypedTransaction) -> Result<U256> where Self: Sized {
        self.request("eth_estimateGas", [tx]).await
    }

    async fn create_access_list(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<AccessListWithGasUsed> where Self: Sized {
        let tx = utils::serialize(tx);
        let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));
        self.request("eth_createAccessList", [tx, block]).await
    }

    /// Sends the transaction to the entire Ethereum network and returns the transaction's hash
    /// This will consume gas from the account that signed the transaction.
    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self>> where Self: Sized {
        // let mut tx = tx.into();
        // self.fill_transaction(&mut tx, block).await?;
        // let tx_hash = self.request("eth_sendTransaction", [tx]).await?;
        //
        // Ok(PendingTransaction::new(tx_hash, self).interval(self.get_interval()))
        todo!()
    }

    /// Send the raw RLP encoded transaction to the entire Ethereum network and returns the transaction's hash
    /// This will consume gas from the account that signed the transaction.
    async fn send_raw_transaction<'a>(
        &'a self,
        tx: Bytes,
    ) -> Result<PendingTransaction<'a, Self>>  where Self: Sized {
        // let rlp = utils::serialize(&tx);
        // let tx_hash = self.request("eth_sendRawTransaction", [rlp]).await?;
        // Ok(PendingTransaction::new(tx_hash, self).interval(self.get_interval()))
        todo!()
    }

    /// The JSON-RPC provider is at the bottom-most position in the middleware stack. Here we check
    /// if it has the key for the sender address unlocked, as well as supports the `eth_sign` call.
    async fn is_signer(&self) -> bool where Self: Sized {
        // match self.from {
        //     Some(sender) => self.sign(vec![], &sender).await.is_ok(),
        //     None => false,
        // }
        todo!()
    }

    /// Signs data using a specific account. This account needs to be unlocked.
    async fn sign<T: Into<Bytes> + Send + Sync>(
        &self,
        data: T,
        from: &Address,
    ) -> Result<Signature> where Self: Sized {
        let data = utils::serialize(&data.into());
        let from = utils::serialize(from);

        // get the response from `eth_sign` call and trim the 0x-prefix if present.
        let sig: String = self.request("eth_sign", [from, data]).await?;
        let sig = sig.strip_prefix("0x").unwrap_or(&sig);

        // decode the signature.
        let sig = hex::decode(sig)?;
        Ok(Signature::try_from(sig.as_slice())
            .map_err(|e| ProviderError::CustomError(e.to_string()))?)
    }

    ////// Contract state

    /// Returns an array (possibly empty) of logs that match the filter
    async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>> where Self: Sized {
        self.request("eth_getLogs", [filter]).await
    }

    // /// Streams matching filter logs
    // async fn watch<'a>(
    //     &'a self,
    //     filter: &Filter,
    // ) -> Result<FilterWatcher<'a, P, Log>> where Self: Sized {
        // let id = self.new_filter(FilterKind::Logs(filter)).await?;
        // let filter = FilterWatcher::new(id, self).interval(self.get_interval());
        // Ok(filter)
    // }
    //
    // /// Streams new block hashes
    // async fn watch_blocks(&self) -> Result<FilterWatcher<'_, P, H256>> where Self: Sized {
    //     let id = self.new_filter(FilterKind::NewBlocks).await?;
    //     let filter = FilterWatcher::new(id, self).interval(self.get_interval());
    //     Ok(filter)
    // }
    //
    // /// Streams pending transactions
    // async fn watch_pending_transactions(
    //     &self,
    // ) -> Result<FilterWatcher<'_, P, H256>> where Self: Sized {
    //     let id = self.new_filter(FilterKind::PendingTransactions).await?;
    //     let filter = FilterWatcher::new(id, self).interval(self.get_interval());
    //     Ok(filter)
    // }

    /// Creates a filter object, based on filter options, to notify when the state changes (logs).
    /// To check if the state has changed, call `get_filter_changes` with the filter id.
    async fn new_filter(&self, filter: FilterKind<'_>) -> Result<U256> where Self: Sized {
        let (method, args) = match filter {
            FilterKind::NewBlocks => ("eth_newBlockFilter", vec![]),
            FilterKind::PendingTransactions => ("eth_newPendingTransactionFilter", vec![]),
            FilterKind::Logs(filter) => ("eth_newFilter", vec![utils::serialize(&filter)]),
        };

        self.request(method, args).await
    }

    /// Uninstalls a filter
    async fn uninstall_filter<T: Into<U256> + Send + Sync>(
        &self,
        id: T,
    ) -> Result<bool> where Self: Sized  {
        let id = utils::serialize(&id.into());
        self.request("eth_uninstallFilter", [id]).await
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
    async fn get_filter_changes<T, R>(&self, id: T) -> Result<Vec<R>>
        where
            Self: Sized,
            T: Into<U256> + Send + Sync,
            R: Serialize + DeserializeOwned + Send + Sync + Debug,
    {
        let id = utils::serialize(&id.into());
        self.request("eth_getFilterChanges", [id]).await
    }

    /// Get the storage of an address for a particular slot location
    async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: H256,
        block: Option<BlockId>,
    ) -> Result<H256> where Self: Sized  {
        let from = match from.into() {
            NameOrAddress::Name(ens_name) => self.resolve_name(&ens_name).await?,
            NameOrAddress::Address(addr) => addr,
        };

        let from = utils::serialize(&from);
        let location = utils::serialize(&location);
        let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));

        // get the hex encoded value.
        let value: String = self
            .request("eth_getStorageAt", [from, location, block])
            .await?;
        // get rid of the 0x prefix and left pad it with zeroes.
        let value = format!("{:0>64}", value.replace("0x", ""));
        Ok(H256::from_slice(&Vec::from_hex(value)?))
    }

    /// Returns the deployed code at a given address
    async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<BlockId>,
    ) -> Result<Bytes> where Self: Sized  {
        let at = match at.into() {
            NameOrAddress::Name(ens_name) => self.resolve_name(&ens_name).await?,
            NameOrAddress::Address(addr) => addr,
        };

        let at = utils::serialize(&at);
        let block = utils::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));
        self.request("eth_getCode", [at, block]).await
    }

    ////// Ethereum Naming Service
    // The Ethereum Naming Service (ENS) allows easy to remember and use names to
    // be assigned to Ethereum addresses. Any provider operation which takes an address
    // may also take an ENS name.
    //
    // ENS also provides the ability for a reverse lookup, which determines the name for an address if it has been configured.

    /// Returns the address that the `ens_name` resolves to (or None if not configured).
    ///
    /// # Panics
    ///
    /// If the bytes returned from the ENS registrar/resolver cannot be interpreted as
    /// an address. This should theoretically never happen.
    async fn resolve_name(&self, ens_name: &str) -> Result<Address> where Self: Sized  {
        self.query_resolver(ParamType::Address, ens_name, ens::ADDR_SELECTOR)
            .await
    }

    /// Returns the ENS name the `address` resolves to (or None if not configured).
    /// # Panics
    ///
    /// If the bytes returned from the ENS registrar/resolver cannot be interpreted as
    /// a string. This should theoretically never happen.
    async fn lookup_address(&self, address: Address) -> Result<String> where Self: Sized  {
        let ens_name = ens::reverse_address(address);
        self.query_resolver(ParamType::String, &ens_name, ens::NAME_SELECTOR)
            .await
    }

    /// Returns the details of all transactions currently pending for inclusion in the next
    /// block(s), as well as the ones that are being scheduled for future execution only.
    /// Ref: [Here](https://geth.ethereum.org/docs/rpc/ns-txpool#txpool_content)
    async fn txpool_content(&self) -> Result<TxpoolContent> where Self: Sized  {
        self.request("txpool_content", ()).await
    }

    /// Returns a summary of all the transactions currently pending for inclusion in the next
    /// block(s), as well as the ones that are being scheduled for future execution only.
    /// Ref: [Here](https://geth.ethereum.org/docs/rpc/ns-txpool#txpool_inspect)
    async fn txpool_inspect(&self) -> Result<TxpoolInspect> where Self: Sized  {
        self.request("txpool_inspect", ()).await
    }

    /// Returns the number of transactions currently pending for inclusion in the next block(s), as
    /// well as the ones that are being scheduled for future execution only.
    /// Ref: [Here](https://geth.ethereum.org/docs/rpc/ns-txpool#txpool_status)
    async fn txpool_status(&self) -> Result<TxpoolStatus> where Self: Sized  {
        self.request("txpool_status", ()).await
    }

    /// Executes the given call and returns a number of possible traces for it
    async fn trace_call<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        req: T,
        trace_type: Vec<TraceType>,
        block: Option<BlockNumber>,
    ) -> Result<BlockTrace> where Self: Sized  {
        let req = req.into();
        let req = utils::serialize(&req);
        let block = utils::serialize(&block.unwrap_or(BlockNumber::Latest));
        let trace_type = utils::serialize(&trace_type);
        self.request("trace_call", [req, trace_type, block]).await
    }

    /// Traces a call to `eth_sendRawTransaction` without making the call, returning the traces
    async fn trace_raw_transaction(
        &self,
        data: Bytes,
        trace_type: Vec<TraceType>,
    ) -> Result<BlockTrace> where Self: Sized  {
        let data = utils::serialize(&data);
        let trace_type = utils::serialize(&trace_type);
        self.request("trace_rawTransaction", [data, trace_type])
            .await
    }

    /// Replays a transaction, returning the traces
    async fn trace_replay_transaction(
        &self,
        hash: H256,
        trace_type: Vec<TraceType>,
    ) -> Result<BlockTrace> where Self: Sized  {
        let hash = utils::serialize(&hash);
        let trace_type = utils::serialize(&trace_type);
        self.request("trace_replayTransaction", [hash, trace_type])
            .await
    }

    /// Replays all transactions in a block returning the requested traces for each transaction
    async fn trace_replay_block_transactions(
        &self,
        block: BlockNumber,
        trace_type: Vec<TraceType>,
    ) -> Result<Vec<BlockTrace>> where Self: Sized  {
        let block = utils::serialize(&block);
        let trace_type = utils::serialize(&trace_type);
        self.request("trace_replayBlockTransactions", [block, trace_type])
            .await
    }

    /// Returns traces created at given block
    async fn trace_block(&self, block: BlockNumber) -> Result<Vec<Trace>> where Self: Sized {
        let block = utils::serialize(&block);
        self.request("trace_block", [block]).await
    }

    /// Return traces matching the given filter
    async fn trace_filter(&self, filter: TraceFilter) -> Result<Vec<Trace>> where Self: Sized  {
        let filter = utils::serialize(&filter);
        self.request("trace_filter", vec![filter]).await
    }

    /// Returns trace at the given position
    async fn trace_get<T: Into<U64> + Send + Sync>(
        &self,
        hash: H256,
        index: Vec<T>,
    ) -> Result<Trace> where Self: Sized  {
        let hash = utils::serialize(&hash);
        let index: Vec<U64> = index.into_iter().map(|i| i.into()).collect();
        let index = utils::serialize(&index);
        self.request("trace_get", vec![hash, index]).await
    }

    /// Returns all traces of a given transaction
    async fn trace_transaction(&self, hash: H256) -> Result<Vec<Trace>> where Self: Sized  {
        let hash = utils::serialize(&hash);
        self.request("trace_transaction", vec![hash]).await
    }

    /// Returns all receipts for that block. Must be done on a parity node.
    async fn parity_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>> where Self: Sized  {
        self.request("parity_getBlockReceipts", vec![block.into()])
            .await
    }

    // async fn subscribe<T, R>(
    //     &self,
    //     params: T,
    // ) -> Result<SubscriptionStream<'_, P, R>>
    //     where
    //         Self: Sized,
    //         T: Debug + Serialize + Send + Sync,
    //         R: DeserializeOwned + Send + Sync,
    //         P: PubsubClient,
    // {
    //     let id: U256 = self.request("eth_subscribe", params).await?;
    //     SubscriptionStream::new(id, self).map_err(Into::into)
    // }
    //
    // async fn unsubscribe<T>(&self, id: T) -> Result<bool>
    //     where
    //         Self: Sized,
    //         T: Into<U256> + Send + Sync,
    //         P: PubsubClient,
    // {
    //     self.request("eth_unsubscribe", [id.into()]).await
    // }
    //
    // async fn subscribe_blocks(
    //     &self,
    // ) -> Result<SubscriptionStream<'_, P, Block<TxHash>>>
    //     where
    //         Self: Sized,
    //         P: PubsubClient,
    // {
    //     self.subscribe(["newHeads"]).await
    // }
    //
    // async fn subscribe_pending_txs(
    //     &self,
    // ) -> Result<SubscriptionStream<'_, P, TxHash>>
    //     where
    //         Self: Sized,
    //         P: PubsubClient,
    // {
    //     self.subscribe(["newPendingTransactions"]).await
    // }
    //
    // async fn subscribe_logs<'a>(
    //     &'a self,
    //     filter: &Filter,
    // ) -> Result<SubscriptionStream<'a, P, Log>>
    //     where
    //         Self: Sized,
    //         P: PubsubClient,
    // {
    //     let logs = utils::serialize(&"logs"); // TODO: Make this a static
    //     let filter = utils::serialize(filter);
    //     self.subscribe([logs, filter]).await
    // }

    async fn fee_history<T: Into<U256> + serde::Serialize + Send + Sync>(
        &self,
        block_count: T,
        last_block: BlockNumber,
        reward_percentiles: &[f64],
    ) -> Result<FeeHistory> where Self: Sized  {
        let last_block = utils::serialize(&last_block);
        let reward_percentiles = utils::serialize(&reward_percentiles);

        // The blockCount param is expected to be an unsigned integer up to geth v1.10.6.
        // Geth v1.10.7 onwards, this has been updated to a hex encoded form. Failure to
        // decode the param from client side would fallback to the old API spec.
        self.request(
            "eth_feeHistory",
            [
                utils::serialize(&block_count),
                last_block.clone(),
                reward_percentiles.clone(),
            ],
        )
            .await
            .or(self
                .request(
                    "eth_feeHistory",
                    [
                        utils::serialize(&block_count.into().as_u64()),
                        last_block,
                        reward_percentiles,
                    ],
                )
                .await)
    }

    async fn query_resolver<T: Detokenize>(
        &self,
        param: ParamType,
        ens_name: &str,
        selector: Selector,
    ) -> Result<T> where Self: Sized {
        todo!()
        // // Get the ENS address, prioritize the local override variable
        // let ens_addr = self.ens.unwrap_or(ens::ENS_ADDRESS);
        //
        // // first get the resolver responsible for this name
        // // the call will return a Bytes array which we convert to an address
        // let data = self
        //     .call(&ens::get_resolver(ens_addr, ens_name).into(), None)
        //     .await?;
        //
        // let resolver_address: Address = decode_bytes(ParamType::Address, data);
        // if resolver_address == Address::zero() {
        //     return Err(ProviderError::EnsError(ens_name.to_owned()))?;
        // }
        //
        // // resolve
        // let data = self
        //     .call(
        //         &ens::resolve(resolver_address, selector, ens_name).into(),
        //         None,
        //     )
        //     .await?;
        //
        // Ok(decode_bytes(param, data))
    }

    async fn get_block_gen<Tx: Default + Serialize + DeserializeOwned + Debug>(
        &self,
        id: BlockId,
        include_txs: bool,
    ) -> Result<Option<Block<Tx>>>  where Self: Sized {
        let include_txs = utils::serialize(&include_txs);

        Ok(match id {
            BlockId::Hash(hash) => {
                let hash = utils::serialize(&hash);
                self.request("eth_getBlockByHash", [hash, include_txs])
                    .await?
            }
            BlockId::Number(num) => {
                let num = utils::serialize(&num);
                self.request("eth_getBlockByNumber", [num, include_txs])
                    .await?
            }
        })
    }
}


#[async_trait]
impl JsonRpcProvider for Box<dyn JsonRpcProvider> {
    fn default_sender(&self) -> Option<Address> {
        self.as_ref().default_sender()
    }
}

#[async_trait]
pub trait PubsubProvider :  JsonRpcProvider {

    // TODO
}

#[derive(Debug)]
pub struct HttpProvider {
    id: AtomicU64,
    client: reqwest::Client,
    url: Url,
}

#[async_trait]
impl JsonRpcProvider for HttpProvider {

    /// Sends a POST request with the provided method and the params serialized as JSON
    /// over HTTP
    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R> {
        let next_id = self.id.load(Ordering::SeqCst) + 1;
        self.id.store(next_id, Ordering::SeqCst);

        let payload = Request::new(next_id, method, params);

        let res = self
            .client
            .post(self.url.as_ref())
            .json(&payload)
            .send()
            .await?;
        let text = res.text().await?;
        let res: Response<R> =
            serde_json::from_str(&text)?;

        Ok(res.data.into_result()?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Error)]
/// A JSON-RPC 2.0 error
pub struct JsonRpcError {
    /// The error code
    pub code: i64,
    /// The error message
    pub message: String,
    /// Additional data
    pub data: Option<serde_json::Value>,
}

impl fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(code: {}, message: {}, data: {:?})",
            self.code, self.message, self.data
        )
    }
}

fn is_zst<T>(_t: &T) -> bool {
    std::mem::size_of::<T>() == 0
}

#[derive(Serialize, Deserialize, Debug)]
/// A JSON-RPC request
pub struct Request<'a, T> {
    id: u64,
    jsonrpc: &'a str,
    method: &'a str,
    #[serde(skip_serializing_if = "is_zst")]
    params: T,
}

impl<'a, T> Request<'a, T> {
    /// Creates a new JSON RPC request
    pub fn new(id: u64, method: &'a str, params: T) -> Self {
        Self {
            id,
            jsonrpc: "2.0",
            method,
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response<T> {
    pub(crate) id: u64,
    jsonrpc: String,
    #[serde(flatten)]
    pub data: ResponseData<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ResponseData<R> {
    Error { error: JsonRpcError },
    Success { result: R },
}

impl<R> ResponseData<R> {
    /// Consume response and return value
    pub fn into_result(self) -> Result<R> {
        match self {
            ResponseData::Success { result } => Ok(result),
            ResponseData::Error { error } => Err(error)?,
        }
    }
}


pub struct SignerMiddleware<Signer, Provider> {
    pub(crate) inner: Provider,
    pub(crate) signer: Signer,
    pub(crate) address: Address,
}

#[async_trait]
impl<Signer: Send + Sync, Provider : JsonRpcProvider> JsonRpcProvider for SignerMiddleware<Signer, Provider>
{
    fn default_sender(&self) -> Option<Address> {
        Some(self.address)
    }

}

#[derive(Debug, Error)]
/// An error thrown when making a call to the provider
pub enum ProviderError {
    /// An internal error in the JSON RPC Client
    #[error(transparent)]
    JsonRpcClientError(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// An error during ENS name resolution
    #[error("ens name not found: {0}")]
    EnsError(String),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    HexError(#[from] hex::FromHexError),

    #[error("custom error: {0}")]
    CustomError(String),
}


/// Types of filters supported by the JSON-RPC.
#[derive(Clone, Debug)]
pub enum FilterKind<'a> {
    /// `eth_newBlockFilter`
    Logs(&'a Filter),

    /// `eth_newBlockFilter` filter
    NewBlocks,

    /// `eth_newPendingTransactionFilter` filter
    PendingTransactions,
}

/// infallible conversion of Bytes to Address/String
///
/// # Panics
///
/// If the provided bytes were not an interpretation of an address
fn decode_bytes<T: Detokenize>(param: ParamType, bytes: Bytes) -> T {
    let tokens = abi::decode(&[param], bytes.as_ref())
        .expect("could not abi-decode bytes to address tokens");
    T::from_tokens(tokens).expect("could not parse tokens as address")
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

#[cfg(test)]
mod tests {
    use super::*;


    // Just some random test to ensure it compiles
    #[tokio::test]
    async fn signer_provider() {

        async fn create_signer(http: HttpProvider) {
            let signer = SignerMiddleware { inner: http, signer: (), address: Default::default() };
            signer.get_block_number().await.unwrap();

            // can nest
            let signer: SignerMiddleware<(), Box<dyn JsonRpcProvider>> = SignerMiddleware { inner: Box::new(signer), signer: (), address: Default::default() };

            signer.get_block_number().await.unwrap();
        }
    }
}