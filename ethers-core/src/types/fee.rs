use crate::types::{serde_helpers::deserialize_stringified_numeric, U256};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeeHistory {
    pub base_fee_per_gas: Vec<U256>,
    pub gas_used_ratio: Vec<f64>,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
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
