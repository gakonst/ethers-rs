use ethers::{
    core::types::Chain,
    etherscan::Client,
    middleware::gas_oracle::{
        BlockNative, Etherscan, GasCategory, GasNow, GasOracle, Polygon, ProviderOracle,
    },
    providers::{Http, Provider},
};

/// In Ethereum, the "gas" of a transaction refers to the amount of computation required to execute
/// the transaction on the blockchain. Gas is typically measured in units of "gas," and the cost of
/// a transaction is determined by the amount of gas it consumes.
///
/// A "gas oracle" is a tool or service that provides information about the current price of gas on
/// the Ethereum network. Gas oracles are often used to help determine the appropriate amount of gas
/// to include in a transaction, in order to ensure that it will be processed in a timely manner
/// without running out of gas.
///
/// Ethers-rs includes a feature called "gas oracle middleware" that allows you to customize the
/// behavior of the library when it comes to determining the gas cost of transactions.
#[tokio::main]
async fn main() {
    blocknative().await;
    etherscan().await;
    gas_now().await;
    polygon().await;
    provider_oracle().await;
    //etherchain().await; // FIXME: Etherchain URL is broken (Http 404)
}

async fn blocknative() {
    let api_key: Option<String> = std::env::var("BLOCK_NATIVE_API_KEY").ok();
    let oracle = BlockNative::new(api_key).category(GasCategory::Fastest);
    match oracle.fetch().await {
        Ok(gas_price) => println!("[Blocknative]: Gas price is {gas_price:?}"),
        Err(e) => panic!("[Blocknative]: Cannot estimate gas: {e:?}"),
    }
}

async fn etherscan() {
    let client = Client::new_from_opt_env(Chain::Mainnet).unwrap();
    let oracle = Etherscan::new(client).category(GasCategory::Fast);
    match oracle.fetch().await {
        Ok(gas_price) => println!("[Etherscan]: Gas price is {gas_price:?}"),
        Err(e) => panic!("[Etherscan]: Cannot estimate gas: {e:?}"),
    }
}

async fn gas_now() {
    let oracle = GasNow::new().category(GasCategory::Fast);
    match oracle.fetch().await {
        Ok(gas_price) => println!("[GasNow]: Gas price is {gas_price:?}"),
        Err(e) => panic!("[GasNow]: Cannot estimate gas: {e:?}"),
    }
}

async fn polygon() {
    let chain = Chain::Polygon;
    if let Ok(oracle) = Polygon::new(chain) {
        match oracle.category(GasCategory::SafeLow).fetch().await {
            Ok(gas_price) => println!("[Polygon]: Gas price is {gas_price:?}"),
            Err(e) => panic!("[Polygon]: Cannot estimate gas: {e:?}"),
        }
    }
}

async fn provider_oracle() {
    const RPC_URL: &str = "https://eth.llamarpc.com";
    let provider = Provider::<Http>::try_from(RPC_URL).unwrap();
    let oracle = ProviderOracle::new(provider);
    match oracle.fetch().await {
        Ok(gas_price) => println!("[Provider oracle]: Gas price is {gas_price:?}"),
        Err(e) => panic!("[Provider oracle]: Cannot estimate gas: {e:?}"),
    }
}

/*
// FIXME: Etherchain URL is broken (Http 404)
async fn etherchain() {
    let oracle = Etherchain::new().category(GasCategory::Standard);
    match oracle.fetch().await {
        Ok(gas_price) => println!("[Etherchain]: Gas price is {gas_price:?}"),
        Err(e) => panic!("[Etherchain]: Cannot estimate gas: {e:?}"),
    }
}*/
