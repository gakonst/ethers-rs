use ethers::{prelude::*, utils::Ganache};
use eyre::Result;
use std::{convert::TryFrom, sync::Arc, time::Duration};

// Generate the type-safe contract bindings by providing the json artifact
// *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact:
// `{"abi": [..], "bin": "..."}`  or `{"abi": [..], "bytecode": {"object": "..."}}`
// this will embedd the bytecode in a variable `GREETER_BYTECODE`
abigen!(Greeter, "ethers-contract/tests/solidity-contracts/greeter.json",);

#[tokio::main]
async fn main() -> Result<()> {
    // 1. compile the contract (note this requires that you are inside the `examples` directory) and
    // launch ganache
    let ganache = Ganache::new().spawn();

    // 2. instantiate our wallet
    let wallet: LocalWallet = ganache.keys()[0].clone().into();

    // 3. connect to the network
    let provider =
        Provider::<Http>::try_from(ganache.endpoint())?.interval(Duration::from_millis(10u64));

    // 4. instantiate the client with the wallet
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    // 5. deploy contract, note the `legacy` call required for non EIP-1559
    let greeter_contract =
        Greeter::deploy(client, "Hello World!".to_string()).unwrap().legacy().send().await.unwrap();

    // 6. call contract function
    let greeting = greeter_contract.greet().call().await.unwrap();
    assert_eq!("Hello World!", greeting);

    Ok(())
}
