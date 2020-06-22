use anyhow::Result;
use ethers::{prelude::*, utils::Ganache};
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<()> {
    let ganache = Ganache::new().spawn();

    let wallet: Wallet = ganache.keys()[0].clone().into();
    let wallet2: Wallet = ganache.keys()[1].clone().into();

    // connect to the network
    let provider = Provider::<Http>::try_from(ganache.endpoint())?;

    // connect the wallet to the provider
    let client = wallet.connect(provider);

    // craft the transaction
    let tx = TransactionRequest::new().to(wallet2.address()).value(10000);

    // send it!
    let tx_hash = client.send_transaction(tx, None).await?;

    // get the mined tx
    let receipt = client.pending_transaction(tx_hash).await?;
    let tx = client.get_transaction(receipt.transaction_hash).await?;

    println!("Sent tx: {}\n", serde_json::to_string(&tx)?);
    println!("Tx receipt: {}", serde_json::to_string(&receipt)?);

    Ok(())
}
