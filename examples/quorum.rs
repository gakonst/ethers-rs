//! Example usage for the `QuorumProvider` that requests multiple backends and only returns
//! a value if the configured `Quorum` was reached.

use ethers::{prelude::*, utils::Anvil};
use std::{str::FromStr, time::Duration};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let anvil = Anvil::new().spawn();

    // create a quorum provider with some providers
    let quorum = QuorumProvider::dyn_rpc()
        .add_provider(WeightedProvider::new(Box::new(Http::from_str(&anvil.endpoint())?)))
        .add_provider(WeightedProvider::with_weight(
            Box::new(Ws::connect(anvil.ws_endpoint()).await?),
            2,
        ))
        .add_provider(WeightedProvider::with_weight(
            Box::new(Ws::connect(anvil.ws_endpoint()).await?),
            2,
        ))
        // the quorum provider will yield the response if >50% of the weighted inner provider
        // returned the same value
        .quorum(Quorum::Majority)
        .build();

    let provider = Provider::quorum(quorum).interval(Duration::from_millis(10u64));

    dbg!(provider.get_accounts().await?);

    Ok(())
}
