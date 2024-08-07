use async_trait::async_trait;
use ethers_core::{types::*, utils::Anvil};
use ethers_middleware::gas_oracle::{
    BlockNative, Etherchain, GasNow, GasOracle, GasOracleError, GasOracleMiddleware, Polygon,
    ProviderOracle, Result,
};
use ethers_providers::{Http, Middleware, Provider};

#[derive(Debug)]
struct FakeGasOracle {
    gas_price: U256,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl GasOracle for FakeGasOracle {
    async fn fetch(&self) -> Result<U256> {
        Ok(self.gas_price)
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256)> {
        Err(GasOracleError::Eip1559EstimationNotSupported)
    }
}

#[tokio::test]
async fn provider_using_gas_oracle() {
    let anvil = Anvil::new().spawn();

    let from = anvil.addresses()[0];

    // connect to the network
    let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();

    // assign a gas oracle to use
    let expected_gas_price = U256::from(1234567890_u64);
    let gas_oracle = FakeGasOracle { gas_price: expected_gas_price };
    let gas_price = gas_oracle.fetch().await.unwrap();
    assert_eq!(gas_price, expected_gas_price);

    let provider = GasOracleMiddleware::new(provider, gas_oracle);

    // broadcast a transaction
    let tx = TransactionRequest::new().from(from).to(Address::zero()).value(10000);
    let tx_hash = provider.send_transaction(tx, None).await.unwrap();

    let tx = provider.get_transaction(*tx_hash).await.unwrap().unwrap();
    assert_eq!(tx.gas_price, Some(expected_gas_price));
}

#[tokio::test]
async fn provider_oracle() {
    // spawn anvil and connect to it
    let anvil = Anvil::new().spawn();
    let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();

    // assert that provider.get_gas_price() and oracle.fetch() return the same value
    let expected_gas_price = provider.get_gas_price().await.unwrap();
    let provider_oracle = ProviderOracle::new(provider);
    let gas = provider_oracle.fetch().await.unwrap();
    assert_eq!(gas, expected_gas_price);
}

#[tokio::test]
async fn blocknative() {
    let gas_now_oracle = BlockNative::default();
    let gas_price = gas_now_oracle.fetch().await.unwrap();
    assert!(gas_price > U256::zero());
}

#[tokio::test]
#[ignore = "ETHGasStation is shutting down: https://twitter.com/ETHGasStation/status/1597341610777317376"]
#[allow(deprecated)]
async fn eth_gas_station() {
    let eth_gas_station_oracle = ethers_middleware::gas_oracle::EthGasStation::default();
    let gas_price = eth_gas_station_oracle.fetch().await.unwrap();
    assert!(gas_price > U256::zero());
}

#[tokio::test]
#[ignore = "Etherchain / beaconcha.in's `gasPriceOracle` API currently returns 404: https://www.etherchain.org/api/gasPriceOracle"]
async fn etherchain() {
    let etherchain_oracle = Etherchain::default();
    let gas_price = etherchain_oracle.fetch().await.unwrap();
    assert!(gas_price > U256::zero());
}

#[cfg(feature = "etherscan")]
#[tokio::test]
async fn etherscan() {
    use ethers_core::utils::parse_ether;
    use ethers_etherscan::Client;
    use ethers_middleware::gas_oracle::{Etherscan, GasCategory};

    let chain = Chain::Mainnet;
    let etherscan_client = Client::new_from_opt_env(chain).unwrap();

    // initialize and fetch gas estimates from Etherscan
    // since etherscan does not support `fastest` category, we expect an error
    let etherscan_oracle = Etherscan::new(etherscan_client.clone()).category(GasCategory::Fastest);
    let error = etherscan_oracle.fetch().await.unwrap_err();
    assert!(matches!(error, GasOracleError::GasCategoryNotSupported));

    // but fetching the `standard` gas price should work fine
    let etherscan_oracle = Etherscan::new(etherscan_client).category(GasCategory::SafeLow);

    let gas_price = etherscan_oracle.fetch().await.unwrap();
    assert!(gas_price > U256::zero());
    let ten_ethers = parse_ether(10).unwrap();
    assert!(gas_price < ten_ethers, "gas calculation is wrong (too high)");
}

#[tokio::test]
async fn gas_now() {
    let gas_now_oracle = GasNow::default();
    let gas_price = gas_now_oracle.fetch().await.unwrap();
    assert!(gas_price > U256::zero());
}

#[tokio::test]
async fn polygon() {
    let polygon_oracle = Polygon::default();
    let gas_price = polygon_oracle.fetch().await.unwrap();
    assert!(gas_price > U256::zero());
}
