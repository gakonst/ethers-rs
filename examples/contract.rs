use ethers::{
    abi::ParamType,
    contract::Contract,
    types::{Address, Filter},
    HttpProvider, MainnetWallet,
};
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // connect to the network
    let provider = HttpProvider::try_from("http://localhost:8545")?;

    // create a wallet and connect it to the provider
    let client = "d22cf25d564c3c3f99677f8710b2f045045f16eccd31140c92d6feb18c1169e9"
        .parse::<MainnetWallet>()?
        .connect(&provider);

    // Contract should take both provider or a signer

    // get the contract's address
    let addr = "683BEE23D79A1D8664dF70714edA966e1484Fd3d".parse::<Address>()?;

    // get the contract's ABI
    let abi = r#"[{"inputs":[{"internalType":"string","name":"value","type":"string"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"author","type":"address"},{"indexed":false,"internalType":"string","name":"oldValue","type":"string"},{"indexed":false,"internalType":"string","name":"newValue","type":"string"}],"name":"ValueChanged","type":"event"},{"inputs":[],"name":"getValue","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"string","name":"value","type":"string"}],"name":"setValue","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;

    // instantiate it
    let contract = Contract::new(&client, serde_json::from_str(abi)?, addr);

    // get the args
    let event = "ValueChanged(address,string,string)";

    let args = &[ethabi::Token::String("hello!".to_owned())];

    // call the method
    let tx_hash = contract.method("setValue", args)?.send().await?;

    #[derive(Clone, Debug)]
    struct ValueChanged {
        author: Address,
        old_value: String,
        new_value: String,
    }

    let filter = Filter::new().from_block(0).address(addr).event(event);
    let logs = provider
        .get_logs(&filter)
        .await?
        .into_iter()
        .map(|log| {
            // decode the non-indexed data
            let data = ethabi::decode(&[ParamType::String, ParamType::String], log.data.as_ref())?;

            let author = log.topics[1].into();

            // Unwrap?
            let old_value = data[0].clone().to_string().unwrap();
            let new_value = data[1].clone().to_string().unwrap();

            Ok(ValueChanged {
                old_value,
                new_value,
                author,
            })
        })
        .collect::<Result<Vec<_>, ethabi::Error>>()?;

    dbg!(logs);
    Ok(())
}
