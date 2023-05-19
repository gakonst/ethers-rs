## Ethers-rs: Subscriptions

Here we will discuss how to use `ethers-rs` to subscribe and listen to blocks, events, and logs. Subscriptions provide a way to receive real-time updates on various activities on the Ethereum blockchain, allowing you to monitor the network and react to changes as they happen.

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
