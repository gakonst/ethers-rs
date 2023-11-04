# Http

The `Http` provider establishes an HTTP connection with a node, allowing you to send RPC requests to the node to fetch data, simulate calls, send transactions and much more.

## Initializing an Http Provider

Lets take a quick look at few ways to create a new `Http` provider. One of the easiest ways to initialize a new `Provider<Http>` is by using the [`TryFrom`](https://doc.rust-lang.org/stable/std/convert/trait.TryFrom.html) trait's `try_from` method.

```rust
use ethers::providers::{Http, Middleware, Provider};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize a new Http provider
    let rpc_url = "https://eth.llamarpc.com";
    let provider = Provider::try_from(rpc_url)?;

    Ok(())
}
```

The `Http` provider also supplies a way to initialize a new authorized connection.

```rust
use ethers::providers::{Authorization, Http};
use url::Url;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize a new HTTP Client with authentication
    let url = Url::parse("http://localhost:8545")?;
    let provider = Http::new_with_auth(url, Authorization::basic("admin", "good_password"));

    Ok(())
}
```

Additionally, you can initialize a new provider with your own custom `reqwest::Client`.

```rust
use ethers::providers::Http;
use url::Url;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let url = Url::parse("http://localhost:8545")?;
    let client = reqwest::Client::builder().build()?;
    let provider = Http::new_with_client(url, client);

    Ok(())
}
```

## Basic Usage

Now that you have successfully established an Http connection with the node, you can use any of the methods provided by the `Middleware` trait. In the code snippet below, the provider is used to get the chain id, current block number and the content of the node's mempool.

```rust
use ethers::providers::{Http, Middleware, Provider};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let rpc_url = "https://eth.llamarpc.com";
    let provider = Provider::try_from(rpc_url)?;

    let chain_id = provider.get_chainid().await?;
    let block_number = provider.get_block_number().await?;
    let tx_pool_content = provider.txpool_content().await?;

    Ok(())
}
```

You can also use the provider to interact with smart contracts. The snippet below uses the provider to establish a new instance of a UniswapV2Pool and uses the `get_reserves()` method from the smart contract to fetch the current state of the pool's reserves.

```rust
use ethers::{
    prelude::abigen,
    providers::{Http, Provider},
    types::Address,
};
use std::sync::Arc;

abigen!(
    IUniswapV2Pair,
    "[function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)]"
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let rpc_url = "https://eth.llamarpc.com";
    let provider = Arc::new(Provider::try_from(rpc_url)?);

    // Initialize a new instance of the Weth/Dai Uniswap V2 pair contract
    let pair_address: Address = "0xA478c2975Ab1Ea89e8196811F51A7B7Ade33eB11".parse()?;
    let uniswap_v2_pair = IUniswapV2Pair::new(pair_address, provider);

    // Use the get_reserves() function to fetch the pool reserves
    let (reserve_0, reserve_1, block_timestamp_last) =
        uniswap_v2_pair.get_reserves().call().await?;

    Ok(())
}
```

This example is a little more complicated, so let's walk through what is going on. The `IUniswapV2Pair` is a struct that is generated from the `abigen!()` macro. The `IUniswapV2Pair::new()` function is used to create a new instance of the contract, taking in an `Address` and an `Arc<M>` as arguments, where `M` is any type that implements the `Middleware` trait. Note that the provider is wrapped in an [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) when being passed into the `new()` function.

It is very common to wrap a provider in an `Arc` to share the provider across threads. Let's look at another example where the provider is used asynchronously across two tokio threads. In the next example, a new provider is initialized and used to asynchronously fetch the number of Ommer blocks from the most recent block, as well as the previous block.

```rust
use ethers::providers::{Http, Middleware, Provider};
use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let rpc_url = "https://eth.llamarpc.com";
    let provider = Arc::new(Provider::try_from(rpc_url)?);

    let current_block_number = provider.get_block_number().await?;
    let prev_block_number = current_block_number - 1;

    // Clone the Arc<Provider> and pass it into a new thread to get the uncle count of the current block
    let provider_1 = provider.clone();
    let task_0 =
        tokio::spawn(async move { provider_1.get_uncle_count(current_block_number).await });

    // Spin up a new thread to get the uncle count of the previous block
    let task_1 = tokio::spawn(async move { provider.get_uncle_count(prev_block_number).await });

    // Wait for the tasks to finish
    for task in [task_0, task_1] {
        if let Ok(uncle_count) = task.await? {
            println!("Success!");
        }
    }

    Ok(())
}
```

<br>

Before heading to the next chapter, feel free to check out the docs for the [`Http` provider](https://docs.rs/ethers/latest/ethers/providers/struct.Http.html). Keep in mind that we will cover advanced usage of providers at the end of this chapter. Now that we have the basics covered, let's move on to the next provider, Websockets!
