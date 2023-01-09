# IPC provider
The IPC (Inter-Process Communication) transport is a way for a process to communicate with a running Ethereum client over a local Unix domain socket. If you are new to IPC, you can [follow this link to learn more](). Using the IPC transport allows the ethers library to send JSON-RPC requests to the Ethereum client and receive responses, without the need for a network connection or HTTP server. This can be useful for interacting with a local Ethereum node that is running on the same machine. Using Ipc [is faster than RPC](), however you will need to have a local node that you can connect to.

## Initializing an Ipc Provider

```rust
#[tokio::main]
async fn main() -> eyre::Result<()> {

    // We instantiate the provider using the path of a local Unix domain socket
    // --------------------------------------------------------------------------------
    // NOTE: The IPC transport supports push notifications, but we still need to specify a polling
    // interval because only subscribe RPC calls (e.g., transactions, blocks, events) support push
    // notifications in Ethereum's RPC API. For other calls we must use repeated polling for many
    // operations even with the IPC transport.
    let provider = Provider::connect_ipc("~/.ethereum/geth.ipc")
        .await?
        .interval(std::time::Duration::from_millis(2000));

    Ok(())
}
```
## Usage

