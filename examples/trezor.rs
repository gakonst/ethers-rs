#[tokio::main]
#[cfg(feature = "trezor")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ethers::{prelude::*, utils::parse_ether};

    // Connect over websockets
    let provider = Provider::new(Ws::connect("ws://localhost:8545").await?);
    // Instantiate the connection to trezor with Trezor Live derivation path and
    // the wallet's index. You may also provide the chain_id.
    // (here: mainnet) for EIP155 support.
    // EIP1559 support
    // No EIP712 support yet.
    let trezor = Trezor::new(TrezorHDPath::TrezorLive(0), 1, None).await?;
    let client = SignerMiddleware::new(provider, trezor);

    // Create and broadcast a transaction (ENS disabled!)
    // (this will require confirming the tx on the device)
    let tx = TransactionRequest::new()
        .to("0x99E2B13A8Ea8b00C68FA017ee250E98e870D8241")
        .value(parse_ether(10)?);
    let pending_tx = client.send_transaction(tx, None).await?;

    // Get the receipt
    let _receipt = pending_tx.confirmations(3).await?;
    Ok(())
}

#[cfg(not(feature = "trezor"))]
fn main() {}
