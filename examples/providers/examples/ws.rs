//! The Ws transport allows you to send JSON-RPC requests and receive responses over
//! [WebSocket](https://en.wikipedia.org/wiki/WebSocket).
//!
//! This allows to interact with the network in real-time without the need for HTTP
//! polling.

use ethers::prelude::*;

const WSS_URL: &str = "wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // A Ws provider can be created from a ws(s) URI.
    // In case of wss you must add the "rustls" or "openssl" feature
    // to the ethers library dependency in `Cargo.toml`.
    let provider = Provider::<Ws>::connect(WSS_URL).await?;

    let mut stream = provider.subscribe_blocks().await?.take(1);
    while let Some(block) = stream.next().await {
        println!("{:?}", block.hash);
    }

    Ok(())
}
