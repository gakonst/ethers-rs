//! This is a basic representation of a contract ABI that does no post processing but contains the raw content of the ABI.

#![allow(missing_docs)]
use serde::{Deserialize, Serialize};

/// Contract ABI as a list of items where each item can be a function, constructor or event
pub type RawAbi = Vec<Item>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(default)]
    pub inputs: Vec<Component>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_mutability: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub outputs: Vec<Component>,
}

/// Either
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    #[serde(
        rename = "internalType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub internal_type: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub components: Vec<Component>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_parse_raw_abi() {
        const VERIFIER_ABI: &str = include_str!("../../tests/solidity-contracts/verifier_abi.json");
        let _ = serde_json::from_str::<RawAbi>(VERIFIER_ABI).unwrap();
    }
}
