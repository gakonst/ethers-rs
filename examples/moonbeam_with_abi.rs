use ethers::prelude::*;

abigen!(
    SimpleContract,
    "./examples/contract_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

/// This requires a running moonbeam dev instance on `localhost:9933`
/// See `https://docs.moonbeam.network/builders/get-started/moonbeam-dev/` for reference
///
/// This has been tested against:
///
/// ```bash
///  docker run --rm --name moonbeam_development -p 9944:9944 -p 9933:9933 purestake/moonbeam:v0.14.2 --dev --ws-external --rpc-external
/// ```
///
/// Also requires the `legacy` feature to send Legacy transaction instead of an EIP-1559
#[tokio::main]
#[cfg(feature = "legacy")]
async fn main() -> eyre::Result<()> {
    use std::{convert::TryFrom, path::Path, sync::Arc, time::Duration};
    const MOONBEAM_DEV_ENDPOINT: &str = "http://localhost:9933";

    // set the path to the contract, `CARGO_MANIFEST_DIR` points to the directory containing the
    // manifest of `ethers`. which will be `../` relative to this file
    let source = Path::new(&env!("CARGO_MANIFEST_DIR")).join("examples/contract.sol");
    let compiled = Solc::default().compile_source(source).expect("Could not compile contracts");
    let (abi, bytecode, _runtime_bytecode) =
        compiled.find("SimpleStorage").expect("could not find contract").into_parts_or_default();

    // 1. get a moonbeam dev key
    let key = ethers::core::utils::moonbeam::dev_keys()[0].clone();

    // 2. instantiate our wallet with chain id
    let wallet: LocalWallet = LocalWallet::from(key).with_chain_id(Chain::MoonbeamDev);

    // 3. connect to the network
    let provider =
        Provider::<Http>::try_from(MOONBEAM_DEV_ENDPOINT)?.interval(Duration::from_millis(10u64));

    // 4. instantiate the client with the wallet
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    // 5. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(abi, bytecode, client.clone());

    // 6. deploy it with the constructor arguments, note the `legacy` call
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

    println!("Value: {value}. Logs: {}", serde_json::to_string(&logs)?);

    Ok(())
}

#[cfg(not(feature = "legacy"))]
fn main() {}
