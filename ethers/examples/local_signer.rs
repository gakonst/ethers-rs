use anyhow::Result;
use ethers::{prelude::*, utils::Ganache};
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<()> {
    let port = 8545u64;
    let url = format!("http://localhost:{}", port).to_string();
    let _ganache = Ganache::new()
        .port(port)
        .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
        .spawn();

    // this private key belongs to the above mnemonic
    let wallet: Wallet =
        "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc".parse()?;

    // connect to the network
    let provider = Provider::<Http>::try_from(url.as_str())?;

    // connect the wallet to the provider
    let client = wallet.connect(provider);

    // craft the transaction
    let tx = TransactionRequest::new()
        .send_to_str("986eE0C8B91A58e490Ee59718Cca41056Cf55f24")?
        .value(10000);

    // send it!
    let tx_hash = client.send_transaction(tx, None).await?;

    // get the mined tx
    let receipt = client.pending_transaction(tx_hash).await?;
    let tx = client.get_transaction(receipt.transaction_hash).await?;

    println!("Sent tx: {}\n", serde_json::to_string(&tx)?);
    println!("Tx receipt: {}", serde_json::to_string(&receipt)?);

    Ok(())
}
