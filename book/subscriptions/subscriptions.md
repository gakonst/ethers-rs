## Ethers-rs: Subscriptions

In this section of the mdbook, we will discuss how to use `ethers-rs` to subscribe and listen to blocks, events, and logs. Subscriptions provide a way to receive real-time updates on various activities on the Ethereum blockchain, allowing you to monitor the network and react to changes as they happen.

## Overview

ethers-rs offers a convenient way to work with subscriptions, enabling you to listen to new blocks, transaction receipts, and logs. The main components you will work with are:

1. Provider: The main struct used to interact with the Ethereum network.
2. SubscriptionStream: A stream of updates you can subscribe to for real-time notifications.

## Getting Started

Before working with subscriptions, make sure you have ethers-rs added to your project's dependencies in Cargo.toml:

```toml
[dependencies]
ethers = { version = "2.0.0", features = ["full"] }
```

Next, import the necessary components from the ethers-rs library:

```rust
use ethers::{prelude::\*,types::H256,};
```

## Subscribing to New Blocks

To subscribe to new blocks, create a Provider instance and call the subscribe_blocks method:

```rust
async fn main() -> Result<(), Box<dyn std::error::Error>> {
let provider = Provider::<Http>::try_from("http://localhost:8545")?;

    let mut stream = provider.subscribe_blocks().await?;

    // Your code to handle new blocks goes here.

    Ok(())

}
```

You can now listen to new blocks as they are mined:

```rust
while let Some(block) = stream.next().await {
    match block {
        Ok(block) => {
            println!("New block: {:?}", block);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}
```

### Subscribing to Logs

To subscribe to logs, create a Filter object that specifies the criteria for the logs you want to listen to. Then, pass the filter to the Provider's subscribe_logs method:

```rust
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from("http://localhost:8545")?;

    let filter = Filter::new().address("0xcontract_address_here".parse()?);

    let mut stream = provider.subscribe_logs(filter).await?;

    // Your code to handle logs goes here.

    Ok(())

}
```

You can now listen to logs that match your filter criteria:

```rust
while let Some(log) = stream.next().await {
    match log {
        Ok(log) => {
            println!("New log: {:?}", log);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}
```

### Subscribing to Events

As we discussed in the previous section on events, you can subscribe to specific events emitted by smart contracts using the EventWatcher struct. To create a SubscriptionStream, call the subscribe method on your EventWatcher:

```rust
let mut stream = watcher.subscribe().await?;
```

Now, you can listen to events as they are emitted by the smart contract:

```rust
while let Some(event) = stream.next().await {
    match event {
        Ok(log) => {
            println!("New event: {:?}", log);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}
```

By using the subscription features provided by ethers-rs, you can efficiently monitor and react to various activities on the Ethereum network. Subscriptions are a powerful tool for building responsive and dynamic applications that can interact with smart contracts and stay up-to-date with the latest network events.

### Unsubscribing from Subscriptions

In some cases, you may want to stop listening to a subscription. To do this, simply drop the SubscriptionStream:

```rust
drop(stream);
```

This will stop the stream from receiving any further updates.
