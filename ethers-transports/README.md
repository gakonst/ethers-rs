# JSON-RPC Transports

This crate provides asynchronous JSON-RPC and subscription transports.

These transports may be used to parameterize an `ethers_providers::Provider`.

# Examples

```no_run
use ethers_core::types::U64;
use ethers_transports::{Http, JsonRpcClient};

# async fn foo() -> Result<(), Box<dyn std::error::Error>> {
let provider: Http = "http://localhost:8545".parse()?;
let block_number: U64 = provider.request("eth_blockNumber", ()).await?;
# Ok(())
# }
```

# Websockets

The crate has support for WebSockets via Tokio. Please ensure that you have the "ws" and "rustls" / "openssl" features enabled if you wish to use WebSockets.

```
use ethers_transports::Ws;

# async fn foo() -> Result<(), Box<dyn std::error::Error>> {
let ws = Ws::connect("ws://localhost:8545").await?;
# Ok(())
# }
```
