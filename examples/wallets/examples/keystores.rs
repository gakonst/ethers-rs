use ethers::prelude::*;
use ethers::core::rand::thread_rng;
use ethers::signers::{LocalWallet};
use dotenv::dotenv;

use eyre::Result;


#[tokio::main]
async fn main() -> Result<()>{

    dotenv().ok();

    let api_key = std::env::var("API_KEY").expect("expected environmental variable");
    let encryption_path = std::env::var("ENCYPTION_PATH").expect("expected environmental variable");
    let decryption_path = std::env::var("DECRYPTION_PATH").expect("expected environmental variable");
    let password = std::env::var("PASSWORD").expect("expected environmental variable");

    let ws = Ws::connect(api_key)
        .await
        .expect("no connection");

    println!("Is Connected: {}", ws.ready());

    //generates a brand new keystore with key
    //name your keystore file, set the path, set the password
    //*NOTE* -- pls don't store your passwords in a plain text file, this is just an example 
    let key_store = LocalWallet::new_keystore(encryption_path, &mut thread_rng(), &password, Some("my_encrypted_keys"))
        .expect("key store fail");
    
    let (your_signing_key, _) = key_store; 

    println!("your signing key is: {:?}", your_signing_key);


    //decrypt your keystore given the filepath with the password you encrypted it with originally
    let decrypt_key_store = LocalWallet::decrypt_keystore(decryption_path, &password)
        .expect("decryption failed");

    println!("your signing key is: {:?}", decrypt_key_store);

    Ok(())
}
