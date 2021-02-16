use anyhow::Result;
use ethers::{
    prelude::*,
    utils::{Ganache, Solc},
};
use std::{convert::TryFrom, sync::Arc, time::Duration};

// Generate the type-safe contract bindings by providing the ABI
// definition in human readable format
abigen!(
    SimpleContract,
    r#"[
        function setValue(string)
        function getValue() external view (string)
        event ValueChanged(address indexed author, string oldValue, string newValue)
    ]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);

#[tokio::main]
async fn main() -> Result<()> {
    // 1. compile the contract (note this requires that you are inside the `ethers/examples` directory)
    let compiled = Solc::new("**/contract.sol").build()?;
    let contract = compiled
        .get("SimpleStorage")
        .expect("could not find contract");

    // 2. launch ganache
    let ganache = Ganache::new().spawn();

    // 3. instantiate our wallet
    let wallet: LocalWallet = ganache.keys()[0].clone().into();

    // 4. connect to the network
    let provider =
        Provider::<Http>::try_from(ganache.endpoint())?.interval(Duration::from_millis(10u64));

    // 5. instantiate the client with the wallet
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    // 6. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(
        contract.abi.clone(),
        contract.bytecode.clone(),
        client.clone(),
    );

    // 7. deploy it with the constructor arguments
    let contract = factory.deploy("initial value".to_string())?.send().await?;

    // 8. get the contract's address
    let addr = contract.address();

    // 9. instantiate the contract
    let contract = SimpleContract::new(addr, client.clone());

    // 10. call the `setValue` method
    // (first `await` returns a PendingTransaction, second one waits for it to be mined)
    let _receipt = contract.set_value("hi".to_owned()).send().await?.await?;

    // 11. get all events
    let logs = contract
        .value_changed_filter()
        .from_block(0u64)
        .query()
        .await?;

    // 12. get the new value
    let value = contract.get_value().call().await?;

    println!("Value: {}. Logs: {}", value, serde_json::to_string(&logs)?);

    Ok(())
}
