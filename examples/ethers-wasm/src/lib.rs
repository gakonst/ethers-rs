pub mod utils;

use crate::utils::SIMPLECONTRACT_BIN;
use ethers::{
    contract::abigen,
    prelude::{ContractFactory, LocalWallet, Provider, SignerMiddleware},
    providers::{Middleware, Ws},
};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use web_sys::console;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

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

/// key[0] of ganache with custom seed `ethers-wasm-seed`
pub const KEY: &str = "817169e55f14ede54f4fd6a4f2ab4209db14aeeb1b9972b3b28f1560af0a061a";

#[wasm_bindgen]
pub fn setup() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub async fn deploy() {
    console::log_2(
        &"ABI: ".into(),
        &JsValue::from_serde(&*SIMPLECONTRACT_ABI).unwrap(),
    );
    let wallet: LocalWallet = KEY.parse().unwrap();
    log!("Wallet: {:?}", wallet);

    let endpoint = "ws://127.0.0.1:8545";
    let provider = Provider::new(Ws::connect(endpoint).await.unwrap());
    let client = Arc::new(SignerMiddleware::new(provider, wallet));
    log!("provider connected to `{}`", endpoint);
    let version = client.client_version().await;
    log!("version {:?}", version);
    let account = client.get_accounts().await.unwrap()[0];
    log!("account {:?}", account);

    let bytecode = hex::decode(SIMPLECONTRACT_BIN).unwrap();
    let factory = ContractFactory::new(SIMPLECONTRACT_ABI.clone(), bytecode.into(), client.clone());
    let init = "hello WASM!";
    let contract = factory
        .deploy(init.to_string())
        .unwrap()
        .send()
        .await
        .unwrap();
    let addr = contract.address();
    log!("deployed contract with address {}", addr);

    let contract = SimpleContract::new(addr, client.clone());

    let value = contract.get_value().call().await.unwrap();
    assert_eq!(init, &value);

    let _receipt = contract
        .set_value("bye WASM!".to_owned())
        .send()
        .await
        .unwrap()
        .await
        .unwrap();
    log!("set value");
    // 10. get all events
    let logs = contract
        .value_changed_filter()
        .from_block(0u64)
        .query()
        .await
        .unwrap();

    let value = contract.get_value().call().await.unwrap();

    log!(
        "Value: {}. Logs: {:?}",
        value,
        JsValue::from_serde(&logs).unwrap()
    );
}
