use crate::types::{
    serde_helpers::deserialize_stringified_numeric, Address, Bytes, H256, U256, U64,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct StorageProof {
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub key: U256,
    pub proof: Vec<Bytes>,
    pub value: U256,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EIP1186ProofResponse {
    pub address: Address,
    pub balance: U256,
    pub code_hash: H256,
    pub nonce: U64,
    pub storage_hash: H256,
    pub account_proof: Vec<Bytes>,
    pub storage_proof: Vec<StorageProof>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_deserialize_proof() {
        serde_json::from_str::<EIP1186ProofResponse>(include_str!("../../testdata/proof.json"))
            .unwrap();
    }

    #[test]
    fn can_deserialize_proof_uint_key() {
        serde_json::from_str::<EIP1186ProofResponse>(include_str!(
            "../../testdata/proof_uint_key.json"
        ))
        .unwrap();
    }

    #[test]
    fn can_deserialize_proof_empty_key() {
        serde_json::from_str::<EIP1186ProofResponse>(include_str!(
            "../../testdata/proof_empty_key.json"
        ))
        .unwrap();
    }
}
