use ethers::{
    core::types::BlockNumber,
    middleware::{
        gas_escalator::{Frequency, GasEscalatorMiddleware, GeometricGasPrice},
        gas_oracle::{GasNow, GasOracleMiddleware},
        MiddlewareBuilder, NonceManagerMiddleware, SignerMiddleware,
    },
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};

const RPC_URL: &str = "https://eth.llamarpc.com";
const SIGNING_KEY: &str = "fdb33e2105f08abe41a8ee3b758726a31abdd57b7a443f470f23efce853af169";

/// In ethers-rs, middleware is a way to customize the behavior of certain aspects of the library by
/// injecting custom logic into the process of sending transactions and interacting with contracts
/// on the Ethereum blockchain. The MiddlewareBuilder trait provides a way to define a chain of
/// middleware that will be called at different points in this process, allowing you to customize
/// the behavior of the Provider based on your needs.
#[tokio::main]
async fn main() {
    builder_example().await;
    builder_example_raw_wrap().await;
}

async fn builder_example() {
    let signer = SIGNING_KEY.parse::<LocalWallet>().unwrap();
    let address = signer.address();
    let escalator = GeometricGasPrice::new(1.125, 60_u64, None::<u64>);
    let gas_oracle = GasNow::new();

    let provider = Provider::<Http>::try_from(RPC_URL)
        .unwrap()
        .wrap_into(|p| GasEscalatorMiddleware::new(p, escalator, Frequency::PerBlock))
        .gas_oracle(gas_oracle)
        .with_signer(signer)
        .nonce_manager(address); // Outermost layer

    match provider.get_block(BlockNumber::Latest).await {
        Ok(Some(block)) => println!("{:?}", block.number),
        _ => println!("Unable to get latest block"),
    }
}

async fn builder_example_raw_wrap() {
    let signer = SIGNING_KEY.parse::<LocalWallet>().unwrap();
    let address = signer.address();
    let escalator = GeometricGasPrice::new(1.125, 60_u64, None::<u64>);

    let provider = Provider::<Http>::try_from(RPC_URL)
        .unwrap()
        .wrap_into(|p| GasEscalatorMiddleware::new(p, escalator, Frequency::PerBlock))
        .wrap_into(|p| SignerMiddleware::new(p, signer))
        .wrap_into(|p| GasOracleMiddleware::new(p, GasNow::new()))
        .wrap_into(|p| NonceManagerMiddleware::new(p, address)); // Outermost layer

    match provider.get_block(BlockNumber::Latest).await {
        Ok(Some(block)) => println!("{:?}", block.number),
        _ => println!("Unable to get latest block"),
    }
}
