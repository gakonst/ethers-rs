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
#[derive(Debug)]
pub struct Etherscan {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

#[derive(Deserialize)]
struct EtherscanResponse {
    result: EtherscanResponseInner,
}

#[derive(Deserialize)]
struct EtherscanResponseInner {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "SafeGasPrice")]
    safe_gas_price: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "ProposeGasPrice")]
    propose_gas_price: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "FastGasPrice")]
    fast_gas_price: u64,
}

impl Etherscan {
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

    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }
}

#[async_trait]
impl GasOracle for Etherscan {
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        if matches!(self.gas_category, GasCategory::Fastest) {
            return Err(GasOracleError::GasCategoryNotSupported);
        }

        let res = self
            .client
            .get(self.url.as_ref())
            .send()
            .await?
            .json::<EtherscanResponse>()
            .await?;

        match self.gas_category {
            GasCategory::SafeLow => Ok(U256::from(res.result.safe_gas_price * GWEI_TO_WEI)),
            GasCategory::Standard => Ok(U256::from(res.result.propose_gas_price * GWEI_TO_WEI)),
            GasCategory::Fast => Ok(U256::from(res.result.fast_gas_price * GWEI_TO_WEI)),
            _ => Err(GasOracleError::GasCategoryNotSupported),
        }
    }
}
