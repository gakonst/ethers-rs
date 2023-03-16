use crate::{Client, EtherscanError, Response, Result};
use ethers_core::types::U256;
use serde::{de, Deserialize, Deserializer};
use std::{collections::HashMap, str::FromStr};

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct GasOracle {
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

fn deserialize_number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: std::fmt::Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt<T> {
        String(String),
        Number(T),
    }

    match StringOrInt::<T>::deserialize(deserializer)? {
        StringOrInt::String(s) => s.parse::<T>().map_err(serde::de::Error::custom),
        StringOrInt::Number(i) => Ok(i),
    }
}

fn deserialize_f64_vec<'de, D>(deserializer: D) -> core::result::Result<Vec<f64>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let str_sequence = String::deserialize(deserializer)?;
    str_sequence
        .split(',')
        .map(|item| f64::from_str(item).map_err(|err| de::Error::custom(err.to_string())))
        .collect()
}

impl Client {
    /// Returns the estimated time, in seconds, for a transaction to be confirmed on the blockchain
    /// for the specified gas price
    pub async fn gas_estimate(&self, gas_price: U256) -> Result<u32> {
        let query = self.create_query(
            "gastracker",
            "gasestimate",
            HashMap::from([("gasprice", gas_price.to_string())]),
        );
        let response: Response<String> = self.get_json(&query).await?;

        if response.status == "1" {
            Ok(u32::from_str(&response.result).map_err(|_| EtherscanError::GasEstimationFailed)?)
        } else {
            Err(EtherscanError::GasEstimationFailed)
        }
    }

    /// Returns the current Safe, Proposed and Fast gas prices
    /// Post EIP-1559 changes:
    /// - Safe/Proposed/Fast gas price recommendations are now modeled as Priority Fees.
    /// - New field `suggestBaseFee`, the baseFee of the next pending block
    /// - New field `gasUsedRatio`, to estimate how busy the network is
    pub async fn gas_oracle(&self) -> Result<GasOracle> {
        let query = self.create_query("gastracker", "gasoracle", serde_json::Value::Null);
        let response: Response<GasOracle> = self.get_json(&query).await?;

        Ok(response.result)
    }
}
