# WebSocket provider
The Ws provider allows you to send JSON-RPC requests and receive responses over WebSocket connections. The WS provider can be used with any Ethereum node that supports WebSocket connections. This allows programs interact with the network in real-time without the need for HTTP polling for things like new block headers and filter logs. Ethers-rs has support for WebSockets via Tokio. Make sure that you have the “ws” and “rustls” / “openssl” features enabled in your project's toml file if you wish to use WebSockets.



## Initializing a WS Provider
Lets look at a few ways to create a new `WS` provider.


```rust
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let ws_endpoint = "";
    let provider = Provider::<Ws>::connect(ws_endpoint).await?;
    Ok(())
}
```

TODO: note on setting the polling interval


TODO: note on initializing a new ws provider with authorization like the http provider
```rust
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let ws_endpoint = "";
    let auth = Authorization::basic("username", "password");
    
    if let Ok(_provider) = Provider::<Ws>::connect_with_auth(url, auth).await {
        println!("Create Ws provider with auth");
    }
    
    Ok(())
}
```



TODO: snippet of creating a new ws provider with a custom WS. Initializes a new WebSocket Client, given a Stream/Sink Websocket implementer. The websocket connection must be initiated separately.

```rust
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let ws_endpoint = "";
    let auth = Authorization::basic("username", "password");
    
    if let Ok(_provider) = Provider::<Ws>::connect_with_auth(url, auth).await {
        println!("Create Ws provider with auth");
    }
    
    Ok(())
}
```

## Usage
TODO: Examples of syncing to new blocks, filter logs. Mention that the WS client implements the `PubSubClient` trait which gives access to the subscribe and unsubscribe methods. 

```rust
use ethers::providers::{Middleware, Provider, StreamExt, Ws};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let ws_endpoint = "";
    let provider = Provider::<Ws>::connect(ws_endpoint).await?;
    //Create a new stream yielding pending transactions from the mempool
    let mut tx_pool_stream = provider.subscribe_pending_txs().await?;

    while let Some(tx_hash) = tx_pool_stream.next().await {
        println!("Pending tx: {:?}", tx_hash);
    }

    Ok(())
}
```