use ethers_core::types::U256;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use url::Url;

use crate::gas_oracle::{GasCategory, GasOracle, GasOracleError};

const GAS_NOW_URL: &str = "https://www.gasnow.org/api/v1/gas/price";

/// A client over HTTP for the [GasNow](https://www.gasnow.org/api/v1/gas/price) gas tracker API
/// that implements the `GasOracle` trait
#[derive(Debug)]
pub struct GasNow {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

impl Default for GasNow {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct GasNowResponse {
    data: GasNowResponseInner,
}

#[derive(Deserialize)]
struct GasNowResponseInner {
    #[serde(rename = "top50")]
    top_50: u64,
    #[serde(rename = "top200")]
    top_200: u64,
    #[serde(rename = "top400")]
    top_400: u64,
}

impl GasNow {
    pub fn new() -> Self {
        let url = Url::parse(GAS_NOW_URL).expect("invalid url");

        Self {
            client: Client::new(),
            url,
            gas_category: GasCategory::Standard,
        }
    }

    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }
}

#[async_trait]
impl GasOracle for GasNow {
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        let res = self
            .client
            .get(self.url.as_ref())
            .send()
            .await?
            .json::<GasNowResponse>()
            .await?;

        let gas_price = match self.gas_category {
            GasCategory::SafeLow => U256::from(res.data.top_400),
            GasCategory::Standard => U256::from(res.data.top_200),
            _ => U256::from(res.data.top_50),
        };

        Ok(gas_price)
    }
}
