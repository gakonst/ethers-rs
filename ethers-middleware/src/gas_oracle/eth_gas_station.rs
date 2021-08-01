use ethers_core::types::U256;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use url::Url;

use crate::gas_oracle::{GasCategory, GasOracle, GasOracleError, GWEI_TO_WEI};

const ETH_GAS_STATION_URL_PREFIX: &str = "https://ethgasstation.info/api/ethgasAPI.json";

/// A client over HTTP for the [EthGasStation](https://ethgasstation.info/api/ethgasAPI.json) gas tracker API
/// that implements the `GasOracle` trait
#[derive(Debug)]
pub struct EthGasStation {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

#[derive(Deserialize)]
struct EthGasStationResponse {
    #[serde(rename = "safeLow")]
    safe_low: f64,
    average: u64,
    fast: u64,
    fastest: u64,
}

impl EthGasStation {
    /// Creates a new [EthGasStation](https://docs.ethgasstation.info/) gas oracle
    pub fn new(api_key: Option<&'static str>) -> Self {
        let url = match api_key {
            Some(key) => format!("{}?api-key={}", ETH_GAS_STATION_URL_PREFIX, key),
            None => ETH_GAS_STATION_URL_PREFIX.to_string(),
        };

        let url = Url::parse(&url).expect("invalid url");

        EthGasStation {
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
impl GasOracle for EthGasStation {
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        let res = self
            .client
            .get(self.url.as_ref())
            .send()
            .await?
            .json::<EthGasStationResponse>()
            .await?;

        let gas_price = match self.gas_category {
            GasCategory::SafeLow => U256::from((res.safe_low.ceil() as u64 * GWEI_TO_WEI) / 10),
            GasCategory::Standard => U256::from((res.average * GWEI_TO_WEI) / 10),
            GasCategory::Fast => U256::from((res.fast * GWEI_TO_WEI) / 10),
            GasCategory::Fastest => U256::from((res.fastest * GWEI_TO_WEI) / 10),
        };

        Ok(gas_price)
    }
}
