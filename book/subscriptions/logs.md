# Subscribing to Logs

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

Here is another example of subscribing to logs:

```rust
{{#include ../../examples/subscriptions/examples/subscribe_logs.rs}}
```
