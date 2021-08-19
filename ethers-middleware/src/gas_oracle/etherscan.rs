use ethers_core::types::U256;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_aux::prelude::*;
use url::Url;

use crate::gas_oracle::{GasCategory, GasOracle, GasOracleError, GWEI_TO_WEI};

const ETHERSCAN_URL_PREFIX: &str =
    "https://api.etherscan.io/api?module=gastracker&action=gasoracle";

/// A client over HTTP for the [Etherscan](https://api.etherscan.io/api?module=gastracker&action=gasoracle) gas tracker API
/// that implements the `GasOracle` trait
#[derive(Clone, Debug)]
pub struct Etherscan {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

#[derive(Deserialize)]
struct EtherscanResponseWrapper {
    result: EtherscanResponse,
}

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct EtherscanResponse {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub safe_gas_price: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub propose_gas_price: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub fast_gas_price: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub last_block: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "suggestBaseFee")]
    pub suggested_base_fee: f64,
    #[serde(deserialize_with = "deserialize_f64_vec")]
    #[serde(rename = "gasUsedRatio")]
    pub gas_used_ratio: Vec<f64>,
}

use serde::de;
use std::str::FromStr;
fn deserialize_f64_vec<'de, D>(deserializer: D) -> Result<Vec<f64>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let str_sequence = String::deserialize(deserializer)?;
    str_sequence
        .split(',')
        .map(|item| f64::from_str(item).map_err(|err| de::Error::custom(err.to_string())))
        .collect()
}

impl Etherscan {
    /// Creates a new [Etherscan](https://etherscan.io/gastracker) gas price oracle.
    pub fn new(api_key: Option<&str>) -> Self {
        let url = match api_key {
            Some(key) => format!("{}&apikey={}", ETHERSCAN_URL_PREFIX, key),
            None => ETHERSCAN_URL_PREFIX.to_string(),
        };

        let url = Url::parse(&url).expect("invalid url");

        Etherscan {
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

    pub async fn query(&self) -> Result<EtherscanResponse, GasOracleError> {
        let res = self
            .client
            .get(self.url.as_ref())
            .send()
            .await?
            .json::<EtherscanResponseWrapper>()
            .await?;
        Ok(res.result)
    }
}

#[async_trait]
impl GasOracle for Etherscan {
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        if matches!(self.gas_category, GasCategory::Fastest) {
            return Err(GasOracleError::GasCategoryNotSupported);
        }

        let res = self.query().await?;

        match self.gas_category {
            GasCategory::SafeLow => Ok(U256::from(res.safe_gas_price * GWEI_TO_WEI)),
            GasCategory::Standard => Ok(U256::from(res.propose_gas_price * GWEI_TO_WEI)),
            GasCategory::Fast => Ok(U256::from(res.fast_gas_price * GWEI_TO_WEI)),
            _ => Err(GasOracleError::GasCategoryNotSupported),
        }
    }
}
