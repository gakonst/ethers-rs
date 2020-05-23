use ethers::providers::{Provider, ProviderTrait};
use ethers::types::{Address, BlockNumber, TransactionRequest};
use std::convert::TryFrom;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    let provider = Provider::try_from("http://localhost:8545")?;

    let from = Address::from_str("4916064D2E9C1b2ccC466EEc3d30B2b08F1C130D")?;

    let tx_hash = provider
        .send_transaction(TransactionRequest {
            from,
            to: Some(Address::from_str(
                "9A7e5d4bcA656182e66e33340d776D1542143006",
            )?),
            value: Some(1000u64.into()),
            gas: None,
            gas_price: None,
            data: None,
            nonce: None,
        })
        .await?;

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
