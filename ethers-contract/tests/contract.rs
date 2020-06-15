use ethers_contract::ContractFactory;
use ethers_core::{
    types::{Address, H256},
    utils::Ganache,
};

mod common;
pub use common::*;

#[tokio::test]
async fn deploy_and_call_contract() {
    let (abi, bytecode) = compile();

    // launch ganache
    let _ganache = Ganache::new()
        .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
        .spawn();

    // Instantiate the clients. We assume that clients consume the provider and the wallet
    // (which makes sense), so for multi-client tests, you must clone the provider.
    let client = connect("380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc");
    let client2 = connect("cc96601bc52293b53c4736a12af9130abf347669b3813f9ec4cafdf6991b087e");

    // create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(abi, bytecode, &client);

    // `send` consumes the deployer so it must be cloned for later re-use
    // (practically it's not expected that you'll need to deploy multiple instances of
    // the _same_ deployer, so it's fine to clone here from a dev UX vs perf tradeoff)
    let deployer = factory.deploy("initial value".to_string()).unwrap();
    let contract = deployer.clone().send().await.unwrap();

    let get_value = contract.method::<_, String>("getValue", ()).unwrap();
    let last_sender = contract.method::<_, Address>("lastSender", ()).unwrap();

    // the initial value must be the one set in the constructor
    let value = get_value.clone().call().await.unwrap();
    assert_eq!(value, "initial value");

    // make a call with `client2`
    let _tx_hash = contract
        .connect(&client2)
        .method::<_, H256>("setValue", "hi".to_owned())
        .unwrap()
        .send()
        .await
        .unwrap();
    assert_eq!(last_sender.clone().call().await.unwrap(), client2.address());
    assert_eq!(get_value.clone().call().await.unwrap(), "hi");

    // we can also call contract methods at other addresses with the `at` call
    // (useful when interacting with multiple ERC20s for example)
    let contract2_addr = deployer.clone().send().await.unwrap().address();
    let contract2 = contract.at(contract2_addr);
    let init_value: String = contract2
        .method::<_, String>("getValue", ())
        .unwrap()
        .call()
        .await
        .unwrap();
    let init_address = contract2
        .method::<_, Address>("lastSender", ())
        .unwrap()
        .call()
        .await
        .unwrap();
    assert_eq!(init_address, Address::zero());
    assert_eq!(init_value, "initial value");
}
