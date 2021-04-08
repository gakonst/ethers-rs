use ethers::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = Provider::connect_ipc("~/.ethereum/geth.ipc")
        .await?
        .interval(Duration::from_millis(2000));
    let block = provider.get_block_number().await?;
    println!("Current block: {}", block);
    let mut stream = provider.watch_blocks().await?.stream();
    while let Some(block) = stream.next().await {
        dbg!(block);
    }

    Ok(())
}
