/// The IPC (Inter-Process Communication) transport is a way for a process to communicate with a
/// running Ethereum client over a local Unix domain socket. Using the IPC transport allows the
/// ethers library to send JSON-RPC requests to the Ethereum client and receive responses, without
/// the need for a network connection or HTTP server. This can be useful for interacting with a
/// local Ethereum node that is running on the same machine.
#[tokio::main]
#[cfg(feature = "ipc")]
async fn main() -> eyre::Result<()> {
    use ethers::prelude::*;

    // We instantiate the provider using the path of a local Unix domain socket
    // --------------------------------------------------------------------------------
    // NOTE: The IPC transport supports push notifications, but we still need to specify a polling
    // interval because only subscribe RPC calls (e.g., transactions, blocks, events) support push
    // notifications in Ethereum's RPC API. For other calls we must use repeated polling for many
    // operations even with the IPC transport.
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
