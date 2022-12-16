#![allow(deprecated)]

use super::{GasCategory, GasOracle, GasOracleError, Result, GWEI_TO_WEI_U256};
use async_trait::async_trait;
use ethers_core::types::U256;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

const URL: &str = "https://ethgasstation.info/api/ethgasAPI.json";

/// A client over HTTP for the [EthGasStation](https://ethgasstation.info) gas tracker API
/// that implements the `GasOracle` trait.
#[derive(Clone, Debug)]
#[deprecated = "ETHGasStation is shutting down: https://twitter.com/ETHGasStation/status/1597341610777317376"]
#[must_use]
pub struct EthGasStation {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

/// Eth Gas Station's response for the current recommended fast, standard and
/// safe low gas prices on the Ethereum network, along with the current block
/// and wait times for each "speed".
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// Recommended safe (expected to be mined in < 30 minutes).
    ///
    /// In gwei * 10 (divide by 10 to convert it to gwei).
    pub safe_low: u64,
    /// Recommended average (expected to be mined in < 5 minutes).
    ///
    /// In gwei * 10 (divide by 10 to convert it to gwei).
    pub average: u64,
    /// Recommended fast (expected to be mined in < 2 minutes).
    ///
    /// In gwei * 10 (divide by 10 to convert it to gwei).
    pub fast: u64,
    /// Recommended fastest (expected to be mined in < 30 seconds).
    ///
    /// In gwei * 10 (divide by 10 to convert it to gwei).
    pub fastest: u64,

    // post eip-1559 fields
    /// Average time (in seconds) to mine a single block.
    #[serde(rename = "block_time")] // inconsistent json response naming...
    pub block_time: f64,
    /// The latest block number.
    pub block_num: u64,
    /// Smallest value of `gasUsed / gaslimit` from the last 10 blocks.
    pub speed: f64,
    /// Waiting time (in minutes) for the `safe_low` gas price.
    pub safe_low_wait: f64,
    /// Waiting time (in minutes) for the `average` gas price.
    pub avg_wait: f64,
    /// Waiting time (in minutes) for the `fast` gas price.
    pub fast_wait: f64,
    /// Waiting time (in minutes) for the `fastest` gas price.
    pub fastest_wait: f64,
    // What is this?
    pub gas_price_range: HashMap<u64, f64>,
}

impl Response {
    #[inline]
    pub fn gas_from_category(&self, gas_category: GasCategory) -> u64 {
        match gas_category {
            GasCategory::SafeLow => self.safe_low,
            GasCategory::Standard => self.average,
            GasCategory::Fast => self.fast,
            GasCategory::Fastest => self.fastest,
        }
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
    async fn fetch(&self) -> Result<U256> {
        let res = self.query().await?;
        let gas_price = res.gas_from_category(self.gas_category);
        // gas_price is in `gwei * 10`
        Ok(U256::from(gas_price) * GWEI_TO_WEI_U256 / U256::from(10_u64))
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256)> {
        Err(GasOracleError::Eip1559EstimationNotSupported)
    }
}

impl EthGasStation {
    /// Creates a new [EthGasStation](https://docs.ethgasstation.info/) gas oracle.
    pub fn new(api_key: Option<&str>) -> Self {
        Self::with_client(Client::new(), api_key)
    }

    /// Same as [`Self::new`] but with a custom [`Client`].
    pub fn with_client(client: Client, api_key: Option<&str>) -> Self {
        let mut url = Url::parse(URL).unwrap();
        if let Some(key) = api_key {
            url.query_pairs_mut().append_pair("api-key", key);
        }
        EthGasStation { client, url, gas_category: GasCategory::Standard }
    }

    /// Sets the gas price category to be used when fetching the gas price.
    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }

    /// Perform a request to the gas price API and deserialize the response.
    pub async fn query(&self) -> Result<Response> {
        let response =
            self.client.get(self.url.clone()).send().await?.error_for_status()?.json().await?;
        Ok(response)
    }
}
