use ethabi::ethereum_types::U256;
use serde::{Deserialize, Serialize};

use crate::types::{transaction::eip2718::TypedTransaction, BlockNumber};

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EthCallManyBlockOverride {
    pub block_number: U256,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EthCallManyOptions<'a> {
    pub block_number: &'a BlockNumber,
}

#[derive(Serialize, Debug, Clone)]
pub struct EthCallManyBalanceDiff {
    pub balance: U256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthCallManyResponse {
    pub value: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct EthCallManyBundle<T: Into<TypedTransaction> + Send + Sync> {
    pub transactions: Vec<T>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_override: Option<EthCallManyBlockOverride>,
}
