use crate::types::{Address, H256};

use rlp_derive::RlpEncodable;
use serde::{Deserialize, Serialize};

/// Access list
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, RlpEncodable)]
pub struct AccessList(pub Vec<AccessListItem>);

impl From<Vec<AccessListItem>> for AccessList {
    fn from(src: Vec<AccessListItem>) -> AccessList {
        AccessList(src)
    }
}

/// Access list item
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, RlpEncodable)]
#[serde(rename_all = "camelCase")]
pub struct AccessListItem {
    /// Accessed address
    pub address: Address,
    /// Accessed storage keys
    pub storage_keys: Vec<H256>,
}
