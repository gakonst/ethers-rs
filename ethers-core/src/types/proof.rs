use crate::types::{Address, Bytes, H256, U256, U64};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct StorageProof {
    pub key: H256,
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
}
