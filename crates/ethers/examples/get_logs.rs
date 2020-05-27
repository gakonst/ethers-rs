use anyhow::Result;
use ethers::{
    providers::{networks::Any, HttpProvider},
    types::{Address, Filter},
};
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<()> {
    // connect to the network
    let provider = HttpProvider::<Any>::try_from("http://localhost:8545")?;

    let filter = Filter::new()
        .address_str("f817796F60D268A36a57b8D2dF1B97B14C0D0E1d")?
        .event("ValueChanged(address,string,string)") // event name
        .topic("9729a6fbefefc8f6005933898b13dc45c3a2c8b7".parse::<Address>()?); // indexed param

    let logs = provider.get_logs(&filter).await?;
    println!("Got logs: {}", serde_json::to_string(&logs).unwrap());

    Ok(())
}
