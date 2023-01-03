use super::{from_gwei_f64, GasCategory, GasOracle, GasOracleError, Result};
use async_trait::async_trait;
use ethers_core::types::{Chain, U256};
use reqwest::Client;
use serde::Deserialize;
use url::Url;

const MAINNET_URL: &str = "https://gasstation-mainnet.matic.network/v2";
const MUMBAI_URL: &str = "https://gasstation-mumbai.matic.today/v2";

/// The [Polygon](https://docs.polygon.technology/docs/develop/tools/polygon-gas-station/) gas station API
/// Queries over HTTP and implements the `GasOracle` trait.
#[derive(Clone, Debug)]
#[must_use]
pub struct Polygon {
    client: Client,
    url: Url,
    gas_category: GasCategory,
}

/// The response from the Polygon gas station API.
///
/// Gas prices are in __Gwei__.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub estimated_base_fee: f64,
    pub safe_low: GasEstimate,
    pub standard: GasEstimate,
    pub fast: GasEstimate,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GasEstimate {
    pub max_priority_fee: f64,
    pub max_fee: f64,
}

impl Response {
    #[inline]
    pub fn estimate_from_category(&self, gas_category: GasCategory) -> GasEstimate {
        match gas_category {
            GasCategory::SafeLow => self.safe_low,
            GasCategory::Standard => self.standard,
            GasCategory::Fast => self.fast,
            GasCategory::Fastest => self.fast,
        }
    }
}

impl Default for Polygon {
    fn default() -> Self {
        Self::new(Chain::Polygon).unwrap()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl GasOracle for Polygon {
    async fn fetch(&self) -> Result<U256> {
        let response = self.query().await?;
        let base = response.estimated_base_fee;
        let prio = response.estimate_from_category(self.gas_category).max_priority_fee;
        let fee = base + prio;
        Ok(from_gwei_f64(fee))
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256)> {
        let response = self.query().await?;
        let estimate = response.estimate_from_category(self.gas_category);
        let max = from_gwei_f64(estimate.max_fee);
        let prio = from_gwei_f64(estimate.max_priority_fee);
        Ok((max, prio))
    }
}

impl Polygon {
    pub fn new(chain: Chain) -> Result<Self> {
        Self::with_client(Client::new(), chain)
    }

    pub fn with_client(client: Client, chain: Chain) -> Result<Self> {
        // TODO: Sniff chain from chain id.
        let url = match chain {
            Chain::Polygon => MAINNET_URL,
            Chain::PolygonMumbai => MUMBAI_URL,
            _ => return Err(GasOracleError::UnsupportedChain),
        };
        Ok(Self { client, url: Url::parse(url).unwrap(), gas_category: GasCategory::Standard })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_polygon_gas_station_response() {
        let s = r#"{"safeLow":{"maxPriorityFee":2.1267086610666666,"maxFee":2.1267086760666665},"standard":{"maxPriorityFee":2.3482958369333335,"maxFee":2.3482958519333335},"fast":{"maxPriorityFee":2.793454819,"maxFee":2.793454834},"estimatedBaseFee":1.5e-8,"blockTime":2,"blockNumber":30328888}"#;
        let _resp: Response = serde_json::from_str(s).unwrap();
    }
}
