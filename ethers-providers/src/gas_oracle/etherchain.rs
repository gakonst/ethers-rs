use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_aux::prelude::*;
use url::Url;

use crate::gas_oracle::{GasOracle, GasOracleError, GasOracleResponse};

const ETHERCHAIN_URL: &str = "https://www.etherchain.org/api/gasPriceOracle";

/// A client over HTTP for the [Etherchain](https://www.etherchain.org/api/gasPriceOracle) gas tracker API
/// that implements the `GasOracle` trait
#[derive(Debug)]
pub struct Etherchain {
    client: Client,
    url: Url,
}

impl Default for Etherchain {
    fn default() -> Self {
        Self::new()
    }
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
impl GasOracle for Etherchain {
    async fn fetch(&self) -> Result<GasOracleResponse, GasOracleError> {
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
