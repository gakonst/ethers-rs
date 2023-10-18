use ethers::prelude::*;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const WSS_URL: &str = "wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27";

#[derive(Clone, Debug, Serialize, Deserialize, EthEvent)]
pub struct Transfer {
    #[ethevent(indexed)]
    pub from: Address,
    #[ethevent(indexed)]
    pub to: Address,
    pub tokens: U256,
}

/// This example shows how to subscribe to events using the Ws transport for a specific event
#[tokio::main]
async fn main() -> Result<()> {
    let provider = Provider::<Ws>::connect(WSS_URL).await?;
    let provider = Arc::new(provider);
    let event = Transfer::new::<_, Provider<Ws>>(Filter::new(), Arc::clone(&provider));
    let mut transfers = event.subscribe().await?.take(5);
    while let Some(log) = transfers.next().await {
        println!("Transfer: {:?}", log);
    }

    Ok(())
}
