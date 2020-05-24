use ethers::{MainnetWallet as Wallet, Signer};

fn main() {
    let message = "Some data";
    let wallet = Wallet::new(&mut rand::thread_rng());

    // sign a message
    let signature = wallet.sign_message(message);
    println!("Produced signature {}", signature);

    // recover the address that signed it
    let recovered = signature.recover(message).unwrap();

    assert_eq!(recovered, wallet.address);

    println!("Verified signature produced by {:?}!", wallet.address);
}
