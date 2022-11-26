use ethers::{contract::Contract, prelude::*};
use std::{error::Error, sync::Arc};
abigen!(
    AggregatorInterface,
    r#"[
        event AnswerUpdated(int256 indexed current, uint256 indexed roundId, uint256 updatedAt)
    ]"#,
);

const PRICE_FEED_1: &str = "0x7de93682b9b5d80d45cd371f7a14f74d49b0914c";
const PRICE_FEED_2: &str = "0x0f00392fcb466c0e4e4310d81b941e07b4d5a079";
const PRICE_FEED_3: &str = "0xebf67ab8cff336d3f609127e8bbf8bd6dd93cd81";

/// Subscribe to a typed event stream without requiring a `Contract` instance.
/// In this example we subscribe Chainlink price feeds and filter out them
/// by address.
/// -------------------------------------------------------------------------------
/// In order to run this example you need to include Ws and TLS features
/// Run this example with
/// `cargo run -p ethers --example subscribe_events_by_type --features="ws","rustls"`
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = get_client().await;
    let client = Arc::new(client);

    // Build an Event by type. We are not tied to a contract instance. We use builder functions to
    // refine the event filter
    let event = Contract::event_of_type::<AnswerUpdatedFilter>(&client)
        .from_block(16022082)
        .address(ValueOrArray::Array(vec![
            PRICE_FEED_1.parse()?,
            PRICE_FEED_2.parse()?,
            PRICE_FEED_3.parse()?,
        ]));

    let mut stream = event.subscribe_with_meta().await?.take(2);

    // Note that `log` has type AnswerUpdatedFilter
    while let Some(Ok((log, meta))) = stream.next().await {
        println!("{:?}", log);
        println!("{:?}", meta)
    }

    Ok(())
}

async fn get_client() -> Provider<Ws> {
    Provider::<Ws>::connect("wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27")
        .await
        .unwrap()
}
