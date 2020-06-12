use ethers_contract::ContractFactory;
use ethers_core::{
    types::H256,
    utils::{Ganache, Solc},
};
use ethers_providers::{Http, Provider};
use ethers_signers::Wallet;
use std::convert::TryFrom;

mod test_helpers;
use test_helpers::ValueChanged;

#[tokio::test]
async fn get_past_events() {
    // compile the contract
    let compiled = Solc::new("./tests/contract.sol").build().unwrap();
    let contract = compiled
        .get("SimpleStorage")
        .expect("could not find contract");

    // launch ganache
    let port = 8546u64;
    let url = format!("http://localhost:{}", port).to_string();
    let _ganache = Ganache::new().port(port)
        .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
        .spawn();

    // connect to the network
    let provider = Provider::<Http>::try_from(url.as_str()).unwrap();
    let client = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
        .parse::<Wallet>()
        .unwrap()
        .connect(provider);

    let factory = ContractFactory::new(&contract.abi, &contract.bytecode, &client);
    let contract = factory
        .deploy("initial value".to_string())
        .unwrap()
        .send()
        .await
        .unwrap();

    // make a call with `client2`
    let _tx_hash = contract
        .method::<_, H256>("setValue", "hi".to_owned())
        .unwrap()
        .send()
        .await
        .unwrap();

    // and we can fetch the events
    let logs: Vec<ValueChanged> = contract
        .event("ValueChanged")
        .unwrap()
        .from_block(0u64)
        .topic1(client.address()) // Corresponds to the first indexed parameter
        .query()
        .await
        .unwrap();
    assert_eq!(logs[0].new_value, "initial value");
    assert_eq!(logs[1].new_value, "hi");
    assert_eq!(logs.len(), 2);
}
