use ethers::{prelude::*, utils::Anvil};
use eyre::Result;
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<()> {
    let anvil = Anvil::new().spawn();

    // connect to the network
    let provider = Provider::<Http>::try_from(anvil.endpoint())?;
    let accounts = provider.get_accounts().await?;
    let from = accounts[0];
    let to = accounts[1];

    // craft the tx
    let tx = TransactionRequest::new().to(to).value(1000).from(from); // specify the `from` field so that the client knows which account to use

    let balance_before = provider.get_balance(from, None).await?;
    let nonce1 = provider.get_transaction_count(from, None).await?;

    // broadcast it via the eth_sendTransaction API
    let tx = provider.send_transaction(tx, None).await?.await?;

    println!("{}", serde_json::to_string(&tx)?);

    let nonce2 = provider.get_transaction_count(from, None).await?;

    assert!(nonce1 < nonce2);

    let balance_after = provider.get_balance(from, None).await?;
    assert!(balance_after < balance_before);

    println!("Balance before {balance_before}");
    println!("Balance after {balance_after}");

    Ok(())
}
