use ethers::{
    contract::abigen,
    prelude::{Provider, SignerMiddleware},
    providers::{Middleware, Ws},
    signers::Signer,
};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use web_sys::console;

pub mod utils;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

abigen!(SimpleContract, "./abi/contract.json", derives(serde::Deserialize, serde::Serialize));

#[wasm_bindgen]
pub async fn deploy() {
    utils::set_panic_hook();

    console::log_2(
        &"SimpleContract ABI: ".into(),
        &serde_wasm_bindgen::to_value(&*SIMPLECONTRACT_ABI).unwrap(),
    );

    let endpoint = "ws://127.0.0.1:8545";
    let provider = Provider::<Ws>::connect(endpoint).await.unwrap();
    log!("Connected to: `{endpoint}`");

    let chain_id = provider.get_chainid().await.unwrap();
    let wallet = utils::key(0).with_chain_id(chain_id.as_u64());
    log!("Wallet: {wallet:?}");
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    log!("Deploying contract...");
    let deploy_tx = SimpleContract::deploy(client.clone(), "hello WASM!".to_string()).unwrap();
    let contract: SimpleContract<_> = deploy_tx.send().await.unwrap();
    let addr = contract.address();
    log!("Deployed contract with address: {:?}", addr);

    let value = "bye from WASM!";
    log!("Setting value... `{}`", value);
    let receipt = contract.set_value(value.to_owned()).send().await.unwrap().await.unwrap();
    console::log_2(&"Set value receipt: ".into(), &serde_wasm_bindgen::to_value(&receipt).unwrap());

    log!("Fetching logs...");
    let logs = contract.value_changed_filter().from_block(0u64).query().await.unwrap();

    let value = contract.get_value().call().await.unwrap();

    console::log_2(
        &format!("Value: `{value}`. Logs: ").into(),
        &serde_wasm_bindgen::to_value(&logs).unwrap(),
    );
}
