use ethers_contract::ContractFactory;
use ethers_core::{
    abi::{Detokenize, InvalidOutputType, Token},
    types::{Address, H256},
    utils::{Ganache, Solc},
};
use ethers_providers::{Http, Provider};
use ethers_signers::Wallet;
use std::convert::TryFrom;

#[tokio::test]
async fn deploy_and_call_contract() {
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

    // instantiate our wallets
    let [wallet1, wallet2]: [Wallet; 2] = [
        "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
            .parse()
            .unwrap(),
        "cc96601bc52293b53c4736a12af9130abf347669b3813f9ec4cafdf6991b087e"
            .parse()
            .unwrap(),
    ];

    // Instantiate the clients. We assume that clients consume the provider and the wallet
    // (which makes sense), so for multi-client tests, you must clone the provider.
    let client = wallet1.connect(provider.clone());
    let client2 = wallet2.connect(provider);

    // create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(&contract.abi, &contract.bytecode, &client);

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

    // we can still interact with the old contract instance
    let _tx_hash = contract
        .method::<_, H256>("setValue", "hi2".to_owned())
        .unwrap()
        .send()
        .await
        .unwrap();
    assert_eq!(last_sender.clone().call().await.unwrap(), client.address());
    assert_eq!(get_value.clone().call().await.unwrap(), "hi2");

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
    assert_eq!(logs[1].new_value, "hi2");
    assert_eq!(logs.len(), 2);

    let logs: Vec<ValueChanged> = contract2
        .event("ValueChanged")
        .unwrap()
        .from_block(0u64)
        .query()
        .await
        .unwrap();
    assert_eq!(logs[0].new_value, "initial value");
    assert_eq!(logs.len(), 1);
}

// Note: We also provide the `abigen` macro for generating these bindings automatically
#[derive(Clone, Debug)]
struct ValueChanged {
    old_author: Address,
    new_author: Address,
    old_value: String,
    new_value: String,
}

impl Detokenize for ValueChanged {
    fn from_tokens(tokens: Vec<Token>) -> Result<ValueChanged, InvalidOutputType> {
        let old_author: Address = tokens[1].clone().to_address().unwrap();
        let new_author: Address = tokens[1].clone().to_address().unwrap();
        let old_value = tokens[2].clone().to_string().unwrap();
        let new_value = tokens[3].clone().to_string().unwrap();

        Ok(Self {
            old_author,
            new_author,
            old_value,
            new_value,
        })
    }
}
