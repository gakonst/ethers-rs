use ethers::prelude::*;
use eyre::Result;
use std::env;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url: String = env::var("RPC_URL")?;
    let client = Provider::<Http>::try_from(rpc_url)?;
    let tx_hash = "0x97a02abf405d36939e5b232a5d4ef5206980c5a6661845436058f30600c52df7";
    let h: H256 = H256::from_str(tx_hash)?;
    let options: GethDebugTracingOptions = GethDebugTracingOptions::default();
    let traces = client.debug_trace_transaction(h, options).await?;
    println!("{:?}", traces);

    Ok(())
}
