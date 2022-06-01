//! Main entry point for ContractMonitor

use ethers::{prelude::*, utils::Anvil};
use std::{convert::TryFrom, sync::Arc, time::Duration};

abigen!(VerifierContract, "ethers-contract/tests/solidity-contracts/verifier_abi.json");

/// This example only demonstrates how to use generated structs for solidity functions that
/// have structs as input.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let anvil = Anvil::new().spawn();
    let provider =
        Provider::<Http>::try_from(anvil.endpoint())?.interval(Duration::from_millis(10u64));
    let wallet: LocalWallet = anvil.keys()[0].clone().into();

    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    let contract = VerifierContract::new(Address::zero(), client);

    // NOTE: this is all just dummy data
    let g1 = G1Point { x: U256::zero(), y: U256::zero() };
    let g2 = G2Point { x: [U256::zero(), U256::zero()], y: [U256::zero(), U256::zero()] };
    let vk = VerifyingKey {
        alfa_1: g1.clone(),
        beta_2: g2.clone(),
        gamma_2: g2.clone(),
        delta_2: g2.clone(),
        ic: vec![g1.clone()],
    };
    let proof = Proof { a: g1.clone(), b: g2, c: g1 };

    let _ = contract.verify(vec![], proof, vk);
    Ok(())
}
