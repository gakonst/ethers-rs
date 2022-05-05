//! Example usage for the `RwClient` that uses a didicated client to send transaction and nother one
//! for read ops

use ethers::{prelude::*, utils::Anvil};
use std::{str::FromStr, time::Duration};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let anvil = Anvil::new().spawn();

    let http = Http::from_str(&anvil.endpoint())?;
    let ws = Ws::connect(anvil.ws_endpoint()).await?;

    let provider = Provider::rw(http, ws).interval(Duration::from_millis(10u64));

    dbg!(provider.get_accounts().await?);

    Ok(())
}
