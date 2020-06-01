use ethers_contract::{Contract, ContractFactory};
use ethers_core::{
    types::H256,
    utils::{GanacheBuilder, Solc},
};
use ethers_providers::{Http, Provider};
use ethers_signers::Wallet;
use std::convert::TryFrom;

#[tokio::test]
async fn deploy_and_call_contract() {
    // 1. compile the contract
    let compiled = Solc::new("./tests/contract.sol").build().unwrap();
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
        .parse::<Wallet>()
        .unwrap();

    // 4. connect to the network
    let provider = Provider::<Http>::try_from(url.as_str()).unwrap();

    // 5. instantiate the client with the wallet
    let client = wallet.connect(provider);

    // 6. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(&client, &contract.abi, &contract.bytecode);

    // 7. deploy it with the constructor arguments
    let contract = factory
        .deploy("initial value".to_string())
        .unwrap()
        .send()
        .await
        .unwrap();

    // 8. get the contract's address
    let addr = contract.address();

    // 9. instantiate the contract
    let contract = Contract::new(*addr, contract.abi(), &client);

    // 10. the initial value must be the one set in the constructor
    let value: String = contract
        .method("getValue", ())
        .unwrap()
        .call()
        .await
        .unwrap();
    assert_eq!(value, "initial value");

    // 11. call the `setValue` method (ugly API here)
    let _tx_hash = contract
        .method::<_, H256>("setValue", "hi".to_owned())
        .unwrap()
        .send()
        .await
        .unwrap();

    // 12. get the new value
    let value: String = contract
        .method("getValue", ())
        .unwrap()
        .call()
        .await
        .unwrap();
    assert_eq!(value, "hi");
}
