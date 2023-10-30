//! Etherscan integration tests

#![cfg(not(target_arch = "wasm32"))]

use ethers_core::types::Chain;
use ethers_etherscan::{errors::EtherscanError, Client};
use std::{
    future::Future,
    time::{Duration, Instant},
};

mod account;
mod blocks;
mod contract;
mod gas;
mod stats;
mod transaction;
mod verify;
mod version;

#[tokio::test]
async fn check_wrong_etherscan_api_key() {
    let client = Client::new(Chain::Mainnet, "ABCDEFG").unwrap();
    let resp = client
        .contract_source_code("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
        .await
        .unwrap_err();

    assert!(matches!(resp, EtherscanError::InvalidApiKey));
}

/// Calls the function with a new Etherscan Client.
pub async fn run_with_client<F, Fut, T>(chain: Chain, f: F) -> T
where
    F: FnOnce(Client) -> Fut,
    Fut: Future<Output = T>,
{
    init_tracing();
    let (client, duration) = match Client::new_from_env(chain) {
        Ok(c) => (c, rate_limit(chain, true)),
        Err(_) => {
            (Client::builder().chain(chain).unwrap().build().unwrap(), rate_limit(chain, false))
        }
    };
    run_at_least_duration(duration, f(client)).await
}

#[track_caller]
fn rate_limit(chain: Chain, key: bool) -> Duration {
    match (chain, key) {
        // Rate limit with an API key is 5 call per second.
        (_, true) => Duration::from_millis(250),

        // Rate limit without an API key is 1 call every 5 seconds.
        // (Chain::Mainnet, false) => Duration::from_millis(5100),
        (Chain::Mainnet, false) => panic!("ETHERSCAN_API_KEY is not set"),

        // Ignore other chains since we don't have more than 1 test with each.
        (_, false) => Duration::ZERO,
    }
}

async fn run_at_least_duration<T>(duration: Duration, block: impl Future<Output = T>) -> T {
    let start = Instant::now();
    let output = block.await;
    let elapsed = start.elapsed();
    if elapsed < duration {
        tokio::time::sleep(duration - elapsed).await;
    }
    output
}

#[track_caller]
fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}
