use std::sync::Arc;

use crate::spawn_anvil;
use ethers_core::types::*;
use ethers_middleware::{MiddlewareBuilder, MulticallMiddleware};
use ethers_providers::Middleware;

use ethers_contract::{
    multicall::constants::{DEPLOYER_ADDRESS, MULTICALL_ADDRESS, SIGNED_DEPLOY_MULTICALL_TX},
    multicall::contract::Multicall3,
    BaseContract,
};
use instant::Duration;

#[tokio::test]
async fn multicall() {
    let (provider, anvil) = spawn_anvil();

    provider
        .request::<(H160, U256), ()>(
            "anvil_setBalance",
            (DEPLOYER_ADDRESS, U256::from(1_000_000_000_000_000_000u64)),
        )
        .await
        .unwrap();
    provider
        .request::<[serde_json::Value; 1], H256>(
            "eth_sendRawTransaction",
            [SIGNED_DEPLOY_MULTICALL_TX.into()],
        )
        .await
        .unwrap();

    // TODO: inject some contract ABIs to call
    let contracts = vec![];
    let mut multicall_provider = MulticallMiddleware::new(
        provider,
        contracts,
        Duration::from_secs(1),
        Some(MULTICALL_ADDRESS),
    )
    .unwrap();

    // spawn the multicall middleware
    tokio::spawn(async move {
        multicall_provider.run().await;
    });

    // TODO: make some async calls and verify that only 1 RPC is made
}
