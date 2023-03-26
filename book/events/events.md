# Ethers-rs: Working with Events

In this section we will discuss how to monitor, subscribe, and listen to events using the ethers-rs library. Events are an essential part of smart contract development, as they allow you to track specific occurrences on the blockchain, such as transactions, state changes, or function calls.

## Overview

ethers-rs provides a simple and efficient way to interact with events emitted by smart contracts. You can listen to events, filter them based on certain conditions, and subscribe to event streams for real-time updates. The key components you will work with are:

1. Event: A struct representing an event emitted by a smart contract.
2. EventWatcher: A struct that allows you to monitor and filter events.
3. SubscriptionStream: A stream of events you can subscribe to for real-time updates.

## Getting Started

Before diving into event handling, ensure you have ethers-rs added to your project's dependencies in Cargo.toml:

```toml
[dependencies]
ethers = { version = "2.0.0.", features = ["full"] }
```

Now, let's import the necessary components from the ethers-rs library:

```rust
use ethers::{
prelude::contract::{Contract, EthEvent},
};
```

### Listening to Events

To listen to events, you'll need to instantiate a Contract object and use the event method to create an Event struct. You'll also need to define a struct that implements the EthEvent trait, representing the specific event you want to listen to.

Consider a simple smart contract that emits an event called ValueChanged:

```solidity
pragma solidity ^0.8.0;

contract SimpleStorage {

    uint256 public value;
    event ValueChanged(uint256 newValue);

    function setValue(uint256 _value) public {
        value = _value;
        emit ValueChanged(_value);
    }

}
```

First, define a struct representing the ValueChanged event:

```rust
#[derive(Debug, Clone, EthEvent)]
pub struct ValueChanged {
    pub new_value: U256,
}
```

Then, create an instance of the Contract object and listen for the ValueChanged event:

```rust
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from("http://localhost:8545")?;
    let contract_address = "0xcontract_address_here".parse()?;
    let contract = Contract::from_json(provider,
        contract_address,
        include_bytes!("../contracts/abis/SimpleStorage.json"))?;

    let event = contract.event::<ValueChanged>()?;

    // Your code to handle the event goes here.

    Ok(())

}
```

### Filtering Events

You can filter events based on specific conditions using the EventWatcher struct. To create an EventWatcher, call the watcher method on your Event object:

```rust
let watcher = event.watcher().from_block(5).to_block(10);
```

In this example, the EventWatcher will only monitor events from block 5 to block 10.

### Subscribing to Events

To receive real-time updates for an event, create a SubscriptionStream by calling the subscribe method on your EventWatcher:

```rust
let mut stream = watcher.subscribe().await?;
```

You can now listen to events as they are emitted by the smart contract:

```rust
while let Some(event) = stream.next().await {
    match event {
        Ok(log) => {println!("New event: {:?}", log)},
        Err(e) => {println!("Error: {:?}", e)},
```
