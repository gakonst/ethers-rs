//! Example usage for the `RwClinet` that uses a didicated client to send transaction and nother one
//! for read ops

use ethers::{prelude::*, utils::Ganache};
use std::{str::FromStr, time::Duration};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let ganache = Ganache::new().spawn();

    let http = Http::from_str(&ganache.endpoint())?;
    let ws = Ws::connect(ganache.ws_endpoint()).await?;

    let provider = Provider::rw(http, ws).interval(Duration::from_millis(10u64));

    dbg!(provider.get_accounts().await?);

    Ok(())
}
