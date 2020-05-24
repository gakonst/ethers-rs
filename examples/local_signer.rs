use ethers::{types::TransactionRequest, HttpProvider, MainnetWallet};
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // connect to the network
    let provider = HttpProvider::try_from("http://localhost:8545")?;

    // create a wallet and connect it to the provider
    let client = "15c42bf2987d5a8a73804a8ea72fb4149f88adf73e98fc3f8a8ce9f24fcb7774"
        .parse::<MainnetWallet>()?
        .connect(&provider);

    // craft the transaction
    let tx = TransactionRequest::new()
        .send_to_str("986eE0C8B91A58e490Ee59718Cca41056Cf55f24")?
        .value(10000);

    // send it!
    let tx = client.sign_and_send_transaction(tx, None).await?;

    // get the mined tx
    let tx = client.get_transaction(tx.hash).await?;

    println!("{}", serde_json::to_string(&tx)?);

    Ok(())
}
