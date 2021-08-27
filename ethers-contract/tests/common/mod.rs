#![cfg(not(target_arch = "wasm32"))]

#[cfg(feature = "abigen")]
use ethers_core::types::Address;

#[cfg(feature = "abigen")]
use ethers_contract::EthEvent;

#[cfg(feature = "abigen")]
mod derive;

use ethers_contract::{Contract, ContractFactory};
use ethers_core::utils::{GanacheInstance, Solc};
use ethers_core::{abi::Abi, types::Bytes};
use ethers_providers::{Http, Middleware, Provider};
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
pub fn compile_contract(name: &str, filename: &str) -> (Abi, Bytes) {
    let compiled = Solc::new(&format!("./tests/solidity-contracts/{}", filename))
        .build()
        .unwrap();
    let contract = compiled.get(name).expect("could not find contract");
    (contract.abi.clone(), contract.bytecode.clone())
}

/// connects the private key to http://localhost:8545
pub fn connect(ganache: &GanacheInstance, idx: usize) -> Arc<Provider<Http>> {
    let sender = ganache.addresses()[idx];
    let provider = Provider::<Http>::try_from(ganache.endpoint())
        .unwrap()
        .interval(Duration::from_millis(10u64))
        .with_sender(sender);
    Arc::new(provider)
}

/// Launches a ganache instance and deploys the SimpleStorage contract
pub async fn deploy<M: Middleware>(client: Arc<M>, abi: Abi, bytecode: Bytes) -> Contract<M> {
    let factory = ContractFactory::new(abi, bytecode, client);
    factory
        .deploy("initial value".to_string())
        .unwrap()
        .legacy()
        .send()
        .await
        .unwrap()
}
