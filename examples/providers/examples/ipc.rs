//! The IPC (Inter-Process Communication) transport allows our program to communicate
//! with a node over a local [Unix domain socket](https://en.wikipedia.org/wiki/Unix_domain_socket)
//! or [Windows named pipe](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipes).
//!
//! It functions much the same as a Ws connection.

use ethers::prelude::*;
use std::sync::Arc;

abigen!(
    IUniswapV2Pair,
    "[function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)]"
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let provider = Provider::connect_ipc("~/.ethereum/geth.ipc").await?;
    let provider = Arc::new(provider);

    let pair_address: Address = "0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc".parse()?;
    let weth_usdc = IUniswapV2Pair::new(pair_address, provider.clone());

    let block = provider.get_block_number().await?;
    println!("Current block: {block}");

    let mut initial_reserves = weth_usdc.get_reserves().call().await?;
    println!("Initial reserves: {initial_reserves:?}");

    let mut stream = provider.subscribe_blocks().await?;
    while let Some(block) = stream.next().await {
        println!("New block: {:?}", block.number);

        let reserves = weth_usdc.get_reserves().call().await?;
        if reserves != initial_reserves {
            println!("Reserves changed: old {initial_reserves:?} - new {reserves:?}");
            initial_reserves = reserves;
        }
    }

    Ok(())
}
