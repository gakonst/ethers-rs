use ethers::{
    abi::{Detokenize, InvalidOutputType, Token},
    contract::Contract,
    providers::HttpProvider,
    signers::MainnetWallet,
    types::Address,
};

use anyhow::Result;
use serde::Serialize;
use std::convert::TryFrom;

const ABI: &'static str = r#"[{"inputs":[{"internalType":"string","name":"value","type":"string"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"author","type":"address"},{"indexed":false,"internalType":"string","name":"oldValue","type":"string"},{"indexed":false,"internalType":"string","name":"newValue","type":"string"}],"name":"ValueChanged","type":"event"},{"inputs":[],"name":"getValue","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"string","name":"value","type":"string"}],"name":"setValue","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;

#[derive(Clone, Debug, Serialize)]
// TODO: This should be `derive`-able on such types -> similar to how Zexe's Deserialize is done
struct ValueChanged {
    author: Address,
    old_value: String,
    new_value: String,
}

impl Detokenize for ValueChanged {
    fn from_tokens(tokens: Vec<Token>) -> Result<ValueChanged, InvalidOutputType> {
        let author: Address = tokens[0].clone().to_address().unwrap();
        let old_value = tokens[1].clone().to_string().unwrap();
        let new_value = tokens[2].clone().to_string().unwrap();

        Ok(Self {
            author,
            old_value,
            new_value,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // connect to the network
    let provider = HttpProvider::try_from("http://localhost:8545")?;

    // create a wallet and connect it to the provider
    let client = "d22cf25d564c3c3f99677f8710b2f045045f16eccd31140c92d6feb18c1169e9"
        .parse::<MainnetWallet>()?
        .connect(&provider);

    // Contract should take both provider or a signer

    // get the contract's address
    let addr = "683BEE23D79A1D8664dF70714edA966e1484Fd3d".parse::<Address>()?;

    // instantiate it
    let contract = Contract::new(&client, serde_json::from_str(ABI)?, addr);

    // call the method
    let _tx_hash = contract.method("setValue", "hi".to_owned())?.send().await?;

    let logs: Vec<ValueChanged> = contract
        .event("ValueChanged")?
        .from_block(0u64)
        .query()
        .await?;

    println!("{}", serde_json::to_string(&logs)?);
    Ok(())
}
