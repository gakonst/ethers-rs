use crate::jsonrpc::{ClientError, HttpClient};
use async_trait::async_trait;
use ethereum_types::U256;
use std::convert::TryFrom;
use url::{ParseError, Url};

/// An Ethereum JSON-RPC compatible backend
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
impl ProviderTrait for Provider {
    type Error = ClientError;

    async fn get_block_number(&self) -> Result<U256, Self::Error> {
        self.0.request("eth_blockNumber", None::<()>).await
    }
}

#[async_trait]
pub trait ProviderTrait {
    type Error;

    async fn get_block_number(&self) -> Result<U256, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_balance() {
        let provider = Provider::try_from("http://localhost:8545").unwrap();
        let num = provider.get_block_number().await.unwrap();
        assert_eq!(num, U256::from(0));
    }
}
