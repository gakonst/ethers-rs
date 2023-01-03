use super::{from_gwei_f64, GasCategory, GasOracle, GasOracleError, Result};
use async_trait::async_trait;
use ethers_core::types::U256;
use reqwest::Client;
use serde::Deserialize;
use url::Url;

const URL: &str = "https://www.etherchain.org/api/gasPriceOracle";

/// A client over HTTP for the [Etherchain](https://www.etherchain.org/api/gasPriceOracle) gas tracker API
/// that implements the `GasOracle` trait.
#[derive(Clone, Debug)]
#[must_use]
pub struct Etherchain {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub safe_low: f64,
    pub standard: f64,
    pub fast: f64,
    pub fastest: f64,
    pub current_base_fee: f64,
    pub recommended_base_fee: f64,
}

impl Response {
    #[inline]
    pub fn gas_from_category(&self, gas_category: GasCategory) -> f64 {
        match gas_category {
            GasCategory::SafeLow => self.safe_low,
            GasCategory::Standard => self.standard,
            GasCategory::Fast => self.fast,
            GasCategory::Fastest => self.fastest,
        }
    }
}

impl Default for Etherchain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl GasOracle for Etherchain {
    async fn fetch(&self) -> Result<U256> {
        let res = self.query().await?;
        let gas_price = res.gas_from_category(self.gas_category);
        Ok(from_gwei_f64(gas_price))
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256)> {
        Err(GasOracleError::Eip1559EstimationNotSupported)
    }
}

impl Etherchain {
    /// Creates a new [Etherchain](https://etherchain.org/tools/gasPriceOracle) gas price oracle.
    pub fn new() -> Self {
        Self::with_client(Client::new())
    }

    /// Same as [`Self::new`] but with a custom [`Client`].
    pub fn with_client(client: Client) -> Self {
        let url = Url::parse(URL).unwrap();
        Etherchain { client, url, gas_category: GasCategory::Standard }
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
