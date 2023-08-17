use ethers::{
    contract::{abigen, ContractFactory},
    middleware::SignerMiddleware,
    prelude::k256::ecdsa::SigningKey,
    providers::{Http, Provider},
    signers::{Signer, Wallet},
    solc::{Artifact, Project, ProjectPathsConfig},
    types::Address,
};
use eyre::Result;
use std::{path::PathBuf, sync::Arc, time::Duration};

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
    let project = Project::builder().paths(paths).ephemeral().no_artifacts().build().unwrap();
    // compile the project and get the artifacts
    let output = project.compile().unwrap();
    let contract = output.find_first("SimpleStorage").expect("could not find contract").clone();
    let (abi, bytecode, _) = contract.into_parts();

    // 2. instantiate our wallet and quorum with tessera
    let wallet: Wallet<SigningKey> =
        "0x8bbbb1b345af56b560a5b20bd4b0ed1cd8cc9958a16262bc75118453cb546df7".parse().unwrap();

    let quorum_endpoint = "http://localhost:20000";
    let chain_id: u64 = 1337;
    let private_for = Some(vec!["BULeR8JyUWhiuuCMU/HLA0Q5pzkYT+cHII3ZKBey3Bo=".to_string()]);

    // 3. connect to the network
    let provider =
        Provider::<Http>::try_from(quorum_endpoint)?.interval(Duration::from_millis(10u64));
    let signer = wallet.with_chain_id(chain_id);

    // 4. instantiate the client with the wallet
    let client = SignerMiddleware::new(provider, signer);
    let client = Arc::new(client);

    // 5. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(abi.unwrap(), bytecode.unwrap(), client.clone());

    // 6. deploy it with the constructor arguments
    let contract = factory.deploy("initial value".to_string())?.send(private_for.clone()).await?;

    // 7. get the contract's address
    let addr = contract.address();

    // 8. instantiate the contract
    let contract = SimpleContract::new(addr, client.clone());

    // 9. call the `setValue` method
    // (first `await` returns a PendingTransaction, second one waits for it to be mined)
    let _receipt = contract.set_value("hi".to_owned()).send(private_for).await?.await?;

    // 10. get all events
    let logs = contract.value_changed_filter().from_block(0u64).query().await?;

    // 11. get the new value
    let value = contract.get_value().call().await?;

    println!("Value: {value}. Logs: {}", serde_json::to_string(&logs)?);

    Ok(())
}
