use anyhow::Result;
use ethers::{prelude::*, utils::Ganache};
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<()> {
    let ganache = Ganache::new().spawn();

    let wallet: LocalWallet = ganache.keys()[0].clone().into();
    let wallet2: LocalWallet = ganache.keys()[1].clone().into();

    // connect to the network
    let provider = Provider::<Http>::try_from(ganache.endpoint())?;

    // connect the wallet to the provider
    let client = SignerMiddleware::new(provider, wallet);

    // craft the transaction
    let tx = TransactionRequest::new().to(wallet2.address()).value(10000);

    // send it!
    let pending_tx = client.send_transaction(tx, None).await?;

    // get the mined tx
    let receipt =
        pending_tx.await?.ok_or_else(|| anyhow::format_err!("tx dropped from mempool"))?;
    let tx = client.get_transaction(receipt.transaction_hash).await?;

    println!("Sent tx: {}\n", serde_json::to_string(&tx)?);
    println!("Tx receipt: {}", serde_json::to_string(&receipt)?);

    Ok(())
}
