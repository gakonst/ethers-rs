# Advanced Usage

## `CallBuilder`

The `CallBuilder` is an enum to help create complex calls. `CallBuilder` implements [`RawCall`](https://docs.rs/ethers/latest/ethers/providers/call_raw/trait.RawCall.html) methods for overriding parameters to the `eth_call` rpc method.

Lets take a quick look at how to use the `CallBuilder`.

```rust
use ethers::{
    providers::{Http, Provider},
    types::{TransactionRequest, H160},
    utils::parse_ether,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let rpc_url = "https://eth.llamarpc.com";
    let provider: Arc<Provider<Http>> = Arc::new(Provider::<Http>::try_from(rpc_url)?);

    let from_adr: H160 = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse()?;
    let to_adr: H160 = "0x000000000000000000000000000000000000dead".parse()?;
    let val = parse_ether(1u64)?;

    let tx = TransactionRequest::default()
        .from(from_adr)
        .to(to_adr)
        .value(val)
        .into();

    let result = provider.call_raw(&tx).await?;

    Ok(())
}

```

First, we initialize a new provider and create a transaction that sends `1 ETH` from one address to another. Then we use `provider.call_raw()`, which returns a `CallBuilder`. From here, we can use `await` to send the call to the node with exactly the same behavior as simply using `provider.call()`. We can also override the parameters sent to the node by using the methods provided by the `RawCall` trait. These methods allow you to set the block number that the call should execute on as well as give you access to the [state override set](https://geth.ethereum.org/docs/interacting-with-geth/rpc/ns-eth#3-object---state-override-set).

Here is an example with the exact same raw call, but executed on the previous block.

```rust
use ethers::{
    providers::{call_raw::RawCall, Http, Middleware, Provider},
    types::{BlockId, TransactionRequest, H160},
    utils::parse_ether,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let rpc_url = "https://eth.llamarpc.com";
    let provider: Arc<Provider<Http>> = Arc::new(Provider::<Http>::try_from(rpc_url)?);

    let from_adr: H160 = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse()?;
    let to_adr: H160 = "0x000000000000000000000000000000000000dead".parse()?;
    let val = parse_ether(1u64)?;

    let tx = TransactionRequest::default()
        .from(from_adr)
        .to(to_adr)
        .value(val)
        .into();

    let previous_block_number: BlockId = (provider.get_block_number().await? - 1).into();
    let result = provider.call_raw(&tx).block(previous_block_number).await?;

    Ok(())
}
```

Let's look at how to use the state override set. In short, the state override set is an optional address-to-state mapping, where each entry specifies some state to be ephemerally overridden prior to executing the call. The state override set allows you to override an account's balance, an account's nonce, the code at a given address, the entire state of an account's storage or an individual slot in an account's storage. Note that the state override set is not a default feature and is not available on every node.

```rust
use ethers::{
    providers::{
        call_raw::RawCall,
        Http, Provider,
    },
    types::{TransactionRequest, H160, U256, U64},
    utils::parse_ether,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let rpc_url = "https://eth.llamarpc.com";
    let provider: Arc<Provider<Http>> = Arc::new(Provider::<Http>::try_from(rpc_url)?);

    let from_adr: H160 = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse()?;
    let to_adr: H160 = "0x000000000000000000000000000000000000dead".parse()?;
    let val = parse_ether(1u64)?;

    let tx = TransactionRequest::default()
        .from(from_adr)
        .to(to_adr)
        .value(val)
        .into();

    let mut state = spoof::State::default();

    // Set the account balance to max u256
    state.account(from_adr).balance(U256::MAX);
    // Set the nonce to 0
    state.account(from_adr).nonce(U64::zero());

    let result = provider.call_raw(&tx).state(&state).await?;

    Ok(())
}
```

In this example, the account balance and nonce for the `from_adr` is overridden. The state override set is a very powerful tool that you can use to simulate complicated transactions without undergoing any actual state changes.
