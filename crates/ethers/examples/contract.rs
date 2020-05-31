use anyhow::Result;
use ethers::{
    contract::{abigen, ContractFactory},
    providers::HttpProvider,
    signers::MainnetWallet,
    types::utils::{GanacheBuilder, Solc},
};
use std::convert::TryFrom;

// Generate the contract bindings by providing the ABI
abigen!(
    SimpleContract,
    r#"[{"inputs":[{"internalType":"string","name":"value","type":"string"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"author","type":"address"},{"indexed":false,"internalType":"string","name":"oldValue","type":"string"},{"indexed":false,"internalType":"string","name":"newValue","type":"string"}],"name":"ValueChanged","type":"event"},{"inputs":[],"name":"getValue","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","constant": true, "type":"function"},{"inputs":[{"internalType":"string","name":"value","type":"string"}],"name":"setValue","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);

#[tokio::main]
async fn main() -> Result<()> {
    // 1. compile the contract (note this requires that you are inside the `ethers/examples` directory)
    let compiled = Solc::new("./contract.sol").build()?;
    let contract = compiled
        .get("SimpleStorage")
        .expect("could not find contract");

    // 2. launch ganache
    let port = 8546u64;
    let url = format!("http://localhost:{}", port).to_string();
    let _ganache = GanacheBuilder::new().port(port)
        .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
        .spawn();

    // 3. instantiate our wallet
    let wallet = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
        .parse::<MainnetWallet>()?;

    // 4. connect to the network
    let provider = HttpProvider::try_from(url.as_str())?;

    // 5. instantiate the client with the wallet
    let client = wallet.connect(&provider);

    // 6. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(&client, &contract.abi, &contract.bytecode);

    // 7. deploy it with the constructor arguments
    let contract = factory.deploy("initial value".to_string())?.send().await?;

    // 8. get the contract's address
    let addr = contract.address();

    // 9. instantiate the contract
    let contract = SimpleContract::new(*addr, &client);

    // 10. call the `setValue` method
    let _tx_hash = contract.set_value("hi".to_owned()).send().await?;

    // 11. get all events
    let logs = contract.value_changed().from_block(0u64).query().await?;

    // 12. get the new value
    let value = contract.get_value().call().await?;

    println!("Value: {}. Logs: {}", value, serde_json::to_string(&logs)?);

    Ok(())
}
