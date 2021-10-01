use derive_eip712::*;
use ethers_core::types::{transaction::eip712::Eip712, H160};
use serde::Serialize;

#[derive(Debug, Eip712, Serialize)]
#[eip712(
    name = "Radicle",
    version = "1",
    chain_id = 1,
    verifying_contract = "0x0000000000000000000000000000000000000000"
)]
pub struct Puzzle {
    pub organization: H160,
    pub contributor: H160,
    pub commit: String,
    pub project: String,
}

#[test]
fn test_derive_eip712() {
    let puzzle = Puzzle {
        organization: "0000000000000000000000000000000000000000"
            .parse::<H160>()
            .expect("failed to parse address"),
        contributor: "0000000000000000000000000000000000000000"
            .parse::<H160>()
            .expect("failed to parse address"),
        commit: "5693b7019eb3e4487a81273c6f5e1832d77acb53".to_string(),
        project: "radicle-reward".to_string(),
    };

    let hash = puzzle.encode_eip712().expect("failed to encode struct");

    // TODO: Compare against solidity computed hash

    println!("Hash: {:?}", hash);

    assert_eq!(hash.len(), 64)
}
