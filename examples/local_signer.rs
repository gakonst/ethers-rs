use ethers::{types::UnsignedTransaction, HttpProvider, MainnetWallet};
use std::convert::TryFrom;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    let provider = HttpProvider::try_from("http://localhost:8545")?;
    let client = MainnetWallet::from_str(
        "d8ebe1e50cfea1f9961908d9df28e64bb163fee9ee48320361b2eb0a54974269",
    )?
    .connect(&provider);

    let nonce = provider
        .get_transaction_count(client.signer.address, None)
        .await?;

    let tx = UnsignedTransaction {
        to: Some("986eE0C8B91A58e490Ee59718Cca41056Cf55f24".parse().unwrap()),
        gas: 21000.into(),
        gas_price: 100_000.into(),
        value: 10000.into(),
        input: vec![].into(),
        nonce,
    };

    let tx = client.send_transaction(tx).await?;

    let tx = client.get_transaction(tx.hash).await?;

    println!("{}", serde_json::to_string(&tx)?);

    Ok(())
}
