use anyhow::Result;
use ethers::{prelude::*, utils::Ganache};
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<()> {
    let port = 8546u64;
    let url = format!("http://localhost:{}", port).to_string();
    let _ganache = Ganache::new().port(port).spawn();

    // connect to the network
    let provider = Provider::<Http>::try_from(url.as_str())?;
    let accounts = provider.get_accounts().await?;
    let from = accounts[0];

    // craft the tx
    let tx = TransactionRequest::new()
        .send_to_str("9A7e5d4bcA656182e66e33340d776D1542143006")?
        .value(1000)
        .from(from); // specify the `from` field so that the client knows which account to use

    let balance_before = provider.get_balance(from, None).await?;

    // broadcast it via the eth_sendTransaction API
    let pending_tx = provider.send_transaction(tx).await?;

    let tx = pending_tx.await?;

    println!("{}", serde_json::to_string(&tx)?);

    let nonce1 = provider
        .get_transaction_count(from, Some(BlockNumber::Latest))
        .await?;

    let nonce2 = provider
        .get_transaction_count(from, Some(BlockNumber::Number(0.into())))
        .await?;

    assert!(nonce2 < nonce1);

    let balance_after = provider.get_balance(from, None).await?;
    assert!(balance_after < balance_before);

    println!("Balance before {}", balance_before);
    println!("Balance after {}", balance_after);

    Ok(())
}
