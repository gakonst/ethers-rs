use ethers::{prelude::*, utils::Anvil};
use std::time::Duration;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let ws = Ws::connect(anvil.ws_endpoint()).await?;
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));
    let mut stream = provider.watch_blocks().await?.take(5);
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
