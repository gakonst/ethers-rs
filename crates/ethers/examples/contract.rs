use anyhow::Result;
use ethers::{providers::HttpProvider, signers::MainnetWallet, types::Address};
use std::convert::TryFrom;

use ethers::contract::abigen;

// Generate the contract code
abigen!(
    SimpleContract,
    r#"[{"inputs":[{"internalType":"string","name":"value","type":"string"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"author","type":"address"},{"indexed":false,"internalType":"string","name":"oldValue","type":"string"},{"indexed":false,"internalType":"string","name":"newValue","type":"string"}],"name":"ValueChanged","type":"event"},{"inputs":[],"name":"getValue","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","constant": true, "type":"function"},{"inputs":[{"internalType":"string","name":"value","type":"string"}],"name":"setValue","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);

#[tokio::main]
async fn main() -> Result<()> {
    // connect to the network
    let provider = HttpProvider::try_from("http://localhost:8545")?;

    // create a wallet and connect it to the provider
    let client = "ea878d94d9b1ffc78b45fc7bfc72ec3d1ce6e51e80c8e376c3f7c9a861f7c214"
        .parse::<MainnetWallet>()?
        .connect(&provider);

    // Contract should take both provider or a signer

    // get the contract's address
    let addr = "ebBe15d9C365fC8a04a82E06644d6B39aF20cC31".parse::<Address>()?;

    // instantiate it
    let contract = SimpleContract::new(addr, &client);

    // call the method
    let _tx_hash = contract.set_value("hi".to_owned()).send().await?;

    let logs = contract.value_changed().from_block(0u64).query().await?;

    let value = contract.get_value().call().await?;

    println!("Value: {}. Logs: {}", value, serde_json::to_string(&logs)?);

    Ok(())
}
