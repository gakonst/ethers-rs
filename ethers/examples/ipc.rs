use ethers::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO switch to some default IPC after
    let ws = Ipc::new("/home/austinabell/development/github.com/austinabell/ethdev/node01/geth.ipc").await?;
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));
    let block = provider.get_block_number().await?;
    println!("Current block: {}", block);
    let mut stream = provider.watch_blocks().await?.stream();
    while let Some(block) = stream.next().await {
        dbg!(block);
    }

    Ok(())
}
