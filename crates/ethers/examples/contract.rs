use ethers::{
    abi::{Detokenize, InvalidOutputType, Token},
    contract::{Contract, Event, Sender},
    providers::{HttpProvider, JsonRpcClient},
    signers::{Client, MainnetWallet, Signer},
    types::{Address, H256},
};

use anyhow::Result;
use serde::Serialize;
use std::convert::TryFrom;

const ABI: &'static str = r#"[{"inputs":[{"internalType":"string","name":"value","type":"string"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"author","type":"address"},{"indexed":false,"internalType":"string","name":"oldValue","type":"string"},{"indexed":false,"internalType":"string","name":"newValue","type":"string"}],"name":"ValueChanged","type":"event"},{"inputs":[],"name":"getValue","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"string","name":"value","type":"string"}],"name":"setValue","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;

// abigen!(SimpleContract, ABI);

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

struct SimpleContract<'a, S, P>(Contract<'a, S, P>);

impl<'a, S: Signer, P: JsonRpcClient> SimpleContract<'a, S, P> {
    fn new<T: Into<Address>>(address: T, client: &'a Client<'a, S, P>) -> Self {
        let contract = Contract::new(client, serde_json::from_str(&ABI).unwrap(), address.into());
        Self(contract)
    }

    fn set_value<T: Into<String>>(&self, val: T) -> Sender<'a, S, P, H256> {
        self.0
            .method("setValue", Some(val.into()))
            .expect("method not found (this should never happen)")
    }

    fn value_changed<'b>(&'a self) -> Event<'a, 'b, P, ValueChanged>
    where
        'a: 'b,
    {
        self.0.event("ValueChanged").expect("event does not exist")
    }

    fn get_value(&self) -> Sender<'a, S, P, String> {
        self.0
            .method("getValue", None::<()>)
            .expect("method not found (this should never happen)")
    }
}

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
    let _tx_hash = contract.set_value("hi").send().await?;

    let logs = contract.value_changed().from_block(0u64).query().await?;

    let value = contract.get_value().call().await?;

    println!("Value: {}. Logs: {}", value, serde_json::to_string(&logs)?);

    Ok(())
}
