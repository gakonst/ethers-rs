# WebSocket provider
The Ws provider allows you to send JSON-RPC requests and receive responses over WebSocket connections. The WS provider can be used with any Ethereum node that supports WebSocket connections. This allows programs interact with the network in real-time without the need for HTTP polling for things like new block headers and filter logs. Ethers-rs has support for WebSockets via Tokio. Make sure that you have the “ws” and “rustls” / “openssl” features enabled in your project's toml file if you wish to use WebSockets.



## Initializing a WS Provider
Lets look at a few ways to create a new `WS` provider. Below is the most straightforward way to initialize a new `Ws` provider.


```rust
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let ws_endpoint = "";
    let provider = Provider::<Ws>::connect(ws_endpoint).await?;
    Ok(())
}
```

Similar to the other providers, you can also establish an authorized connection with a node via websockets.

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

The `Ws` provider allows a user to send requests to the node just like the other providers. In addition to these methods, the `Ws` provider can also subscribe to new logs and events, watch transactions in the mempool and other types of data streams from the node. The default polling interval for the `Ws` provider is `7 seconds`. You can update the polling interval, by using the `provider.interval()` method.

In the snippet below, a new `Ws` provider is used to watch pending transactions in the mempool as well as new block headers in two separate threads.

```rust
use ethers::providers::{Middleware, Provider, StreamExt, Ws};
use std::{sync::Arc, time::Duration};
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let ws_endpoint = "";
    let mut provider = Provider::<Ws>::connect(ws_endpoint).await?;

    // Update the polling interval
    provider.set_interval(Duration::new(3, 0));

    // Clone the providers to use in separate threads
    let provider = Arc::new(provider);
    let provider_0 = provider.clone();
    let provider_1 = provider.clone();

    let mut handles = vec![];

    let pending_tx_handle = tokio::spawn(async move {
        let mut tx_pool_stream = provider_0.watch_pending_transactions().await.unwrap();
        while let Some(tx_hash) = tx_pool_stream.next().await {
            println!("Pending tx: {:?}", tx_hash);
        }
    });

    let new_block_headers_handle = tokio::spawn(async move {
        let mut new_block_headers_stream = provider_1.watch_blocks().await.unwrap();
        while let Some(block_hash) = new_block_headers_stream.next().await {
            println!("New block: {:?}", block_hash);
        }
    });

    // Add the JoinHandles to a vec and wait for the handles to complete
    handles.push(pending_tx_handle);
    handles.push(new_block_headers_handle);
    for handle in handles {
        if let Err(err) = handle.await {
            panic!("{}", err);
        }
    }

    Ok(())
}
```