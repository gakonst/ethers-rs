//! Example usage for the `RwClient` that uses a didicated client to send transaction and nother one
//! for read ops

use ethers_core::utils::Anvil;
use ethers_providers::{Http, Middleware, Provider, Ws};
use eyre::Result;
use std::{str::FromStr, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    let anvil = Anvil::new().spawn();

    let http = Http::from_str(&anvil.endpoint())?;
    let ws = Ws::connect(anvil.ws_endpoint()).await?;

    let provider = Provider::rw(http, ws).interval(Duration::from_millis(10u64));

    dbg!(provider.get_accounts().await?);

    Ok(())
}
