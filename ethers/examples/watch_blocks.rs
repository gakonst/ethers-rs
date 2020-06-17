use ethers::prelude::*;
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = Provider::<Http>::try_from("http://localhost:8545")?;
    let mut stream = provider.watch_blocks().await?.interval(2000u64).stream();
    while let Some(block) = stream.next().await {
        dbg!(block);
    }

    Ok(())
}
