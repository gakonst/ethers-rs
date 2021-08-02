mod derive;
use ethers_core::{
    abi::Abi,
    types::{Address, Bytes},
};

use ethers_contract::{Contract, ContractFactory, EthEvent};
use ethers_core::utils::{GanacheInstance, Solc};
use ethers_middleware::signer::SignerMiddleware;
use ethers_providers::{Http, Middleware, Provider};
use ethers_signers::{LocalWallet, Signer};
use std::{convert::TryFrom, sync::Arc, time::Duration};

// Note: The `EthEvent` derive macro implements the necessary conversion between `Tokens` and
// the struct
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

type HttpWallet = SignerMiddleware<Provider<Http>, LocalWallet>;

/// connects the private key to http://localhost:8545
pub fn connect(ganache: &GanacheInstance, idx: usize) -> Arc<HttpWallet> {
    let provider = Provider::<Http>::try_from(ganache.endpoint())
        .unwrap()
        .interval(Duration::from_millis(10u64));
    let wallet: LocalWallet = ganache.keys()[idx].clone().into();
    let wallet = wallet.with_chain_id(1u64);
    Arc::new(SignerMiddleware::new(provider, wallet))
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
