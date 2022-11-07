use ethers::prelude::*;
use eyre::Result;
use std::sync::Arc;

// Generate the type-safe contract bindings by providing the ABI
// definition in human readable format
abigen!(
    ERC20,
    r#"[
        event  Transfer(address indexed src, address indexed dst, uint wad)
    ]"#,
);

// In order to run this example you need to include Ws and TLS features
// Run this example with
// `cargo run -p ethers --example subscribe_contract_events --features="ws","rustls"`
#[tokio::main]
async fn main() -> Result<()> {
    let client =
        Provider::<Ws>::connect("wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27")
            .await?;

    let client = Arc::new(client);

    // WETH Token
    let address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse::<Address>()?;
    let weth = ERC20::new(address, Arc::clone(&client));

    // Subscribe Transfer events
    let events = weth.events();
    let mut stream = events.stream().await?;

    while let Some(Ok(event)) = stream.next().await {
        println!("src: {:?}, dst: {:?}, wad: {:?}", event.src, event.dst, event.wad);
    }

    Ok(())
}
