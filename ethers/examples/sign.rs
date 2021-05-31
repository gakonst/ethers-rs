//use the anyhow crate for easy idiomatic error handling
use anyhow::Result;
//use the ethers_core rand crate to manage creation of random numbers
use ethers_core::rand::thread_rng;
//use the ethers_signers crate to manage LocalWallet and Signer
use ethers_signers::{LocalWallet, Signer};

//Declare tokio because our main function uses future(async)
#[tokio::main]
async fn main() -> Result<()> {
    //Create random numbers for your wallet.
    let wallet = LocalWallet::new(&mut thread_rng());
    
    //Declare message you want to sign.
    let message = "Some data";

    // sign message from your wallet and print out signature produced.
    let signature = wallet.sign_message(message).await?;
    println!("Produced signature {}", signature);

    // verify the signature produced from your wallet.
    signature.verify(message, wallet.address()).unwrap();
    println!("Verified signature produced by {:?}!", wallet.address());

    Ok(())
}
