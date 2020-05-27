use anyhow::Result;
use ethers::{providers::HttpProvider, signers::MainnetWallet, types::TransactionRequest};
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<()> {
    // connect to the network
    let provider =
        HttpProvider::try_from("https://mainnet.infura.io/v3/9408f47dedf04716a03ef994182cf150")?;

    // create a wallet and connect it to the provider
    let client = "dcf2cbdd171a21c480aa7f53d77f31bb102282b3ff099c78e3118b37348c72f7"
        .parse::<MainnetWallet>()?
        .connect(&provider);

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
