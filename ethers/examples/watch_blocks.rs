use ethers::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ws = Ws::connect("ws://localhost:8546").await?;
    let provider = Provider::new(ws);
    let mut stream = provider.watch_blocks().await?.interval(2000u64).stream();
    while let Some(block) = stream.next().await {
        dbg!(block);
    }

    Ok(())
}
