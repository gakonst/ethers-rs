use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{TxpoolInspect, TxpoolInspectSummary, H160};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
struct TxpoolInspectData {
    #[serde(rename = "queued")]
    queued: BTreeMap<H160, BTreeMap<String, TxpoolInspectSummary>>,
    #[serde(rename = "pending")]
    pending: BTreeMap<H160, BTreeMap<String, TxpoolInspectSummary>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let provider = Provider::<Http>::try_from("https://bsc-dataseed2.defibit.io")?;
    let inspect: TxpoolInspect = provider.txpool_inspect().await?;

    let data = TxpoolInspectData {
        queued: inspect.queued.clone(),
        pending: inspect.pending.clone(),
    };

    // Serialize the data to JSON format
    let json_data = serde_json::to_string_pretty(&data)?;

    // Print the JSON data
    println!("{}", json_data);

    Ok(())
}
