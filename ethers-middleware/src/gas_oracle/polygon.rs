use crate::gas_oracle::{GasCategory, GasOracle, GasOracleError};
use async_trait::async_trait;
use ethers_core::types::{u256_from_f64_saturating, Chain, U256};
use reqwest::Client;
use serde::Deserialize;
use url::Url;

const GAS_PRICE_ENDPOINT: &str = "https://gasstation-mainnet.matic.network/v2";
const MUMBAI_GAS_PRICE_ENDPOINT: &str = "https://gasstation-mumbai.matic.today/v2";

/// The [Polygon](https://docs.polygon.technology/docs/develop/tools/polygon-gas-station/) gas station API
/// Queries over HTTP and implements the `GasOracle` trait
#[derive(Clone, Debug)]
pub struct Polygon {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

/// The response from the Polygon gas station API.
/// Gas prices are in Gwei.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    estimated_base_fee: f64,
    safe_low: GasEstimate,
    standard: GasEstimate,
    fast: GasEstimate,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GasEstimate {
    max_priority_fee: f64,
    max_fee: f64,
}

impl Polygon {
    pub fn new(chain: Chain) -> Result<Self, GasOracleError> {
        Self::with_client(Client::new(), chain)
    }

    pub fn with_client(client: Client, chain: Chain) -> Result<Self, GasOracleError> {
        // TODO: Sniff chain from chain id.
        let url = match chain {
            Chain::Polygon => Url::parse(GAS_PRICE_ENDPOINT).unwrap(),
            Chain::PolygonMumbai => Url::parse(MUMBAI_GAS_PRICE_ENDPOINT).unwrap(),
            _ => return Err(GasOracleError::UnsupportedChain),
        };
        Ok(Self { client, url, gas_category: GasCategory::Standard })
    }

    /// Sets the gas price category to be used when fetching the gas price.
    #[must_use]
    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }

    /// Perform request to Blocknative, decode response
    pub async fn request(&self) -> Result<(f64, GasEstimate), GasOracleError> {
        let response: Response =
            self.client.get(self.url.as_ref()).send().await?.error_for_status()?.json().await?;
        let estimate = match self.gas_category {
            GasCategory::SafeLow => response.safe_low,
            GasCategory::Standard => response.standard,
            GasCategory::Fast => response.fast,
            GasCategory::Fastest => response.fast,
        };
        Ok((response.estimated_base_fee, estimate))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl GasOracle for Polygon {
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        let (base_fee, estimate) = self.request().await?;
        let fee = base_fee + estimate.max_priority_fee;
        Ok(from_gwei(fee))
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256), GasOracleError> {
        let (_, estimate) = self.request().await?;
        Ok((from_gwei(estimate.max_fee), from_gwei(estimate.max_priority_fee)))
    }
}

fn from_gwei(gwei: f64) -> U256 {
    u256_from_f64_saturating(gwei * 1.0e9_f64)
}
