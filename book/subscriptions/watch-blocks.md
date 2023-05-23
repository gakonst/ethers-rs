# Subscribing to New Blocks

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

Here is another example of subscribing to new blocks:

```rust
{{#include ../../examples/subscriptions/examples/subscribe_blocks.rs}}
```
