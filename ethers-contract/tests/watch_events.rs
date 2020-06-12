use ethers_contract::ContractFactory;
use ethers_core::{
    types::H256,
    utils::{Ganache, Solc},
};
use ethers_providers::{Http, Provider, StreamExt};
use ethers_signers::Wallet;
use std::convert::TryFrom;

mod test_helpers;
use test_helpers::ValueChanged;

#[tokio::test]
async fn watch_events() {
    // compile the contract
    let compiled = Solc::new("./tests/contract.sol").build().unwrap();
    let contract = compiled
        .get("SimpleStorage")
        .expect("could not find contract");

    // launch ganache
    let port = 8545u64;
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

    // We spawn the event listener:
    let mut stream = contract
        .event::<ValueChanged>("ValueChanged")
        .unwrap()
        .stream()
        .await
        .unwrap();

    let num_calls = 3u64;

    // and we make a few calls
    for i in 0..num_calls {
        let _tx_hash = contract
            .method::<_, H256>("setValue", i.to_string())
            .unwrap()
            .send()
            .await
            .unwrap();
    }

    for i in 0..num_calls {
        // unwrap the option of the stream, then unwrap the decoding result
        let log = stream.next().await.unwrap().unwrap();
        assert_eq!(log.new_value, i.to_string());
    }
}
