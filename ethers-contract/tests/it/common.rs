use ethers_contract::EthEvent;
use ethers_core::types::Address;

#[cfg(feature = "providers")]
use ethers_contract::{Contract, ContractFactory};
#[cfg(feature = "providers")]
use ethers_core::{
    abi::{Abi, JsonAbi},
    types::Bytes,
    utils::AnvilInstance,
};
#[cfg(feature = "providers")]
use ethers_providers::{Http, Middleware, Provider};
#[cfg(feature = "providers")]
use std::{convert::TryFrom, fs, sync::Arc, time::Duration};

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

/// Gets the contract ABI and bytecode from a JSON file
#[cfg(feature = "providers")]
#[track_caller]
pub fn get_contract(filename: &str) -> (Abi, Bytes) {
    let path = format!("./tests/solidity-contracts/{filename}");
    let contents = fs::read_to_string(path).unwrap();
    let obj: JsonAbi = serde_json::from_str(&contents).unwrap();
    let JsonAbi::Object(obj) = obj else { panic!() };
    (
        serde_json::from_str(&serde_json::to_string(&obj.abi).unwrap()).unwrap(),
        obj.bytecode.unwrap(),
    )
}

/// connects the private key to http://localhost:8545
#[cfg(feature = "providers")]
pub fn connect(anvil: &AnvilInstance, idx: usize) -> Arc<Provider<Http>> {
    let sender = anvil.addresses()[idx];
    let provider = Provider::<Http>::try_from(anvil.endpoint())
        .unwrap()
        .interval(Duration::from_millis(10u64))
        .with_sender(sender);
    Arc::new(provider)
}

/// Launches a Anvil instance and deploys the SimpleStorage contract
#[cfg(feature = "providers")]
pub async fn deploy<M: Middleware>(client: Arc<M>, abi: Abi, bytecode: Bytes) -> Contract<M> {
    let factory = ContractFactory::new(abi, bytecode, client);
    let deployer = factory.deploy("initial value".to_string()).unwrap();
    deployer.call().await.unwrap();
    deployer.legacy().send().await.unwrap()
}
