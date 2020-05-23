use crate::{
    jsonrpc::{ClientError, HttpClient},
    types::{Address, BlockNumber, Bytes, Transaction, TransactionRequest, TxHash, U256},
    utils,
};
use async_trait::async_trait;
use std::convert::TryFrom;
use url::{ParseError, Url};

/// An Ethereum JSON-RPC compatible backend
#[derive(Clone, Debug)]
pub struct Provider(HttpClient);

impl From<HttpClient> for Provider {
    fn from(src: HttpClient) -> Self {
        Self(src)
    }
}

impl TryFrom<&str> for Provider {
    type Error = ParseError;

    fn try_from(src: &str) -> Result<Self, Self::Error> {
        Ok(Provider(HttpClient::new(Url::parse(src)?)))
    }
}

#[async_trait]
// TODO: Figure out a way to re-use the arguments with various transports -> need a trait which has a
// `request` method
impl ProviderTrait for Provider {
    type Error = ClientError;

    async fn get_block_number(&self) -> Result<U256, Self::Error> {
        self.0.request("eth_blockNumber", None::<()>).await
    }

    async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        hash: T,
    ) -> Result<Transaction, Self::Error> {
        let hash = hash.into();
        self.0.request("eth_getTransactionByHash", Some(hash)).await
    }

    async fn send_transaction(&self, tx: TransactionRequest) -> Result<TxHash, Self::Error> {
        self.0.request("eth_sendTransaction", Some(vec![tx])).await
    }

    async fn send_raw_transaction(&self, rlp: &Bytes) -> Result<TxHash, Self::Error> {
        let rlp = utils::serialize(&rlp);
        self.0.request("eth_sendRawTransaction", Some(rlp)).await
    }

    async fn get_transaction_count(
        &self,
        from: Address,
        block: Option<BlockNumber>,
    ) -> Result<U256, Self::Error> {
        let from = utils::serialize(&from);
        let block = utils::serialize(&block);
        self.0
            .request("eth_getTransactionCount", Some(&[from, block]))
            .await
    }
}

/// Trait for providing backend services. Different implementations for this may be used for using
/// indexers or using multiple providers at the same time
#[async_trait]
pub trait ProviderTrait {
    type Error;

    async fn get_block_number(&self) -> Result<U256, Self::Error>;

    /// Gets a transaction by it shash
    async fn get_transaction<T: Into<TxHash> + Send + Sync>(
        &self,
        tx_hash: T,
    ) -> Result<Transaction, Self::Error>;

    /// Sends a transaciton request to the node
    async fn send_transaction(&self, tx: TransactionRequest) -> Result<TxHash, Self::Error>;

    /// Broadcasts an RLP encoded signed transaction
    async fn send_raw_transaction(&self, tx: &Bytes) -> Result<TxHash, Self::Error>;

    async fn get_transaction_count(
        &self,
        from: Address,
        block: Option<BlockNumber>,
    ) -> Result<U256, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethereum_types::Address;
    use std::str::FromStr;

    // TODO: Make a Ganache helper

    #[tokio::test]
    async fn get_balance() {
        let provider = Provider::try_from("http://localhost:8545").unwrap();
        let num = provider.get_block_number().await.unwrap();
        assert_eq!(num, U256::from(0));
    }

    #[tokio::test]
    async fn send_transaction() {
        let provider = Provider::try_from("http://localhost:8545").unwrap();
        let tx_req = TransactionRequest {
            from: Address::from_str("e98C5Abe55bD5478717BC67DcE404B8730672298").unwrap(),
            to: Some(Address::from_str("d5CB69Fb66809B7Ca203DAe8fB571DD291a86764").unwrap()),
            nonce: None,
            data: None,
            value: Some(1000.into()),
            gas_price: None,
            gas: None,
        };
        let tx_hash = provider.send_transaction(tx_req).await.unwrap();
        dbg!(tx_hash);
    }
}
