use crate::types::{Address, Bytes, H256, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct StorageProof {
    pub key: H256,
    pub proof: Vec<Bytes>,
    pub value: U256,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct EIP1186ProofResponse {
    address: Address,
    balance: U256,
    code_hash: H256,
    nonce: U256,
    storage_hash: H256,
    account_proof: Vec<Bytes>,
    storage_proof: Vec<StorageProof>,
}
