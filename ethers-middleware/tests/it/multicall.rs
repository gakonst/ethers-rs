use crate::spawn_anvil;
use ethers_core::types::*;
use ethers_middleware::{MiddlewareBuilder, MulticallMiddleware};
use ethers_providers::Middleware;

use ethers_contract::multicall::constants::{DEPLOY_MULTICALL_TX, MULTICALL_ADDRESS};
use instant::Duration;

#[tokio::test]
async fn multicall() {
    let (provider, anvil) = spawn_anvil();

    let tx_bytes: Bytes = DEPLOY_MULTICALL_TX.into();

    provider.send_raw_transaction(tx_bytes).await.unwrap();

    let contracts = Vec::new();

    let mut multicall_provider = MulticallMiddleware::new(
        provider,
        contracts,
        Duration::from_secs(1),
        Some(MULTICALL_ADDRESS),
    ).unwrap();

    // spawn the multicall middleware
    tokio::spawn(async move {
        multicall_provider.run().await;
    });
}
