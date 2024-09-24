use crate::types::{Address, BlockNumber, H256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extra options parameter for `eth_sendRawTransactionConditional`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionalOptions {
    /// A map of accounts with expected storage
    #[serde(rename = "knownAccounts")]
    pub known_accounts: HashMap<Address, AccountStorage>,

    /// Minimal block number for inclusion
    #[serde(rename = "blockNumberMin", skip_serializing_if = "Option::is_none")]
    pub block_number_min: Option<BlockNumber>,

    /// Maximum block number for inclusion
    #[serde(rename = "blockNumberMax", skip_serializing_if = "Option::is_none")]
    pub block_number_max: Option<BlockNumber>,

    /// Minimal block timestamp for inclusion
    #[serde(rename = "timestampMin", skip_serializing_if = "Option::is_none")]
    pub timestamp_min: Option<u64>,

    /// Maximum block timestamp for inclusion
    #[serde(rename = "timestampMax", skip_serializing_if = "Option::is_none")]
    pub timestamp_max: Option<u64>,
}

/// Account storage
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AccountStorage {
    RootHash(H256),
    SlotValues(HashMap<String, String>),
}
