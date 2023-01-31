//! The RwClient wraps two data transports: the first is used for read operations, and the second
//! one is used for write operations, that consume gas like sending transactions.

use ethers::{prelude::*, utils::Anvil};
use url::Url;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let anvil = Anvil::new().spawn();

    let http_url = Url::parse(&anvil.endpoint())?;
    let http = Http::new(http_url);

    let ws = Ws::connect(anvil.ws_endpoint()).await?;

    let _provider = Provider::rw(http, ws);

    Ok(())
}
