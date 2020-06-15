use ethers_core::types::H256;
use ethers_providers::StreamExt;

mod common;
use common::{compile, connect, deploy, ValueChanged};

#[tokio::test]
async fn watch_events() {
    let (abi, bytecode) = compile();
    let client = connect("380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc");
    let (_ganache, contract) = deploy(&client, abi, bytecode).await;

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
