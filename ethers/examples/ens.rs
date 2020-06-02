use anyhow::Result;
use ethers::prelude::*;
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<()> {
    // connect to the network
    let provider = Provider::<Http>::try_from(
        "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
    )?;

    // create a wallet and connect it to the provider
    let client = "dcf2cbdd171a21c480aa7f53d77f31bb102282b3ff099c78e3118b37348c72f7"
        .parse::<Wallet>()?
        .connect(provider);

    // craft the transaction
    let tx = TransactionRequest::new().to("vitalik.eth").value(100_000);

    // send it!
    let hash = client.send_transaction(tx, None).await?;

    // get the mined tx
    let tx = client.get_transaction(hash).await?;

    let receipt = client.get_transaction_receipt(tx.hash).await?;

    println!("{}", serde_json::to_string(&tx)?);
    println!("{}", serde_json::to_string(&receipt)?);

    Ok(())
}
