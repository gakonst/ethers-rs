#![cfg(not(target_arch = "wasm32"))]

#[cfg(feature = "abigen")]
use ethers_core::types::Address;

#[cfg(feature = "abigen")]
use ethers_contract::EthEvent;

#[cfg(feature = "abigen")]
mod derive;

use ethers_contract::{Contract, ContractFactory};
use ethers_core::{abi::Abi, types::Bytes, utils::AnvilInstance};
use ethers_providers::{Http, Middleware, Provider};
use ethers_solc::Solc;
use std::{convert::TryFrom, sync::Arc, time::Duration};

// Note: The `EthEvent` derive macro implements the necessary conversion between `Tokens` and
// the struct
#[cfg(feature = "abigen")]
#[derive(Clone, Debug, EthEvent)]
pub struct ValueChanged {
    #[ethevent(indexed)]
    pub old_author: Address,
    #[ethevent(indexed)]
    pub new_author: Address,
    pub old_value: String,
    pub new_value: String,
}

/// compiles the given contract and returns the ABI and Bytecode
#[track_caller]
pub fn compile_contract(name: &str, filename: &str) -> (Abi, Bytes) {
    let path = format!("./tests/solidity-contracts/{filename}");
    let compiled = Solc::default().compile_source(&path).unwrap();
    let contract = compiled.get(&path, name).expect("could not find contract");
    let (abi, bin, _) = contract.into_parts_or_default();
    (abi, bin)
}

/// connects the private key to http://localhost:8545
pub fn connect(anvil: &AnvilInstance, idx: usize) -> Arc<Provider<Http>> {
    let sender = anvil.addresses()[idx];
    let provider = Provider::<Http>::try_from(anvil.endpoint())
        .unwrap()
        .interval(Duration::from_millis(10u64))
        .with_sender(sender);
    Arc::new(provider)
}

/// Launches a Anvil instance and deploys the SimpleStorage contract
pub async fn deploy<M: Middleware>(client: Arc<M>, abi: Abi, bytecode: Bytes) -> Contract<M> {
    let factory = ContractFactory::new(abi, bytecode, client);
    let deployer = factory.deploy("initial value".to_string()).unwrap();
    deployer.call().await.unwrap();
    deployer.legacy().send().await.unwrap()
}
