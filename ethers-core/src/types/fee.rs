use std::str::FromStr;

use crate::types::U256;
use serde::{de::Deserializer, Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeeHistory {
    pub base_fee_per_gas: Vec<U256>,
    pub gas_used_ratio: Vec<f64>,
    #[serde(deserialize_with = "from_int_or_hex")]
    /// oldestBlock is returned as an unsigned integer up to geth v1.10.6. From
    /// geth v1.10.7, this has been updated to return in the hex encoded form.
    /// The custom deserializer allows backward compatibility for those clients
    /// not running v1.10.7 yet.
    pub oldest_block: U256,
    /// An (optional) array of effective priority fee per gas data points from a single block. All
    /// zeroes are returned if the block is empty.
    #[serde(default)]
    pub reward: Vec<Vec<U256>>,
}

fn from_int_or_hex<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IntOrHex {
        Int(u64),
        Hex(String),
    }
    match IntOrHex::deserialize(deserializer)? {
        IntOrHex::Int(n) => Ok(U256::from(n)),
        IntOrHex::Hex(s) => U256::from_str(s.as_str()).map_err(serde::de::Error::custom),
    }
}
