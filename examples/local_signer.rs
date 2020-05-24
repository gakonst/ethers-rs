use ethers::{types::TransactionRequest, HttpProvider, MainnetWallet};
use std::convert::TryFrom;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // connect to the network
    let provider = HttpProvider::try_from("http://localhost:8545")?;

    // create a wallet and connect it to the provider
    let client = MainnetWallet::from_str(
        "15c42bf2987d5a8a73804a8ea72fb4149f88adf73e98fc3f8a8ce9f24fcb7774",
    )?
    .connect(&provider);

    // get the account's nonce (we abuse the Deref to access the provider's functions)
    let nonce = client.get_transaction_count(client.address(), None).await?;
    dbg!(nonce);

    // craft the transaction
    let tx = TransactionRequest {
        from: None,
        to: Some("986eE0C8B91A58e490Ee59718Cca41056Cf55f24".parse().unwrap()),
        gas: Some(21000.into()),
        gas_price: Some(100_000.into()),
        value: Some(10000.into()),
        data: Some(vec![].into()),
        nonce: Some(nonce),
    };

    // send it!
    let tx = client.sign_and_send_transaction(tx).await?;

    // get the mined tx
    let tx = client.get_transaction(tx.hash).await?;

    println!("{}", serde_json::to_string(&tx)?);

    Ok(())
}
