use ethers::{
    types::{BlockNumber, TransactionRequest},
    HttpProvider,
};
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // connect to the network
    let provider = HttpProvider::try_from("http://localhost:8545")?;
    let from = "4916064D2E9C1b2ccC466EEc3d30B2b08F1C130D".parse()?;

    // craft the tx
    let tx = TransactionRequest {
        from,
        to: Some("9A7e5d4bcA656182e66e33340d776D1542143006".parse()?),
        value: Some(1000u64.into()),
        gas: None,
        gas_price: None,
        data: None,
        nonce: None,
    };

    // broadcast it via the eth_sendTransaction API
    let tx_hash = provider.send_transaction(tx).await?;

    let tx = provider.get_transaction(tx_hash).await?;

    println!("{}", serde_json::to_string(&tx)?);

    let nonce1 = provider
        .get_transaction_count(from, Some(BlockNumber::Latest))
        .await?;

    let nonce2 = provider
        .get_transaction_count(from, Some(BlockNumber::Number(0.into())))
        .await?;

    assert!(nonce2 < nonce1);

    Ok(())
}
