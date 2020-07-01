use ethers_core::{
    abi::{Abi, Detokenize, InvalidOutputType, Token},
    types::{Address, Bytes},
};

use ethers_contract::{Contract, ContractFactory};
use ethers_core::utils::{GanacheInstance, Solc};
use ethers_providers::{Http, Provider};
use ethers_signers::{Client, Wallet};
use std::{convert::TryFrom, sync::Arc, time::Duration};

// Note: We also provide the `abigen` macro for generating these bindings automatically
#[derive(Clone, Debug)]
pub struct ValueChanged {
    pub old_author: Address,
    pub new_author: Address,
    pub old_value: String,
    pub new_value: String,
}

impl Detokenize for ValueChanged {
    fn from_tokens(tokens: Vec<Token>) -> Result<ValueChanged, InvalidOutputType> {
        let old_author: Address = tokens[1].clone().to_address().unwrap();
        let new_author: Address = tokens[1].clone().to_address().unwrap();
        let old_value = tokens[2].clone().to_string().unwrap();
        let new_value = tokens[3].clone().to_string().unwrap();

        Ok(Self {
            old_author,
            new_author,
            old_value,
            new_value,
        })
    }
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
pub fn connect(ganache: &GanacheInstance, idx: usize) -> Arc<Client<Http, Wallet>> {
    let provider = Provider::<Http>::try_from(ganache.endpoint()).unwrap();
    let wallet: Wallet = ganache.keys()[idx].clone().into();
    Arc::new(
        wallet
            .connect(provider)
            .interval(Duration::from_millis(10u64)),
    )
}

/// Launches a ganache instance and deploys the SimpleStorage contract
pub async fn deploy(
    client: Arc<Client<Http, Wallet>>,
    abi: Abi,
    bytecode: Bytes,
) -> Contract<Http, Wallet> {
    let factory = ContractFactory::new(abi, bytecode, client);
    factory
        .deploy("initial value".to_string())
        .unwrap()
        .send()
        .await
        .unwrap()
}
