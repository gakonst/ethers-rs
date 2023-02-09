# IPC provider

The [IPC (Inter-Process Communication)](https://en.wikipedia.org/wiki/Inter-process_communication) transport allows our program to communicate with a node over a local [Unix domain socket](https://en.wikipedia.org/wiki/Unix_domain_socket) or [Windows named pipe](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipes).

Using the IPC transport allows the ethers library to send JSON-RPC requests to the Ethereum client and receive responses, without the need for a network connection or HTTP server. This can be useful for interacting with a local Ethereum node that is running on the same network. Using IPC [is faster than RPC](https://github.com/0xKitsune/geth-ipc-rpc-bench), however you will need to have a local node that you can connect to.

## Initializing an Ipc Provider

Below is an example of how to initialize a new Ipc provider.

```rust
use ethers::providers::Provider;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Using a UNIX domain socket: `/path/to/ipc`
    #[cfg(unix)]
    let provider = Provider::connect_ipc("~/.ethereum/geth.ipc").await?;

    // Using a Windows named pipe: `\\<machine_address>\pipe\<pipe_name>`
    #[cfg(windows)]
    let provider = Provider::connect_ipc(r"\\.\pipe\geth").await?;

    Ok(())
}
```

## Usage

The `Ipc` provider implements both `JsonRpcClient` and `PubsubClient`, just like `Ws`.

In this example, we monitor the [`WETH/USDC`](https://etherscan.io/address/0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc) [UniswapV2](https://docs.uniswap.org/) pair reserves and print when they have changed.

```rust
{{#include ../../examples/providers/examples/ipc.rs}}
```
