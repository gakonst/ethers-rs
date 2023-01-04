use std::time::Duration;

use ethers::prelude::*;

const WSS_URL: &str = "wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27";

type BoxErr = Box<dyn std::error::Error>;

/// The Ws transport allows you to send JSON-RPC requests and receive responses over WebSocket
/// connections. It is useful for connecting to Ethereum nodes that support WebSockets.
/// This allows to interact with the Ethereum network in real-time without the need for HTTP
/// polling.
#[tokio::main]
async fn main() -> Result<(), BoxErr> {
    create_instance().await?;
    watch_blocks().await?;
    Ok(())
}

async fn create_instance() -> Result<(), BoxErr> {
    // An Ws provider can be created from an ws(s) URI.
    // In case of wss you must add the "rustls" or "openssl" feature
    // to the ethers library dependency in `Cargo.toml`.
    //------------------------------------------------------------------------------------------
    // NOTE: The Ws transport supports push notifications, but we still need to specify a polling
    // interval because only subscribe RPC calls (e.g., transactions, blocks, events) support push
    // notifications in Ethereum's RPC API. For other calls we must use repeated polling for many
    // operations even with the Ws transport.
    let _provider = Provider::<Ws>::connect(WSS_URL).await?.interval(Duration::from_millis(500));

    // Instantiate with auth to send basic authorization headers on connection.
    let url = reqwest::Url::parse(WSS_URL)?;
    let auth = Authorization::basic("username", "password");
    if let Ok(_provider) = Provider::<Ws>::connect_with_auth(url, auth).await {
        println!("Create Ws provider with auth");
    }

    Ok(())
}

/// Let's show how the Ws connection enables listening for blocks using a persistent TCP connection
async fn watch_blocks() -> Result<(), BoxErr> {
    let provider = Provider::<Ws>::connect(WSS_URL).await?;
    let mut stream = provider.watch_blocks().await?.take(1);

    while let Some(block_hash) = stream.next().await {
        println!("{block_hash:?}");
    }

    Ok(())
}
