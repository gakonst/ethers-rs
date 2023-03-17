#![cfg(target_arch = "wasm32")]

use ethers::{
    prelude::{Http, JsonRpcClient, LocalWallet, Provider, SignerMiddleware, Ws},
    signers::Signer,
    types::Chain,
};
use ethers_wasm::{utils, SimpleContract};
use std::sync::Arc;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn http_connect_and_deploy() {
    console_log!("connecting http...");
    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    deploy(provider, utils::key(0).with_chain_id(Chain::AnvilHardhat)).await;
}

#[wasm_bindgen_test]
async fn ws_connect_and_deploy() {
    console_log!("connecting ws...");
    let provider = Provider::new(Ws::connect("ws://localhost:8545").await.unwrap());
    deploy(provider, utils::key(1).with_chain_id(Chain::AnvilHardhat)).await;
}

async fn deploy<T: JsonRpcClient>(provider: Provider<T>, wallet: LocalWallet) {
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    let expected = "Hello from Contract!";
    let deploy_tx = SimpleContract::deploy(client, expected.to_string()).unwrap();
    let contract: SimpleContract<_> = deploy_tx.send().await.unwrap();
    let addr = contract.address();
    console_log!("deployed to {addr}");

    let value = contract.get_value().call().await.unwrap();

    console_log!("value: {value:?}");

    assert_eq!(value, expected);
}
