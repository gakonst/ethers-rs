use anyhow::Result;
use ethers::{prelude::*, utils::Ganache};
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<()> {
    // fork mainnet
    let ganache = Ganache::new()
        .fork("https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27")
        .spawn();
    let from = ganache.addresses()[0].clone();
    // connect to the network
    let provider = Provider::<Http>::try_from(ganache.endpoint()).unwrap().with_sender(from);

    // craft the transaction
    let tx = TransactionRequest::new().to("vitalik.eth").value(100_000);

    // send it!
    let receipt = provider
        .send_transaction(tx, None)
        .await?
        .await?
        .ok_or_else(|| anyhow::format_err!("tx dropped from mempool"))?;
    let tx = provider.get_transaction(receipt.transaction_hash).await?;

    println!("{}", serde_json::to_string(&tx)?);
    println!("{}", serde_json::to_string(&receipt)?);

    Ok(())
}
