use std::sync::Arc;

use wasm_bindgen::prelude::*;
use web_sys::console;

use ethers::{
    contract::abigen,
    prelude::{ContractFactory, LocalWallet, Provider, SignerMiddleware},
    providers::Ws,
};

use crate::utils::SIMPLECONTRACT_BIN;

pub mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
abigen!(
    SimpleContract,
    "./../contract_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

/// some keys of ganache with custom seed `ethers-wasm-seed`
pub const KEYS: [&str; 4] = [
    "817169e55f14ede54f4fd6a4f2ab4209db14aeeb1b9972b3b28f1560af0a061a",
    "375715b8ced8bd9b7386ba5dc72efa518aa4379d6a4676d3e26d8f5ff5e7469c",
    "de7c5d8e884fbe9f0915703ff2c123f4cda56f148eb22ca46d47392acf52bcec",
    "0bd6bf22f84f96b39258a46ac2a7c482d0b8e1c5f8af0c07fa304a8d875158ec",
];

#[wasm_bindgen]
pub async fn deploy() {
    utils::set_panic_hook();

    console::log_2(
        &"SimpleContract ABI: ".into(),
        &JsValue::from_serde(&*SIMPLECONTRACT_ABI).unwrap(),
    );

    let wallet: LocalWallet = KEYS[0].parse().unwrap();
    log!("Wallet: {:?}", wallet);

    let endpoint = "ws://127.0.0.1:8545";
    let provider = Provider::new(Ws::connect(endpoint).await.unwrap());
    let client = Arc::new(SignerMiddleware::new(provider, wallet));
    log!("Provider connected to `{}`", endpoint);

    let bytecode = hex::decode(SIMPLECONTRACT_BIN).unwrap();
    let factory = ContractFactory::new(SIMPLECONTRACT_ABI.clone(), bytecode.into(), client.clone());

    log!("Deploying contract...");
    let contract = factory
        .deploy("hello WASM!".to_string())
        .unwrap()
        .send()
        .await
        .unwrap();
    let addr = contract.address();
    log!("Deployed contract with address: {:?}", addr);

    let contract = SimpleContract::new(addr, client.clone());

    let value = "bye from WASM!";
    log!("Setting value... `{}`", value);
    let receipt = contract
        .set_value(value.to_owned())
        .send()
        .await
        .unwrap()
        .await
        .unwrap();
    console::log_2(
        &"Set value receipt: ".into(),
        &JsValue::from_serde(&receipt).unwrap(),
    );

    log!("Fetching logs...");
    let logs = contract
        .value_changed_filter()
        .from_block(0u64)
        .query()
        .await
        .unwrap();

    let value = contract.get_value().call().await.unwrap();

    console::log_2(
        &format!("Value: `{}`. Logs: ", value).into(),
        &JsValue::from_serde(&logs).unwrap(),
    );
}
