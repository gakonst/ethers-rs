use async_trait::async_trait;
use reqwest::{Client, Error as ReqwestError};
use serde::Deserialize;
use serde_aux::prelude::*;
use thiserror::Error;
use url::Url;

use crate::gas_oracle::{GasOracleError, GasOracleFetch, GasOracleResponse};

const ETHERCHAIN_URL: &str = "https://www.etherchain.org/api/gasPriceOracle";

pub struct Etherchain {
    client: Client,
    url: Url,
}

#[derive(Deserialize)]
struct EtherchainResponse {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "safeLow")]
    safe_low: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    standard: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    fast: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    fastest: f32,
}

impl From<EtherchainResponse> for GasOracleResponse {
    fn from(src: EtherchainResponse) -> Self {
        Self {
            block: None,
            safe_low: Some(src.safe_low as u64),
            standard: Some(src.standard as u64),
            fast: Some(src.fast as u64),
            fastest: Some(src.fastest as u64),
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

impl Etherchain {
    pub fn new() -> Self {
        let url = Url::parse(ETHERCHAIN_URL).expect("invalid url");

        Etherchain {
            client: Client::new(),
            url,
        }
    }
}

#[async_trait]
impl GasOracleFetch for Etherchain {
    type Error = ClientError;

    async fn fetch(&self) -> Result<GasOracleResponse, ClientError> {
        let res = self
            .client
            .get(self.url.as_ref())
            .send()
            .await?
            .json::<EtherchainResponse>()
            .await?;

        Ok(res.into())
    }
}
