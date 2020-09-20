use anyhow::Result;
use ethers::{utils::parse_ether, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    // Connect over websockets
    let provider = Provider::new(Ws::connect("ws://localhost:8545").await?);
    // Instantiate the connection to ledger with Ledger Live derivation path and
    // the wallet's index. Alternatively, you may use Legacy with the wallet's
    // index or supply the  full HD path string. You may also provide the chain_id
    // (here: mainnet) for EIP155 support.
    let ledger = Ledger::new(HDPath::LedgerLive(0), Some(1)).await?;
    let client = Client::new(provider, ledger).await?;

    // Create and broadcast a transaction (ENS enabled!)
    // (this will require confirming the tx on the device)
    let tx = TransactionRequest::new()
        .to("vitalik.eth")
        .value(parse_ether(10)?);
    let tx_hash = client.send_transaction(tx, None).await?;

    // Get the receipt
    let receipt = client.pending_transaction(tx_hash).confirmations(3).await?;
    Ok(())
}
