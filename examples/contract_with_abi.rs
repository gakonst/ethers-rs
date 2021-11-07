use anyhow::Result;
use ethers::{prelude::*, utils::Ganache};
use std::{convert::TryFrom, path::Path, sync::Arc, time::Duration};

// Generate the type-safe contract bindings by providing the ABI
// definition in human readable format
abigen!(
    SimpleContract,
    "./examples/contract_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

#[tokio::main]
async fn main() -> Result<()> {
    // 1. compile the contract (note this requires that you are inside the `examples` directory) and
    // launch ganache
    let ganache = Ganache::new().spawn();

    // set the path to the contract, `CARGO_MANIFEST_DIR` points to the directory containing the
    // manifest of `ethers`. which will be `../` relative to this file
    let source = Path::new(&env!("CARGO_MANIFEST_DIR")).join("examples/contract.sol");
    let compiled = Solc::default().compile_source(source).expect("Could not compile contracts");
    let (abi, bytecode, _runtime_bytecode) =
        compiled.find("SimpleStorage").expect("could not find contract").into_parts_or_default();

    // 2. instantiate our wallet
    let wallet: LocalWallet = ganache.keys()[0].clone().into();

    // 3. connect to the network
    let provider =
        Provider::<Http>::try_from(ganache.endpoint())?.interval(Duration::from_millis(10u64));

    // 4. instantiate the client with the wallet
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    // 5. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(abi, bytecode, client.clone());

    // 6. deploy it with the constructor arguments
    let contract = factory.deploy("initial value".to_string())?.legacy().send().await?;

    // 7. get the contract's address
    let addr = contract.address();

    // 8. instantiate the contract
    let contract = SimpleContract::new(addr, client.clone());

    // 9. call the `setValue` method
    // (first `await` returns a PendingTransaction, second one waits for it to be mined)
    let _receipt = contract.set_value("hi".to_owned()).legacy().send().await?.await?;

    // 10. get all events
    let logs = contract.value_changed_filter().from_block(0u64).query().await?;

    // 11. get the new value
    let value = contract.get_value().call().await?;

    println!("Value: {}. Logs: {}", value, serde_json::to_string(&logs)?);

    Ok(())
}
