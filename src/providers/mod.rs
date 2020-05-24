//! Ethereum compatible providers
//! Currently supported:
//! - Raw HTTP POST requests
//!
//! TODO: WebSockets, multiple backends, popular APIs etc.
mod http;

use crate::{
    signers::{Client, Signer},
    types::{Address, BlockNumber, Transaction, TransactionRequest, TxHash, U256},
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
            signer,
            provider: self,
        }
    }

    /// Gets the latest block number via the `eth_BlockNumber` API
    pub async fn get_block_number(&self) -> Result<U256, P::Error> {
        self.0.request("eth_blockNumber", None::<()>).await
    }

    /// Gets the transaction which matches the provided hash via the `eth_getTransactionByHash` API
    pub async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        hash: T,
    ) -> Result<Transaction, P::Error> {
        let hash = hash.into();
        self.0.request("eth_getTransactionByHash", Some(hash)).await
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
}
