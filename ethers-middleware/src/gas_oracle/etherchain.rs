use ethers_core::types::U256;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use url::Url;

use crate::gas_oracle::{GasCategory, GasOracle, GasOracleError, GWEI_TO_WEI};

const ETHERCHAIN_URL: &str = "https://www.etherchain.org/api/gasPriceOracle";

/// A client over HTTP for the [Etherchain](https://www.etherchain.org/api/gasPriceOracle) gas tracker API
/// that implements the `GasOracle` trait
#[derive(Clone, Debug)]
pub struct Etherchain {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct EtherchainResponse {
    pub safe_low: f32,
    pub standard: f32,
    pub fast: f32,
    pub fastest: f32,
    pub current_base_fee: f32,
    pub recommended_base_fee: f32,
}

impl Etherchain {
    /// Creates a new [Etherchain](https://etherchain.org/tools/gasPriceOracle) gas price oracle.
    pub fn new() -> Self {
        Self::with_client(Client::new())
    }

    /// Same as [`Self::new`] but with a custom [`Client`].
    pub fn with_client(client: Client) -> Self {
        let url = Url::parse(ETHERCHAIN_URL).expect("invalid url");

        Etherchain { client, url, gas_category: GasCategory::Standard }
    }

    /// Sets the gas price category to be used when fetching the gas price.
    #[must_use]
    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }

    pub async fn query(&self) -> Result<EtherchainResponse, GasOracleError> {
        Ok(self.client.get(self.url.as_ref()).send().await?.json::<EtherchainResponse>().await?)
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
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        let res = self.query().await?;
        let gas_price = match self.gas_category {
            GasCategory::SafeLow => U256::from((res.safe_low as u64) * GWEI_TO_WEI),
            GasCategory::Standard => U256::from((res.standard as u64) * GWEI_TO_WEI),
            GasCategory::Fast => U256::from((res.fast as u64) * GWEI_TO_WEI),
            GasCategory::Fastest => U256::from((res.fastest as u64) * GWEI_TO_WEI),
        };

        Ok(gas_price)
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256), GasOracleError> {
        Err(GasOracleError::Eip1559EstimationNotSupported)
    }
}
