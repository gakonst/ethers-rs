# Multiple Subscriptions

You may need to handle multiple subscriptions simultaneously in your application. To manage multiple SubscriptionStreams, you can use the futures crate to efficiently process updates from all streams concurrently:

```toml
[dependencies]
futures = "0.3"
```

Then, import the necessary components:

```rust
use futures::{stream, StreamExt, TryStreamExt};
```

Create multiple subscription streams and merge them into a single stream using the stream::select_all function:

```rust
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create multiple subscription streams.
    let mut block_stream = provider.subscribe_blocks().await?;
    let mut log_stream = provider.subscribe_logs(filter).await?;
    let mut event_stream = watcher.subscribe().await?;

    // Merge the streams into a single stream.
    let mut combined_stream = stream::select_all(vec![
        block_stream.map_ok(|block| EventType::Block(block)),
        log_stream.map_ok(|log| EventType::Log(log)),
        event_stream.map_ok(|event| EventType::Event(event)),
    ]);

    // Your code to handle the events goes here.

    Ok(())

}
```

Now, you can listen to updates from all the subscription streams concurrently:

```rust
while let Some(event) = combined_stream.next().await {
    match event {
        Ok(event) => match event {
            EventType::Block(block) => println!("New block: {:?}", block),
            EventType::Log(log) => println!("New log: {:?}", log),
            EventType::Event(event) => println!("New event: {:?}", event),
        },
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}
```

This approach allows you to efficiently handle multiple subscriptions in your application and react to various network activities in a unified manner.

By leveraging the powerful subscription capabilities of ethers-rs, you can create responsive and dynamic applications that stay up-to-date with the latest events on the Ethereum network. The library's flexibility and ease of use make it an ideal choice for developers looking to build robust and performant applications that interact with smart contracts and the Ethereum blockchain.
