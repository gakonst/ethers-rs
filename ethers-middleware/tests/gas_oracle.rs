use ethers_core::{types::*, utils::Ganache};
use ethers_middleware::gas_oracle::{
    EthGasStation, Etherchain, Etherscan, GasCategory, GasNow, GasOracle, GasOracleMiddleware,
};
use ethers_providers::{Http, Middleware, Provider};
use std::convert::TryFrom;

#[tokio::test]
async fn using_gas_oracle() {
    let ganache = Ganache::new().spawn();

    let from = ganache.addresses()[0];

    // connect to the network
    let provider = Provider::<Http>::try_from(ganache.endpoint()).unwrap();

    // assign a gas oracle to use
    let gas_oracle = GasNow::new().category(GasCategory::Fastest);
    let expected_gas_price = gas_oracle.fetch().await.unwrap();

    let provider = GasOracleMiddleware::new(provider, gas_oracle);

    // broadcast a transaction
    let tx = TransactionRequest::new()
        .from(from)
        .to(Address::zero())
        .value(10000);
    let tx_hash = provider.send_transaction(tx, None).await.unwrap();

    let tx = provider.get_transaction(*tx_hash).await.unwrap().unwrap();
    assert_eq!(tx.gas_price, expected_gas_price);
}

#[tokio::test]
#[ignore]
// TODO: Re-enable, EthGasStation changed its response api @ https://ethgasstation.info/api/ethgasAPI.json
async fn eth_gas_station() {
    // initialize and fetch gas estimates from EthGasStation
    let eth_gas_station_oracle = EthGasStation::new(None);
    let data = eth_gas_station_oracle.fetch().await;
    assert!(data.is_ok());
}

#[tokio::test]
async fn etherscan() {
    let api_key = std::env::var("ETHERSCAN_API_KEY").unwrap();
    let api_key = Some(api_key.as_str());

    // initialize and fetch gas estimates from Etherscan
    // since etherscan does not support `fastest` category, we expect an error
    let etherscan_oracle = Etherscan::new(api_key).category(GasCategory::Fastest);
    let data = etherscan_oracle.fetch().await;
    assert!(data.is_err());

    // but fetching the `standard` gas price should work fine
    let etherscan_oracle_2 = Etherscan::new(api_key).category(GasCategory::SafeLow);

    let data = etherscan_oracle_2.fetch().await;
    assert!(data.is_ok());
}

#[tokio::test]
#[ignore]
// TODO: Etherchain has Cloudflare DDOS protection which makes the request fail
// https://twitter.com/gakonst/status/1421796226316578816
async fn etherchain() {
    // initialize and fetch gas estimates from Etherchain
    let etherchain_oracle = Etherchain::new().category(GasCategory::Fast);
    let data = etherchain_oracle.fetch().await;
    assert!(data.is_ok());
}

#[tokio::test]
async fn gas_now() {
    // initialize and fetch gas estimates from SparkPool
    let gas_now_oracle = GasNow::new().category(GasCategory::Fastest);
    let data = gas_now_oracle.fetch().await;
    assert!(data.is_ok());
}
