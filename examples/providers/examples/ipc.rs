#[tokio::main]
#[cfg(feature = "ipc")]
async fn main() -> eyre::Result<()> {
    use ethers::prelude::*;

    let provider = Provider::connect_ipc("~/.ethereum/geth.ipc")
        .await?
        .interval(std::time::Duration::from_millis(2000));
    let block = provider.get_block_number().await?;
    println!("Current block: {block}");
    let mut stream = provider.watch_blocks().await?.stream();
    while let Some(block) = stream.next().await {
        dbg!(block);
    }

    Ok(())
}

#[cfg(not(feature = "ipc"))]
fn main() {}
