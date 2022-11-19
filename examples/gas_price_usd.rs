use ethers::{prelude::*, utils::format_units};
use std::{
    error::Error,
    ops::{Div, Mul},
    sync::Arc,
};

abigen!(
    AggregatorInterface,
    r#"[
        latestAnswer() public view virtual override returns (int256 answer)
    ]"#,
);

const ETH_DECIMALS: u32 = 18;
const USD_PRICE_DECIMALS: u32 = 8;
const ETH_USD_FEED: &str = "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419";
const RPC_URI: &str = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27";

/// Retrieves the USD amount per gas unit, using a Chainlink price oracle.
/// Function gets the amount of `wei` to be spent per gas unit then multiplies
/// for the ETH USD value.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = get_client();
    let oracle = get_oracle(&client);

    let usd_per_eth: I256 = oracle.latest_answer().call().await?;
    let usd_per_eth: U256 = U256::from(usd_per_eth.as_u128());
    let wei_per_gas: U256 = client.get_gas_price().await?;

    // Gas stations use to report gas price in gwei units (1 gwei = 10^9 wei)
    let gwei: f64 = format_units(wei_per_gas, "gwei")?.parse::<f64>()?;

    // Let's convert the gas price to USD
    let usd_per_gas: f64 = usd_value(wei_per_gas, usd_per_eth)?;

    println!(
        r#"
        Gas price
        ---------------
        {:>10.2} gwei
        {:>10.8} usd
        "#,
        gwei, usd_per_gas
    );
    Ok(())
}

/// `amount`: Number of wei per gas unit (18 decimals)
/// `price_usd`: USD price per ETH (8 decimals)
fn usd_value(amount: U256, price_usd: U256) -> Result<f64, Box<dyn Error>> {
    let base: U256 = U256::from(10).pow(ETH_DECIMALS.into());
    let value: U256 = amount.mul(price_usd).div(base);
    let f: String = format_units(value, USD_PRICE_DECIMALS)?;
    Ok(f.parse::<f64>()?)
}

fn get_client() -> Arc<Provider<Http>> {
    let provider: Provider<Http> = Provider::<Http>::try_from(RPC_URI).expect("Valid URL");
    Arc::new(provider)
}

fn get_oracle(client: &Arc<Provider<Http>>) -> AggregatorInterface<Provider<Http>> {
    let address: Address = ETH_USD_FEED.parse().expect("Valid address");
    AggregatorInterface::new(address, Arc::clone(client))
}
