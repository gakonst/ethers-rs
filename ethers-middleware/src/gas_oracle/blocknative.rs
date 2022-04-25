use crate::gas_oracle::{GasCategory, GasOracle, GasOracleError, GWEI_TO_WEI};
use async_trait::async_trait;
use ethers_core::types::U256;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client, ClientBuilder,
};
use serde::Deserialize;
use std::{collections::HashMap, convert::TryInto, iter::FromIterator};
use url::Url;

const BLOCKNATIVE_GAS_PRICE_ENDPOINT: &str = "https://api.blocknative.com/gasprices/blockprices";

fn gas_category_to_confidence(gas_category: &GasCategory) -> u64 {
    match gas_category {
        GasCategory::SafeLow => 80,
        GasCategory::Standard => 90,
        GasCategory::Fast => 95,
        GasCategory::Fastest => 99,
    }
}

/// A client over HTTP for the [BlockNative](https://www.blocknative.com/gas-estimator) gas tracker API
/// that implements the `GasOracle` trait
#[derive(Clone, Debug)]
pub struct BlockNative {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlockNativeGasResponse {
    system: Option<String>,
    network: Option<String>,
    unit: Option<String>,
    max_price: Option<u64>,
    block_prices: Vec<BlockPrice>,
    estimated_base_fees: Vec<HashMap<String, Vec<BaseFeeEstimate>>>,
}

impl BlockNativeGasResponse {
    pub fn get_estimation_for(
        &self,
        gas_category: &GasCategory,
    ) -> Result<EstimatedPrice, GasOracleError> {
        let confidence = gas_category_to_confidence(gas_category);
        Ok(self
            .block_prices
            .first()
            .ok_or(GasOracleError::InvalidResponse)?
            .estimated_prices
            .iter()
            .find(|p| p.confidence == confidence)
            .ok_or(GasOracleError::GasCategoryNotSupported)?
            .clone())
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlockPrice {
    block_number: u64,
    estimated_transaction_count: u64,
    base_fee_per_gas: f64,
    estimated_prices: Vec<EstimatedPrice>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EstimatedPrice {
    confidence: u64,
    price: u64,
    max_priority_fee_per_gas: f64,
    max_fee_per_gas: f64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BaseFeeEstimate {
    confidence: u64,
    base_fee: f64,
}

impl BlockNative {
    /// Creates a new [BlockNative](https://www.blocknative.com/gas-estimator) gas oracle
    pub fn new(api_key: &str) -> Self {
        let header_value = HeaderValue::from_str(api_key).unwrap();
        let headers = HeaderMap::from_iter([(AUTHORIZATION, header_value)]);
        let client = ClientBuilder::new().default_headers(headers).build().unwrap();
        Self {
            client,
            url: BLOCKNATIVE_GAS_PRICE_ENDPOINT.try_into().unwrap(),
            gas_category: GasCategory::Standard,
        }
    }

    /// Sets the gas price category to be used when fetching the gas price.
    #[must_use]
    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }

    /// Perform request to Blocknative, decode response
    pub async fn request(&self) -> Result<BlockNativeGasResponse, GasOracleError> {
        Ok(self.client.get(self.url.as_ref()).send().await?.error_for_status()?.json().await?)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl GasOracle for BlockNative {
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        let prices = self.request().await?.get_estimation_for(&self.gas_category)?;
        Ok(U256::from(prices.price * 100_u64) * U256::from(GWEI_TO_WEI) / U256::from(100))
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256), GasOracleError> {
        let prices = self.request().await?.get_estimation_for(&self.gas_category)?;
        let base_fee = U256::from((prices.max_fee_per_gas * 100.0) as u64) *
            U256::from(GWEI_TO_WEI) /
            U256::from(100);
        let prio_fee = U256::from((prices.max_priority_fee_per_gas * 100.0) as u64) *
            U256::from(GWEI_TO_WEI) /
            U256::from(100);
        Ok((base_fee, prio_fee))
    }
}
