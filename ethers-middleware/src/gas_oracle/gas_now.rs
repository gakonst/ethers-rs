use super::{GasCategory, GasOracle, GasOracleError, Result};
use async_trait::async_trait;
use ethers_core::types::U256;
use reqwest::Client;
use serde::Deserialize;
use url::Url;

const URL: &str = "https://beaconcha.in/api/v1/execution/gasnow";

/// A client over HTTP for the [beaconcha.in GasNow](https://beaconcha.in/gasnow) gas tracker API
/// that implements the `GasOracle` trait.
#[derive(Clone, Debug)]
#[must_use]
pub struct GasNow {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct Response {
    pub code: u64,
    pub data: ResponseData,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct ResponseData {
    pub rapid: u64,
    pub fast: u64,
    pub standard: u64,
    pub slow: u64,
    pub timestamp: u64,
    #[serde(rename = "priceUSD")]
    pub price_usd: f64,
}

impl Response {
    #[inline]
    pub fn gas_from_category(&self, gas_category: GasCategory) -> u64 {
        self.data.gas_from_category(gas_category)
    }
}

impl ResponseData {
    fn gas_from_category(&self, gas_category: GasCategory) -> u64 {
        match gas_category {
            GasCategory::SafeLow => self.slow,
            GasCategory::Standard => self.standard,
            GasCategory::Fast => self.fast,
            GasCategory::Fastest => self.rapid,
        }
    }
}

impl Default for GasNow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl GasOracle for GasNow {
    async fn fetch(&self) -> Result<U256> {
        let res = self.query().await?;
        let gas_price = res.gas_from_category(self.gas_category);
        Ok(U256::from(gas_price))
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256)> {
        Err(GasOracleError::Eip1559EstimationNotSupported)
    }
}

impl GasNow {
    /// Creates a new [beaconcha.in GasNow](https://beaconcha.in/gasnow) gas price oracle.
    pub fn new() -> Self {
        Self::with_client(Client::new())
    }

    /// Same as [`Self::new`] but with a custom [`Client`].
    pub fn with_client(client: Client) -> Self {
        let url = Url::parse(URL).unwrap();
        Self { client, url, gas_category: GasCategory::Standard }
    }

    /// Sets the gas price category to be used when fetching the gas price.
    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }

    /// Perform a request to the gas price API and deserialize the response.
    pub async fn query(&self) -> Result<Response> {
        let response =
            self.client.get(self.url.as_ref()).send().await?.error_for_status()?.json().await?;
        Ok(response)
    }
}
