use ethers::providers::{Middleware, Provider, StreamExt, Ws};
use eyre::Result;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let ws_endpoint = "wss://eth.llamarpc.com";
    let ws = Ws::connect(ws_endpoint).await?;
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));
    let mut stream = provider.watch_blocks().await?.take(1);
    while let Some(block) = stream.next().await {
        let block = provider.get_block(block).await?.unwrap();
        println!(
            "Ts: {:?}, block number: {} -> {:?}",
            block.timestamp,
            block.number.unwrap(),
            block.hash.unwrap()
        );
    }

    Ok(())
}
