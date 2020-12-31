use ethers_core::types::U256;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_aux::prelude::*;
use url::Url;

use crate::gas_oracle::{GasCategory, GasOracle, GasOracleError, GWEI_TO_WEI};

const ETHERCHAIN_URL: &str = "https://www.etherchain.org/api/gasPriceOracle";

/// A client over HTTP for the [Etherchain](https://www.etherchain.org/api/gasPriceOracle) gas tracker API
/// that implements the `GasOracle` trait
#[derive(Debug)]
pub struct Etherchain {
    client: Client,
    url: Url,
    gas_category: GasCategory,
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

impl Etherchain {
    /// Creates a new [Etherchain](https://etherchain.org/tools/gasPriceOracle) gas price oracle.
    pub fn new() -> Self {
        let url = Url::parse(ETHERCHAIN_URL).expect("invalid url");

        Etherchain {
            client: Client::new(),
            url,
            gas_category: GasCategory::Standard,
        }
    }

    /// Sets the gas price category to be used when fetching the gas price.
    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }
}

#[async_trait]
impl GasOracle for Etherchain {
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        let res = self
            .client
            .get(self.url.as_ref())
            .send()
            .await?
            .json::<EtherchainResponse>()
            .await?;

        let gas_price = match self.gas_category {
            GasCategory::SafeLow => U256::from((res.safe_low as u64) * GWEI_TO_WEI),
            GasCategory::Standard => U256::from((res.standard as u64) * GWEI_TO_WEI),
            GasCategory::Fast => U256::from((res.fast as u64) * GWEI_TO_WEI),
            GasCategory::Fastest => U256::from((res.fastest as u64) * GWEI_TO_WEI),
        };

        Ok(gas_price)
    }
}
