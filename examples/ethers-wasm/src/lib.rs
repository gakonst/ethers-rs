use crate::utils::SIMPLECONTRACT_BIN;
use ethers::{
    contract::abigen,
    prelude::{ContractFactory, Provider, SignerMiddleware},
    providers::Ws,
};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use web_sys::console;

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

#[wasm_bindgen]
pub async fn deploy() {
    utils::set_panic_hook();

    console::log_2(
        &"SimpleContract ABI: ".into(),
        &serde_wasm_bindgen::to_value(&*SIMPLECONTRACT_ABI).unwrap(),
    );

    let wallet = utils::key(0);
    log!("Wallet: {:?}", wallet);

    let endpoint = "ws://127.0.0.1:8545";
    let provider = Provider::<Ws>::connect(endpoint).await.unwrap();
    let client = Arc::new(SignerMiddleware::new(provider, wallet));
    log!("Connected to: `{}`", endpoint);

    let bytecode = hex::decode(SIMPLECONTRACT_BIN).unwrap();
    let factory = ContractFactory::new(SIMPLECONTRACT_ABI.clone(), bytecode.into(), client.clone());

    log!("Deploying contract...");
    let contract = factory.deploy("hello WASM!".to_string()).unwrap().send().await.unwrap();
    let addr = contract.address();
    log!("Deployed contract with address: {:?}", addr);

    let contract = SimpleContract::new(addr, client.clone());

    let value = "bye from WASM!";
    log!("Setting value... `{}`", value);
    let receipt = contract.set_value(value.to_owned()).send().await.unwrap().await.unwrap();
    console::log_2(&"Set value receipt: ".into(), &serde_wasm_bindgen::to_value(&receipt).unwrap());

    log!("Fetching logs...");
    let logs = contract.value_changed_filter().from_block(0u64).query().await.unwrap();

    let value = contract.get_value().call().await.unwrap();

    console::log_2(
        &format!("Value: `{}`. Logs: ", value).into(),
        &serde_wasm_bindgen::to_value(&logs).unwrap(),
    );
}
