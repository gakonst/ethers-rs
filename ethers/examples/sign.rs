use anyhow::Result;
use ethers::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let message = "Some data";
    let wallet = Wallet::new(&mut rand::thread_rng());

    // sign a message
    let signature = wallet.sign_message(message).await?;
    println!("Produced signature {}", signature);

    // verify the signature
    signature.verify(message, wallet.address()).unwrap();

    println!("Verified signature produced by {:?}!", wallet.address());

    Ok(())
}
