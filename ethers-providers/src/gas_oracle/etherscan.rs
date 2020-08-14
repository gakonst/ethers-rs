use async_trait::async_trait;
use reqwest::{Client, Error as ReqwestError};
use serde::Deserialize;
use serde_aux::prelude::*;
use thiserror::Error;
use url::Url;

use crate::gas_oracle::{GasOracleError, GasOracleFetch, GasOracleResponse};

const ETHERSCAN_URL_PREFIX: &str =
    "https://api.etherscan.io/api?module=gastracker&action=gasoracle";

pub struct Etherscan {
    client: Client,
    url: Url,
}

#[derive(Deserialize)]
struct EtherscanResponse {
    result: EtherscanResponseInner,
}

#[derive(Deserialize)]
struct EtherscanResponseInner {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "LastBlock")]
    last_block: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "SafeGasPrice")]
    safe_gas_price: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "ProposeGasPrice")]
    propose_gas_price: u64,
}

impl From<EtherscanResponse> for GasOracleResponse {
    fn from(src: EtherscanResponse) -> Self {
        Self {
            block: Some(src.result.last_block),
            safe_low: Some(src.result.safe_gas_price),
            standard: Some(src.result.propose_gas_price),
            fast: None,
            fastest: None,
        }
    }
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error(transparent)]
    ReqwestError(#[from] ReqwestError),
}

impl From<ClientError> for GasOracleError {
    fn from(src: ClientError) -> GasOracleError {
        GasOracleError::HttpClientError(Box::new(src))
    }
}

impl Etherscan {
    pub fn new(api_key: Option<&'static str>) -> Self {
        let url = match api_key {
            Some(key) => format!("{}&apikey={}", ETHERSCAN_URL_PREFIX, key),
            None => ETHERSCAN_URL_PREFIX.to_string(),
        };

        let url = Url::parse(&url).expect("invalid url");

        Etherscan {
            client: Client::new(),
            url,
        }
    }
}

#[async_trait]
impl GasOracleFetch for Etherscan {
    type Error = ClientError;

    async fn fetch(&self) -> Result<GasOracleResponse, ClientError> {
        let res = self
            .client
            .get(self.url.as_ref())
            .send()
            .await?
            .json::<EtherscanResponse>()
            .await?;

        Ok(res.into())
    }
}
