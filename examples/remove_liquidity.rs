use ethers::prelude::*;
use eyre::Result;
use std::{convert::TryFrom, sync::Arc};

abigen!(
    UniswapV2Router,
    r#"[
        removeLiquidity(address tokenA,address tokenB, uint liquidity,uint amountAMin, uint amountBMin, address to, uint ) external returns (uint amountA, uint amountB)
    ]"#,
);

abigen!(
    UniswapV2Pair,
    r#"[
        approve(address,uint256)(bool)
        getReserves()(uint112,uint112,uint32)
        token0()(address)
        token1()(address)
    ]"#
);

fn main() {}

// Remove liquidity from uniswap V2.
// This example will remove 500 liquidity of 2 test tokens, TA and TB on goerli testnet.
// This example uses pair contract and uniswap swap contract to remove liquidity.
#[allow(dead_code)]
async fn example() -> Result<()> {
    let provider = Arc::new({
        // connect to the network
        let provider = Provider::<Http>::try_from(
            "https://rinkeby.infura.io/v3/a111fcada47746d990e0e2b7df50d00a",
        )?;
        let chain_id = provider.get_chainid().await?;

        // this wallet's private key
        let wallet = "725fd1619b2653b7ff1806bf29ae11d0568606d83777afd5b1f2e649bd5132a9"
            .parse::<LocalWallet>()?
            .with_chain_id(chain_id.as_u64());

        SignerMiddleware::new(provider, wallet)
    });

    let pair = "0xA6108E4d436bE592bAc12F9A0aB7D9A10d821176".parse::<Address>()?;
    let pair = UniswapV2Pair::new(pair, provider.clone());

    let router = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".parse::<Address>()?;
    let router = UniswapV2Router::new(router, provider.clone());

    let (reserve0, reserve1, _) = pair.get_reserves().call().await?;

    println!("Reserves (token A, Token B): ({reserve0}, {reserve1})");

    let price =
        if reserve0 > reserve1 { 1000 * reserve0 / reserve1 } else { 1000 * reserve1 / reserve0 } /
            1000;
    println!("token0 / token1 price = {price}");

    let liquidity = 100.into();

    println!("Approving the transaction!");
    let receipt =
        pair.approve(router.address(), liquidity).send().await?.await?.expect("no receipt found");
    println!("contract approved succesfully!");
    println!("{receipt:?}");

    println!("Removing {liquidity} liquidity!");

    let token0 = pair.token_0().call().await?;
    let token1 = pair.token_1().call().await?;

    let receipt = router
        .remove_liquidity(
            token0,
            token1,
            liquidity,
            0.into(),
            0.into(),
            provider.address(),
            U256::MAX,
        )
        .send()
        .await?
        .await?
        .expect("no receipt for remove_liquidity");
    println!("liquidity removed succesfully!");
    println!("{receipt:?}");

    Ok(())
}
