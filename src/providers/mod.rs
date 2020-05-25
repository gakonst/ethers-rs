//! Ethereum compatible providers
//! Currently supported:
//! - Raw HTTP POST requests
//!
//! TODO: WebSockets, multiple backends, popular APIs etc.
mod http;

use crate::{
    signers::{Client, Signer},
    types::{
        Address, Block, BlockId, BlockNumber, Filter, Log, Transaction, TransactionReceipt,
        TransactionRequest, TxHash, U256,
    },
    utils,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Debug};

/// An HTTP provider for interacting with an Ethereum-compatible blockchain
pub type HttpProvider = Provider<http::Provider>;

#[async_trait]
/// Implement this trait in order to plug in different backends
pub trait JsonRpcClient: Debug {
    type Error: Error;

    /// Sends a request with the provided method and the params serialized as JSON
    async fn request<T: Serialize + Send + Sync, R: for<'a> Deserialize<'a>>(
        &self,
        method: &str,
        params: Option<T>,
    ) -> Result<R, Self::Error>;
}

/// An abstract provider for interacting with the [Ethereum JSON RPC
/// API](https://github.com/ethereum/wiki/wiki/JSON-RPC)
#[derive(Clone, Debug)]
pub struct Provider<P>(P);

// JSON RPC bindings
impl<P: JsonRpcClient> Provider<P> {
    /// Connects to a signer and returns a client
    pub fn connect<S: Signer>(&self, signer: S) -> Client<S, P> {
        Client {
            signer: Some(signer),
            provider: self,
        }
    }

    // Cost related

    /// Gets the current gas price as estimated by the node
    pub async fn get_gas_price(&self) -> Result<U256, P::Error> {
        self.0.request("eth_gasPrice", None::<()>).await
    }

    /// Tries to estimate the gas for the transaction
    pub async fn estimate_gas(
        &self,
        tx: &TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<U256, P::Error> {
        let tx = utils::serialize(tx);

        let args = match block {
            Some(block) => vec![tx, utils::serialize(&block)],
            None => vec![tx],
        };

        self.0.request("eth_estimateGas", Some(args)).await
    }

    /// Gets the logs matching a given filter
    pub async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, P::Error> {
        self.0.request("eth_getLogs", Some(filter)).await
    }

    /// Gets the accounts on the node
    pub async fn get_accounts(&self) -> Result<Vec<Address>, P::Error> {
        self.0.request("eth_accounts", None::<()>).await
    }

    /// Gets the latest block number via the `eth_BlockNumber` API
    pub async fn get_block_number(&self) -> Result<U256, P::Error> {
        self.0.request("eth_blockNumber", None::<()>).await
    }

    pub async fn get_block(&self, id: impl Into<BlockId>) -> Result<Block<TxHash>, P::Error> {
        self.get_block_gen(id.into(), false).await
    }

    pub async fn get_block_with_txs(
        &self,
        id: impl Into<BlockId>,
    ) -> Result<Block<Transaction>, P::Error> {
        self.get_block_gen(id.into(), true).await
    }

    async fn get_block_gen<Tx: for<'a> Deserialize<'a>>(
        &self,
        id: BlockId,
        include_txs: bool,
    ) -> Result<Block<Tx>, P::Error> {
        let include_txs = utils::serialize(&include_txs);

        match id {
            BlockId::Hash(hash) => {
                let hash = utils::serialize(&hash);
                let args = vec![hash, include_txs];
                self.0.request("eth_getBlockByHash", Some(args)).await
            }
            BlockId::Number(num) => {
                let num = utils::serialize(&num);
                let args = vec![num, include_txs];
                self.0.request("eth_getBlockByNumber", Some(args)).await
            }
        }
    }

    /// Gets the transaction receipt for tx hash
    pub async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        hash: T,
    ) -> Result<TransactionReceipt, P::Error> {
        let hash = hash.into();
        self.0
            .request("eth_getTransactionReceipt", Some(hash))
            .await
    }

    /// Gets the transaction which matches the provided hash via the `eth_getTransactionByHash` API
    pub async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        hash: T,
    ) -> Result<Transaction, P::Error> {
        let hash = hash.into();
        self.0.request("eth_getTransactionByHash", Some(hash)).await
    }

    // State mutations

    /// Broadcasts the transaction request via the `eth_sendTransaction` API
    pub async fn call<T: for<'a> Deserialize<'a>>(
        &self,
        tx: TransactionRequest,
    ) -> Result<T, P::Error> {
        self.0.request("eth_call", Some(tx)).await
    }

    /// Broadcasts the transaction request via the `eth_sendTransaction` API
    pub async fn send_transaction(&self, tx: TransactionRequest) -> Result<TxHash, P::Error> {
        self.0.request("eth_sendTransaction", Some(tx)).await
    }

    /// Broadcasts a raw RLP encoded transaction via the `eth_sendRawTransaction` API
    pub async fn send_raw_transaction(&self, tx: &Transaction) -> Result<TxHash, P::Error> {
        let rlp = utils::serialize(&tx.rlp());
        self.0.request("eth_sendRawTransaction", Some(rlp)).await
    }

    // Account state

    pub async fn get_transaction_count(
        &self,
        from: Address,
        block: Option<BlockNumber>,
    ) -> Result<U256, P::Error> {
        let from = utils::serialize(&from);
        let block = utils::serialize(&block.unwrap_or(BlockNumber::Latest));
        self.0
            .request("eth_getTransactionCount", Some(&[from, block]))
            .await
    }

    pub async fn get_balance(
        &self,
        from: Address,
        block: Option<BlockNumber>,
    ) -> Result<U256, P::Error> {
        let from = utils::serialize(&from);
        let block = utils::serialize(&block.unwrap_or(BlockNumber::Latest));
        self.0.request("eth_getBalance", Some(&[from, block])).await
    }
}
