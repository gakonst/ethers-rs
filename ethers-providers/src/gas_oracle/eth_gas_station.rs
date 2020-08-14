use async_trait::async_trait;
use reqwest::{Client, Error as ReqwestError};
use serde::Deserialize;
use thiserror::Error;
use url::Url;

use crate::gas_oracle::{GasOracleError, GasOracleFetch, GasOracleResponse};

const ETH_GAS_STATION_URL_PREFIX: &str = "https://ethgasstation.info/api/ethgasAPI.json";

pub struct EthGasStation {
    client: Client,
    url: Url,
}

#[derive(Deserialize)]
struct EthGasStationResponse {
    #[serde(rename = "blockNum")]
    block_num: u64,
    #[serde(rename = "safeLow")]
    safe_low: u64,
    average: u64,
    fast: u64,
    fastest: u64,
}

impl From<EthGasStationResponse> for GasOracleResponse {
    fn from(src: EthGasStationResponse) -> Self {
        Self {
            block: Some(src.block_num),
            safe_low: Some(src.safe_low / 10),
            standard: Some(src.average / 10),
            fast: Some(src.fast / 10),
            fastest: Some(src.fastest / 10),
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

impl EthGasStation {
    pub fn new(api_key: Option<&'static str>) -> Self {
        let url = match api_key {
            Some(key) => format!("{}?api-key={}", ETH_GAS_STATION_URL_PREFIX, key),
            None => ETH_GAS_STATION_URL_PREFIX.to_string(),
        };

        let url = Url::parse(&url).expect("invalid url");

        EthGasStation {
            client: Client::new(),
            url,
        }
    }
}

#[async_trait]
impl GasOracleFetch for EthGasStation {
    type Error = ClientError;

    async fn fetch(&self) -> Result<GasOracleResponse, ClientError> {
        let res = self
            .client
            .get(self.url.as_ref())
            .send()
            .await?
            .json::<EthGasStationResponse>()
            .await?;

        Ok(res.into())
    }
}
