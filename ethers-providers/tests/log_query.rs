#![cfg(not(target_arch = "wasm32"))]
use ethers_core::{
    types::{Filter, BlockNumber},
    utils::Anvil,
};
use ethers_providers::{Http, Middleware, Provider, StreamExt};
use std::convert::TryFrom;

#[tokio::test]
async fn get_logs_paginated() {
    let geth = Anvil::new().block_time(20u64).spawn();
    let provider = Provider::<Http>::try_from(geth.endpoint()).unwrap();

    let filter = Filter::new().from_block(BlockNumber::Latest);
    // try to get beyond the number of blocks available
    let mut stream = provider.get_logs_paginated(&filter, 10);
    let res = stream.next().await;
    assert!(res.is_some());
    let log = res.unwrap();
    assert!(log.is_ok());
}
