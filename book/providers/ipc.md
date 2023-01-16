# IPC provider
The IPC (Inter-Process Communication) transport is a way for a process to communicate with a running Ethereum client over a local Unix domain socket. If you are new to IPC, you can [follow this link to learn more](https://en.wikipedia.org/wiki/Inter-process_communication). Using the IPC transport allows the ethers library to send JSON-RPC requests to the Ethereum client and receive responses, without the need for a network connection or HTTP server. This can be useful for interacting with a local Ethereum node that is running on the same machine. Using Ipc [is faster than RPC](https://github.com/0xKitsune/geth-ipc-rpc-bench), however you will need to have a local node that you can connect to.

## Initializing an Ipc Provider
Below is an example of how to initialize a new Ipc provider. 

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

Note that if you are using Windows, you must use [Windows Ipc (Named pipes)](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipes). Instead of passing the provider the path to the `.ipc` file, you must pass a named pipe (`\\<machine_address>\pipe\<pipe_name>`).  For a local geth connection, the named pipe will look something like this: `\\.\pipe\geth`

## Usage

The `Ipc` provider has the same methods as the `Ws` provider, allowing it to subscribe and unsubscribe via a `NotificationStream`.


```rust
use ethers::providers::{Middleware, Provider, StreamExt, Ws};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let provider = Provider::connect_ipc("~/.ethereum/geth.ipc")
        .await?
        .interval(std::time::Duration::from_millis(2000));
        
    //Create a new stream yielding pending transactions from the mempool
    let mut tx_pool_stream = provider.subscribe_pending_txs().await?;

    while let Some(tx_hash) = tx_pool_stream.next().await {
        println!("Pending tx: {:?}", tx_hash);
    }

    Ok(())
}
```


Note that the `Ipc` provider, like all providers, has access to the methods defined by the `Middleware` trait. With this in mind, we can use the `Ipc` provider just like the `Http` provider as well, with the only difference being that we are connected to the node via a Unix socket now!


```rust
use std::{str::FromStr, sync::Arc};

use ethers::{
    prelude::abigen,
    providers::{Http, Provider},
    types::H160,
};

abigen!(
    IUniswapV2Pair,
    r#"[function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)]"#
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let provider = Provider::connect_ipc("~/.ethereum/geth.ipc")
        .await?
        .interval(std::time::Duration::from_millis(2000));

    //Initialize a new instance of the Weth/Dai Uniswap V2 pair contract
    let pair_address = H160::from_str("0xA478c2975Ab1Ea89e8196811F51A7B7Ade33eB11").unwrap();
    let uniswap_v2_pair = IUniswapV2Pair::new(pair_address, provider);

    //Use the get_reserves() function to fetch the pool reserves
    let (reserve_0, reserve_1, block_timestamp_last) =
        uniswap_v2_pair.get_reserves().call().await?;

    Ok(())
}
```
