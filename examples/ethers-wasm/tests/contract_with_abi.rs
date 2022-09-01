#![cfg(target_arch = "wasm32")]

use ethers::{
    prelude::{
        abigen, ContractFactory, Http, JsonRpcClient, LocalWallet, Provider, SignerMiddleware, Ws,
    },
    signers::Signer,
    types::Chain,
};
use std::{convert::TryFrom, sync::Arc};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// Generate the type-safe contract bindings by providing the ABI
// definition in human readable format
abigen!(
    SimpleContract,
    "../contract_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

#[wasm_bindgen_test]
async fn http_connect_and_deploy() {
    console_log!("connecting http...");
    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    deploy(provider, ethers_wasm::utils::key(0).with_chain_id(Chain::AnvilHardhat)).await;
}

#[wasm_bindgen_test]
async fn ws_connect_and_deploy() {
    console_log!("connecting ws...");
    let provider = Provider::new(Ws::connect("ws://localhost:8545").await.unwrap());
    deploy(provider, ethers_wasm::utils::key(1).with_chain_id(Chain::AnvilHardhat)).await;
}

async fn deploy<T: JsonRpcClient>(provider: Provider<T>, wallet: LocalWallet) {
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    let bytecode = hex::decode(ethers_wasm::utils::SIMPLECONTRACT_BIN).unwrap();
    let factory = ContractFactory::new(SIMPLECONTRACT_ABI.clone(), bytecode.into(), client.clone());
    let contract =
        factory.deploy("Hello from Contract!".to_string()).unwrap().send().await.unwrap();
    let addr = contract.address();
    console_log!("deployed to {}", addr);

    let contract = SimpleContract::new(addr, client.clone());
    let value = contract.get_value().call().await.unwrap();

    console_log!("value: {:?}", value);
}
