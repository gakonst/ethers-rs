use ethers::prelude::*;
use ethers::core::rand::thread_rng;
use ethers::signers::{LocalWallet};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()>{

    let ws = Ws::connect("your_api_key")
        .await
        .expect("no connection");

    println!("Is Connected: {}", ws.ready());

    //generates a brand new keystore with key
    //name your keystore file, set the path, set the password
    //*NOTE* -- pls don't store your passwords in a plain text file, this is just an example 
    let (signing_key, _) = LocalWallet::new_keystore("C:/encryption_path", &mut thread_rng(), "PASSWORD", Some("my_encrypted_keys"))
        .expect("key store fail");
    
    println!("your signing key is: {:?}", signing_key);


    //decrypt your keystore given the filepath with the password you encrypted it with originally
    let decrypt_key_store = LocalWallet::decrypt_keystore("C:/decryption_path/file_name", "PASSWORD")
        .expect("decryption failed");

    println!("your signing key is: {:?}", decrypt_key_store);

    Ok(())
}
