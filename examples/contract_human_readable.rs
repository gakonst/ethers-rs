use anyhow::Result;
use ethers::{prelude::*, utils::Ganache};
use ethers_solc::{ArtifactOutput, Project, ProjectCompileOutput, ProjectPathsConfig};
use std::{convert::TryFrom, path::PathBuf, sync::Arc, time::Duration};

// Generate the type-safe contract bindings by providing the ABI
// definition in human readable format
abigen!(
    SimpleContract,
    r#"[
        function setValue(string)
        function getValue() external view returns (string)
        event ValueChanged(address indexed author, string oldValue, string newValue)
    ]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);

#[tokio::main]
async fn main() -> Result<()> {
    // the directory we use is root-dir/examples
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    // we use `root` for both the project root and for where to search for contracts since
    // everything is in the same directory
    let paths = ProjectPathsConfig::builder().root(&root).sources(&root).build().unwrap();
    // get the solc project instance using the paths above
    let solc = Project::builder()
        .paths(paths)
        .ephemeral()
        .artifacts(ArtifactOutput::Nothing)
        .build()
        .unwrap();
    // compile the project and get the artifacts
    let compiled = solc.compile().unwrap();
    let compiled = match compiled {
        ProjectCompileOutput::Compiled((output, _)) => output,
        _ => panic!("expected compilation artifacts"),
    };
    let path = root.join("contract.sol");
    let path = path.to_str();
    let contract = compiled.get(path.unwrap(), "SimpleStorage").expect("could not find contract");

    // 2. instantiate our wallet & ganache
    let ganache = Ganache::new().spawn();
    let wallet: LocalWallet = ganache.keys()[0].clone().into();

    // 3. connect to the network
    let provider =
        Provider::<Http>::try_from(ganache.endpoint())?.interval(Duration::from_millis(10u64));

    // 4. instantiate the client with the wallet
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    // 5. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(
        contract.abi.unwrap().clone(),
        contract.bin.unwrap().clone(),
        client.clone(),
    );

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
