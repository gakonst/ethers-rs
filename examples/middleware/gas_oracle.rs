use ethers::types::Chain;
use ethers_etherscan::Client;
use ethers_middleware::gas_oracle::{
    BlockNative, EthGasStation, Etherchain, Etherscan, GasCategory, GasNow, GasOracle, Polygon, ProviderOracle,
};
use ethers_providers::{Provider, Http};

#[tokio::main]
async fn main() {
    blocknative().await;
    etherchain().await;
    eth_gas_station().await;
    etherscan().await;
    gas_now().await;
    polygon().await;
    provider_oracle().await;
}

async fn blocknative() {
    let api_key: String = "YOUR-API-KEY".into();
    let oracle = BlockNative::new(api_key).category(GasCategory::Fastest);
    match oracle.fetch().await {
        Ok(gas_price) => println!("Gas price is {gas_price:?}"),
        Err(e) => println!("Cannot estimate gas: {e:?}"),
    }
}

async fn etherchain() {
    let oracle = Etherchain::new().category(GasCategory::Standard);
    match oracle.fetch().await {
        Ok(gas_price) => println!("Gas price is {gas_price:?}"),
        Err(e) => println!("Cannot estimate gas: {e:?}"),
    }
}

async fn eth_gas_station() {
    let api_key: Option<&str> = Some("YOUR-API-KEY");
    let oracle = EthGasStation::new(api_key).category(GasCategory::Fast);
    match oracle.fetch().await {
        Ok(gas_price) => println!("Gas price is {gas_price:?}"),
        Err(e) => println!("Cannot estimate gas: {e:?}"),
    }
}

async fn etherscan() {
    let chain = Chain::Mainnet;
    let api_key = "YOUR-API-KEY";
    if let Ok(client) = Client::new(chain, api_key) {
        let oracle = Etherscan::new(client).category(GasCategory::Fast);
        match oracle.fetch().await {
            Ok(gas_price) => println!("Gas price is {gas_price:?}"),
            Err(e) => println!("Cannot estimate gas: {e:?}"),
        }
    }
}

async fn gas_now() {
    let oracle = GasNow::new().category(GasCategory::Fast);
    match oracle.fetch().await {
        Ok(gas_price) => println!("Gas price is {gas_price:?}"),
        Err(e) => println!("Cannot estimate gas: {e:?}"),
    }
}

async fn polygon() {
    let chain = Chain::Polygon;
    if let Ok(oracle) = Polygon::new(chain) {
        match oracle.category(GasCategory::SafeLow).fetch().await {
            Ok(gas_price) => println!("Gas price is {gas_price:?}"),
            Err(e) => println!("Cannot estimate gas: {e:?}"),
        }
    }
}

async fn provider_oracle() {
    const RPC_URL: &str = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27";
    let provider = Provider::<Http>::try_from(RPC_URL).unwrap();
    let oracle = ProviderOracle::new(provider);
    match oracle.fetch().await {
        Ok(gas_price) => println!("Gas price is {gas_price:?}"),
        Err(e) => println!("Cannot estimate gas: {e:?}"),
    }
}
