use ethabi::ethereum_types::U256;
use serde::{Deserialize, Serialize};

use crate::types::{transaction::eip2718::TypedTransaction, BlockNumber, Bytes};

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EthCallManyBlockOverride {
    pub block_number: U256,
}

#[derive(Serialize, Debug, Clone)]
pub struct EthCallManyBundle {
    pub transactions: Vec<TypedTransaction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_override: Option<EthCallManyBlockOverride>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EthCallManyStateContext {
    pub block_number: BlockNumber,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_index: Option<i32>,
}

#[derive(Serialize, Debug, Clone)]
pub struct EthCallManyBalanceDiff {
    pub balance: U256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthCallManyOutputEmpty {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EthCallManyOutput {
    pub value: Option<Bytes>,
    pub error: Option<EthCallManyOutputEmpty>,
}
