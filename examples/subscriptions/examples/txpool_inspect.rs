use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{TxpoolInspect, TxpoolInspectSummary, H160};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Define a struct to hold the data extracted from the txpool inspect response
#[derive(Serialize, Deserialize, Debug)]
struct TxpoolInspectData {
    #[serde(rename = "queued")]
    queued: BTreeMap<H160, BTreeMap<String, TxpoolInspectSummary>>,
    #[serde(rename = "pending")]
    pending: BTreeMap<H160, BTreeMap<String, TxpoolInspectSummary>>,
}

// The main async function
#[tokio::main]
async fn main() -> Result<()> {
    // Create a provider instance connected to the node, you can use any rpc ethereum compatible here
    let provider = Provider::<Http>::try_from("https://bsc-dataseed2.defibit.io")?;

    // Fetch txpool inspect data from the provider
    let inspect: TxpoolInspect = provider.txpool_inspect().await?;

    // Create a TxpoolInspectData instance with relevant data
    let data = TxpoolInspectData {
        queued: inspect.queued.clone(),
        pending: inspect.pending.clone(),
    };

    // Serialize the data to JSON format
    let json_data = serde_json::to_string_pretty(&data)?;

    // Print the JSON data
    println!("{}", json_data);

    Ok(()) // Return a success result
}

