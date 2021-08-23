//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use ethers::{
    contract::abigen,
    prelude::{ContractFactory, LocalWallet, Provider, SignerMiddleware},
    providers::Ws,
};

use std::sync::Arc;

wasm_bindgen_test_configure!(run_in_browser);

// Generate the type-safe contract bindings by providing the ABI
// definition in human readable format
abigen!(
    SimpleContract,
    "../contract_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

#[wasm_bindgen_test]
async fn connect_and_deploy() {
    console_log!("starting");

    // a private key of a launched ganache `yarn ganache`
    let wallet: LocalWallet = ethers_wasm::KEY.parse().unwrap();

    let provider = Provider::new(Ws::connect("ws://localhost:8545").await.unwrap());
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    let bytecode = hex::decode(ethers_wasm::utils::SIMPLECONTRACT_BIN).unwrap();
    let factory = ContractFactory::new(SIMPLECONTRACT_ABI.clone(), bytecode.into(), client.clone());
    let contract = factory
        .deploy("initial value".to_string())
        .unwrap()
        .send()
        .await
        .unwrap();
    let addr = contract.address();
    console_log!("deployed to {}", addr);

    let contract = SimpleContract::new(addr, client.clone());
    let _receipt = contract
        .set_value("hi".to_owned())
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    //  get all events
    let logs = contract
        .value_changed_filter()
        .from_block(0u64)
        .query()
        .await
        .unwrap();

    let value = contract.get_value().call().await.unwrap();

    console_log!(
        "Value: {}. Logs: {:?}",
        value,
        JsValue::from_serde(&logs).unwrap()
    );
}
