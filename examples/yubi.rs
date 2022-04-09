#[tokio::main]
#[cfg(feature = "yubi")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ethers::{prelude::*, utils::parse_ether};
    use yubihsm::{Connector, Credentials, UsbConfig};

    // Connect over websockets
    let provider = Provider::new(Ws::connect("ws://localhost:8545").await?);

    // We use USB for the example, but you can connect over HTTP as well. Refer
    // to the [YubiHSM](https://docs.rs/yubihsm/0.34.0/yubihsm/) docs for more info
    let connector = Connector::usb(&UsbConfig::default());
    // Instantiate the connection to the YubiKey. Alternatively, use the
    // `from_key` method to upload a key you already have, or the `new` method
    // to generate a new keypair.
    let wallet = YubiWallet::connect(connector, Credentials::default(), 0);
    let client = SignerMiddleware::new(provider, wallet);

    // Create and broadcast a transaction (ENS enabled!)
    let tx = TransactionRequest::new().to("vitalik.eth").value(parse_ether(10)?);
    let pending_tx = client.send_transaction(tx, None).await?;

    // Get the receipt
    let _receipt = pending_tx.confirmations(3).await?;
    Ok(())
}

#[cfg(not(feature = "yubi"))]
fn main() {}
