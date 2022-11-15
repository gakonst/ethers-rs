use ethers::{abi::AbiDecode, prelude::*, providers::Middleware, utils::keccak256};
use eyre::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let client: Provider<Ws> =
        Provider::<Ws>::connect("wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27")
            .await?;
    let client = Arc::new(client);

    let last_block = client.get_block(BlockNumber::Latest).await?.unwrap().number.unwrap();
    println!("last_block: {last_block}");

    let erc20_transfer_filter =
        Filter::new().from_block(last_block - 10000).event("Transfer(address,address,uint256)");

    let mut stream = client.get_logs_paginated(&erc20_transfer_filter, 10);

    while let Some(res) = stream.next().await {
        let log = res?;
        println!(
            "block: {:?}, tx: {:?}, token: {:?}, from: {:?}, to: {:?}, amount: {:?}",
            log.block_number,
            log.transaction_hash,
            log.address,
            log.topics.get(1),
            log.topics.get(2),
            U256::decode(log.data)
        );
    }

    Ok(())
}
