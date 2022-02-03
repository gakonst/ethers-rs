use ethers::{prelude::*, utils::Ganache};
use std::time::Duration;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let ganache = Ganache::new().block_time(1u64).spawn();
    let ws = Ws::connect(ganache.ws_endpoint()).await?;
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));
    let mut stream = provider.watch_blocks().await?.take(5);
    while let Some(block) = stream.next().await {
        dbg!(block);
    }

    Ok(())
}
