use ethers_core::types::H256;

mod common;
use common::{compile, connect, deploy, ValueChanged};

#[tokio::test]
async fn get_past_events() {
    let (abi, bytecode) = compile();
    let client = connect("380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc");
    let (_ganache, contract) = deploy(&client, abi, bytecode).await;

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
