use std::collections::HashMap;

use ethers_core::types::U256;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use url::Url;

use crate::gas_oracle::{GasCategory, GasOracle, GasOracleError, GWEI_TO_WEI};

const ETH_GAS_STATION_URL_PREFIX: &str = "https://ethgasstation.info/api/ethgasAPI.json";

/// A client over HTTP for the [EthGasStation](https://ethgasstation.info/api/ethgasAPI.json) gas tracker API
/// that implements the `GasOracle` trait
#[derive(Clone, Debug)]
pub struct EthGasStation {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
/// Eth Gas Station's response for the current recommended fast, standard and
/// safe low gas prices on the Ethereum network, along with the current block
/// and wait times for each "speed".
pub struct EthGasStationResponse {
    /// Recommended safe(expected to be mined in < 30 minutes) gas price in
    /// x10 Gwei (divide by 10 to convert it to gwei)
    pub safe_low: f64,
    /// Recommended average(expected to be mined in < 5 minutes) gas price in
    /// x10 Gwei (divide by 10 to convert it to gwei)
    pub average: u64,
    /// Recommended fast(expected to be mined in < 2 minutes) gas price in
    /// x10 Gwei (divide by 10 to convert it to gwei)
    pub fast: u64,
    /// Recommended fastest(expected to be mined in < 30 seconds) gas price
    /// in x10 Gwei(divide by 10 to convert it to gwei)
    pub fastest: u64,

    // post eip-1559 fields
    #[serde(rename = "block_time")] // inconsistent json response naming...
    /// Average time(in seconds) to mine one single block
    pub block_time: f64,
    /// The latest block number
    pub block_num: u64,
    /// Smallest value of (gasUsed / gaslimit) from last 10 blocks
    pub speed: f64,
    /// Waiting time(in minutes) for the `safe_low` gas price
    pub safe_low_wait: f64,
    /// Waiting time(in minutes) for the `average` gas price
    pub avg_wait: f64,
    /// Waiting time(in minutes) for the `fast` gas price
    pub fast_wait: f64,
    /// Waiting time(in minutes) for the `fastest` gas price
    pub fastest_wait: f64,
    // What is this?
    pub gas_price_range: HashMap<u64, f64>,
}

impl EthGasStation {
    /// Creates a new [EthGasStation](https://docs.ethgasstation.info/) gas oracle
    pub fn new(api_key: Option<&str>) -> Self {
        Self::with_client(Client::new(), api_key)
    }

    /// Same as [`Self::new`] but with a custom [`Client`].
    pub fn with_client(client: Client, api_key: Option<&str>) -> Self {
        let mut url = Url::parse(ETH_GAS_STATION_URL_PREFIX).expect("invalid url");
        if let Some(key) = api_key {
            url.query_pairs_mut().append_pair("api-key", key);
        }
        EthGasStation { client, url, gas_category: GasCategory::Standard }
    }

    /// Sets the gas price category to be used when fetching the gas price.
    #[must_use]
    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }

    pub async fn query(&self) -> Result<EthGasStationResponse, GasOracleError> {
        Ok(self.client.get(self.url.as_ref()).send().await?.json::<EthGasStationResponse>().await?)
    }
}

impl Default for EthGasStation {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl GasOracle for EthGasStation {
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        let res = self.query().await?;
        let gas_price = match self.gas_category {
            GasCategory::SafeLow => U256::from((res.safe_low.ceil() as u64 * GWEI_TO_WEI) / 10),
            GasCategory::Standard => U256::from((res.average * GWEI_TO_WEI) / 10),
            GasCategory::Fast => U256::from((res.fast * GWEI_TO_WEI) / 10),
            GasCategory::Fastest => U256::from((res.fastest * GWEI_TO_WEI) / 10),
        };

        Ok(gas_price)
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256), GasOracleError> {
        Err(GasOracleError::Eip1559EstimationNotSupported)
    }
}
