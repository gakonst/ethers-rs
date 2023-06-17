use super::{from_gwei_f64, GasCategory, GasOracle, GasOracleError, Result};
use async_trait::async_trait;
use ethers_core::types::{Chain, U256};
use reqwest::Client;
use serde::Deserialize;
use url::Url;

const MAINNET_URL: &str = "https://gasstation.polygon.technology/v2";
const MUMBAI_URL: &str = "https://gasstation-testnet.polygon.technology/v2";

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
    #[serde(deserialize_with = "deserialize_stringified_f64")]
    pub estimated_base_fee: f64,
    pub safe_low: GasEstimate,
    pub standard: GasEstimate,
    pub fast: GasEstimate,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GasEstimate {
    #[serde(deserialize_with = "deserialize_stringified_f64")]
    pub max_priority_fee: f64,
    #[serde(deserialize_with = "deserialize_stringified_f64")]
    pub max_fee: f64,
}

fn deserialize_stringified_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum F64OrString {
        F64(serde_json::Number),
        String(String),
    }
    match Deserialize::deserialize(deserializer)? {
        F64OrString::F64(f) => f.as_f64().ok_or_else(|| serde::de::Error::custom("invalid f64")),
        F64OrString::String(s) => s.parse().map_err(serde::de::Error::custom),
    }
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
        #[cfg(not(target_arch = "wasm32"))]
        static APP_USER_AGENT: &str =
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

        let builder = Client::builder();
        #[cfg(not(target_arch = "wasm32"))]
        let builder = builder.user_agent(APP_USER_AGENT);

        Self::with_client(builder.build()?, chain)
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
        let s = r#"{"safeLow":{"maxPriorityFee":"30.739827732","maxFee":"335.336914674"},"standard":{"maxPriorityFee":"57.257993430","maxFee":"361.855080372"},"fast":{"maxPriorityFee":"103.414268558","maxFee":"408.011355500"},"estimatedBaseFee":"304.597086942","blockTime":2,"blockNumber":43975155}"#;
        let _resp: Response = serde_json::from_str(s).unwrap();
    }

    #[test]
    fn parse_polygon_testnet_gas_station_response() {
        let s = r#"{"safeLow":{"maxPriorityFee":1.3999999978,"maxFee":1.4000000157999999},"standard":{"maxPriorityFee":1.5199999980666665,"maxFee":1.5200000160666665},"fast":{"maxPriorityFee":2.0233333273333334,"maxFee":2.0233333453333335},"estimatedBaseFee":1.8e-8,"blockTime":2,"blockNumber":36917340}"#;
        let _resp: Response = serde_json::from_str(s).unwrap();
    }
}
