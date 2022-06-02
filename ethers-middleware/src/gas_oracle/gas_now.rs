use ethers_core::types::U256;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use url::Url;

use crate::gas_oracle::{GasCategory, GasOracle, GasOracleError};

const GAS_NOW_URL: &str = "https://etherchain.org/api/gasnow";

/// A client over HTTP for the [Etherchain GasNow](https://etherchain.org/tools/gasnow) gas tracker API
/// that implements the `GasOracle` trait
#[derive(Clone, Debug)]
pub struct GasNow {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

#[derive(Deserialize)]
struct GasNowResponseWrapper {
    data: GasNowResponse,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct GasNowResponse {
    pub rapid: u64,
    pub fast: u64,
    pub standard: u64,
    pub slow: u64,
}

impl GasNow {
    /// Creates a new [Etherchain GasNow](https://etherchain.org/tools/gasnow) gas price oracle.
    pub fn new() -> Self {
        Self::with_client(Client::new())
    }

    /// Same as [`Self::new`] but with a custom [`Client`].
    pub fn with_client(client: Client) -> Self {
        let url = Url::parse(GAS_NOW_URL).expect("invalid url");

        Self { client, url, gas_category: GasCategory::Standard }
    }

    /// Sets the gas price category to be used when fetching the gas price.
    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }

    pub async fn query(&self) -> Result<GasNowResponse, GasOracleError> {
        let res = self
            .client
            .get(self.url.as_ref())
            .send()
            .await?
            .json::<GasNowResponseWrapper>()
            .await?;
        Ok(res.data)
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
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        let res = self.query().await?;
        let gas_price = match self.gas_category {
            GasCategory::SafeLow => U256::from(res.slow),
            GasCategory::Standard => U256::from(res.standard),
            GasCategory::Fast => U256::from(res.fast),
            GasCategory::Fastest => U256::from(res.rapid),
        };

        Ok(gas_price)
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256), GasOracleError> {
        Err(GasOracleError::Eip1559EstimationNotSupported)
    }
}
