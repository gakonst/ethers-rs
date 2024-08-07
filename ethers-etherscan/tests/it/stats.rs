use crate::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn eth_supply_success() {
    run_with_client(Chain::Mainnet, |client| async move {
        let result = client.eth_supply().await;

        result.unwrap();
    })
    .await
}

#[tokio::test]
#[serial]
async fn eth_supply2_success() {
    run_with_client(Chain::Mainnet, |client| async move {
        let result = client.eth_supply2().await;

        result.unwrap();
    })
    .await
}

#[tokio::test]
#[serial]
async fn eth_price_success() {
    run_with_client(Chain::Mainnet, |client| async move {
        let result = client.eth_price().await;

        result.unwrap();
    })
    .await
}

#[tokio::test]
#[serial]
async fn node_count_success() {
    run_with_client(Chain::Mainnet, |client| async move {
        let result = client.node_count().await;

        result.unwrap();
    })
    .await
}
