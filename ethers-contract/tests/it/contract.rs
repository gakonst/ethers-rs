use crate::common::*;
use ethers_contract::{
    abigen, ContractFactory, ContractInstance, EthEvent, LogMeta, Multicall, MulticallError,
    MulticallVersion,
};
use ethers_core::{
    abi::{encode, AbiEncode, Token, Tokenizable},
    types::{Address, BlockId, Bytes, Filter, ValueOrArray, H160, H256, U256},
    utils::{keccak256, Anvil},
};
use ethers_providers::{spoof, Http, Middleware, MiddlewareError, Provider, StreamExt, Ws};
use std::{sync::Arc, time::Duration};

#[derive(Debug)]
pub struct NonClone<M> {
    m: M,
}

#[derive(Debug)]
pub struct MwErr<M: Middleware>(M::Error);

impl<M> MiddlewareError for MwErr<M>
where
    M: Middleware,
{
    type Inner = M::Error;

    fn from_err(src: M::Error) -> Self {
        Self(src)
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        Some(&self.0)
    }
}

impl<M: Middleware> std::fmt::Display for MwErr<M> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
impl<M: Middleware> std::error::Error for MwErr<M> {}

impl<M: Middleware> Middleware for NonClone<M> {
    type Error = MwErr<M>;

    type Provider = M::Provider;

    type Inner = M;

    fn inner(&self) -> &Self::Inner {
        &self.m
    }
}

// this is not a test. It is a compile check. :)
// It exists to ensure that trait bounds on contract internal behave as
// expected. It should not be run
fn _it_compiles() {
    let (abi, _bytecode) = get_contract("SimpleStorage.json");

    // launch anvil
    let anvil = Anvil::new().spawn();

    let client = Provider::<Http>::try_from(anvil.endpoint())
        .unwrap()
        .interval(Duration::from_millis(10u64));

    // Works (B == M, M: Clone)
    let c: ContractInstance<&Provider<Http>, Provider<Http>> =
        ContractInstance::new(H160::default(), abi.clone(), &client);

    let _ = c.method::<(), ()>("notARealMethod", ());

    // Works (B == &M, M: Clone)
    let c: ContractInstance<Provider<Http>, Provider<Http>> =
        ContractInstance::new(H160::default(), abi.clone(), client.clone());

    let _ = c.method::<(), ()>("notARealMethod", ());

    let non_clone_mware = NonClone { m: client };

    // Works (B == &M, M: !Clone)
    let c: ContractInstance<&NonClone<Provider<Http>>, NonClone<Provider<Http>>> =
        ContractInstance::new(H160::default(), abi, &non_clone_mware);

    let _ = c.method::<(), ()>("notARealMethod", ());

    // // Fails (B == M, M: !Clone)
    // let c: ContractInternal<NonClone<Provider<Http>>, NonClone<Provider<Http>>> =
    //     ContractInternal::new(H160::default(), abi, non_clone_mware);

    // let _ = c.method::<(), ()>("notARealMethod", ());
}

#[tokio::test]
async fn deploy_and_call_contract() {
    let (abi, bytecode) = get_contract("SimpleStorage.json");

    // launch anvil
    let anvil = Anvil::new().spawn();

    // Instantiate the clients. We assume that clients consume the provider and the wallet
    // (which makes sense), so for multi-client tests, you must clone the provider.
    let addrs = anvil.addresses().to_vec();
    let addr2 = addrs[1];
    let client = connect(&anvil, 0);
    let client2 = connect(&anvil, 1);

    // create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(abi, bytecode, client.clone());

    // `send` consumes the deployer so it must be cloned for later re-use
    // (practically it's not expected that you'll need to deploy multiple instances of
    // the _same_ deployer, so it's fine to clone here from a dev UX vs perf tradeoff)
    let deployer = factory.deploy("initial value".to_string()).unwrap().legacy();
    // dry runs the deployment of the contract. takes the deployer by reference, no need to
    // clone.
    deployer.call().await.unwrap();
    let (contract, receipt) = deployer.clone().send_with_receipt().await.unwrap();
    assert_eq!(receipt.contract_address.unwrap(), contract.address());

    let get_value = contract.method::<_, String>("getValue", ()).unwrap();
    let last_sender = contract.method::<_, Address>("lastSender", ()).unwrap();

    // the initial value must be the one set in the constructor
    let value = get_value.clone().call().await.unwrap();
    assert_eq!(value, "initial value");

    // need to declare the method first, and only then send it
    // this is because it internally clones an Arc which would otherwise
    // get immediately dropped
    let contract_call =
        contract.connect(client2.clone()).method::<_, H256>("setValue", "hi".to_owned()).unwrap();
    let calldata = contract_call.calldata().unwrap();
    let _gas_estimate = contract_call.estimate_gas().await.unwrap();
    let contract_call = contract_call.legacy();
    let pending_tx = contract_call.send().await.unwrap();
    let tx = client.get_transaction(*pending_tx).await.unwrap().unwrap();
    let _tx_receipt = pending_tx.await.unwrap().unwrap();
    assert_eq!(last_sender.clone().call().await.unwrap(), addr2);
    assert_eq!(get_value.clone().call().await.unwrap(), "hi");
    assert_eq!(tx.input, calldata);

    // we can also call contract methods at other addresses with the `at` call
    // (useful when interacting with multiple ERC20s for example)
    let contract2_addr = deployer.send().await.unwrap().address();
    let contract2 = contract.at(contract2_addr);
    let init_value: String =
        contract2.method::<_, String>("getValue", ()).unwrap().call().await.unwrap();
    let init_address =
        contract2.method::<_, Address>("lastSender", ()).unwrap().call().await.unwrap();
    assert_eq!(init_address, Address::zero());
    assert_eq!(init_value, "initial value");

    // methods with multiple args also work
    let _tx_hash = contract
        .method::<_, H256>("setValues", ("hi".to_owned(), "bye".to_owned()))
        .unwrap()
        .legacy()
        .send()
        .await
        .unwrap()
        .await
        .unwrap();
}

#[tokio::test]
#[cfg(feature = "abigen")]
async fn get_past_events() {
    let (abi, bytecode) = get_contract("SimpleStorage.json");
    let anvil = Anvil::new().spawn();
    let client = connect(&anvil, 0);
    let address = client.get_accounts().await.unwrap()[0];
    let contract = deploy(client.clone(), abi, bytecode).await;

    // make a call with `client`
    let func = contract.method::<_, H256>("setValue", "hi".to_owned()).unwrap().legacy();
    let tx = func.send().await.unwrap();
    let _receipt = tx.await.unwrap();

    // and we can fetch the events
    let logs: Vec<ValueChanged> = contract
        .event()
        .from_block(0u64)
        .topic1(address) // Corresponds to the first indexed parameter
        .query()
        .await
        .unwrap();
    assert_eq!(logs[0].new_value, "initial value");
    assert_eq!(logs[1].new_value, "hi");
    assert_eq!(logs.len(), 2);

    // and we can fetch the events at a block hash
    let hash = client.get_block(1).await.unwrap().unwrap().hash.unwrap();
    let logs: Vec<ValueChanged> = contract
        .event()
        .at_block_hash(hash)
        .topic1(address) // Corresponds to the first indexed parameter
        .query()
        .await
        .unwrap();
    assert_eq!(logs[0].new_value, "initial value");
    assert_eq!(logs.len(), 1);
}

#[tokio::test]
#[cfg(feature = "abigen")]
async fn get_events_with_meta() {
    let (abi, bytecode) = get_contract("SimpleStorage.json");
    let anvil = Anvil::new().spawn();
    let client = connect(&anvil, 0);
    let address = anvil.addresses()[0];
    let contract = deploy(client.clone(), abi, bytecode).await;

    // and we can fetch the events
    let logs: Vec<(ValueChanged, LogMeta)> = contract
        .event()
        .from_block(0u64)
        .topic1(address) // Corresponds to the first indexed parameter
        .query_with_meta()
        .await
        .unwrap();

    assert_eq!(logs.len(), 1);
    let (log, meta) = &logs[0];
    assert_eq!(log.new_value, "initial value");

    assert_eq!(meta.address, contract.address());
    assert_eq!(meta.log_index, 0.into());
    assert_eq!(meta.block_number, 1.into());
    let block = client.get_block(1).await.unwrap().unwrap();
    assert_eq!(meta.block_hash, block.hash.unwrap());
    assert_eq!(block.transactions.len(), 1);
    let tx = block.transactions[0];
    assert_eq!(meta.transaction_hash, tx);
    assert_eq!(meta.transaction_index, 0.into());
}

#[tokio::test]
async fn call_past_state() {
    let (abi, bytecode) = get_contract("SimpleStorage.json");
    let anvil = Anvil::new().spawn();
    let client = connect(&anvil, 0);
    let contract = deploy(client.clone(), abi, bytecode).await;
    let deployed_block = client.get_block_number().await.unwrap();

    // assert initial state
    let value =
        contract.method::<_, String>("getValue", ()).unwrap().legacy().call().await.unwrap();
    assert_eq!(value, "initial value");

    // make a call with `client`
    let _tx_hash = *contract
        .method::<_, H256>("setValue", "hi".to_owned())
        .unwrap()
        .legacy()
        .send()
        .await
        .unwrap();

    // assert new value
    let value =
        contract.method::<_, String>("getValue", ()).unwrap().legacy().call().await.unwrap();
    assert_eq!(value, "hi");

    // assert previous value
    let value = contract
        .method::<_, String>("getValue", ())
        .unwrap()
        .legacy()
        .block(BlockId::Number(deployed_block.into()))
        .call()
        .await
        .unwrap();
    assert_eq!(value, "initial value");

    // Here would be the place to test EIP-1898, specifying the `BlockId` of `call` as the
    // first block hash. However, Ganache does not implement this :/

    // let hash = client.get_block(1).await.unwrap().unwrap().hash.unwrap();
    // let value = contract
    //     .method::<_, String>("getValue", ())
    //     .unwrap()
    //     .block(BlockId::Hash(hash))
    //     .call()
    //     .await
    //     .unwrap();
    // assert_eq!(value, "initial value");
}

#[tokio::test]
#[ignore]
async fn call_past_hash_test() {
    // geth --dev --http --http.api eth,web3
    let (abi, bytecode) = get_contract("SimpleStorage.json");
    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let deployer = provider.get_accounts().await.unwrap()[0];

    let client = Arc::new(provider.with_sender(deployer));
    let contract = deploy(client.clone(), abi, bytecode).await;
    let deployed_block = client.get_block_number().await.unwrap();

    // assert initial state
    let value = contract.method::<_, String>("getValue", ()).unwrap().call().await.unwrap();
    assert_eq!(value, "initial value");

    // make a call with `client`
    let _tx_hash =
        *contract.method::<_, H256>("setValue", "hi".to_owned()).unwrap().send().await.unwrap();

    // assert new value
    let value = contract.method::<_, String>("getValue", ()).unwrap().call().await.unwrap();
    assert_eq!(value, "hi");

    // assert previous value using block hash
    let hash = client.get_block(deployed_block).await.unwrap().unwrap().hash.unwrap();
    let value = contract
        .method::<_, String>("getValue", ())
        .unwrap()
        .block(BlockId::Hash(hash))
        .call()
        .await
        .unwrap();
    assert_eq!(value, "initial value");
}

#[tokio::test]
async fn watch_events() {
    let (abi, bytecode) = get_contract("SimpleStorage.json");
    let anvil = Anvil::new().spawn();
    let client = connect(&anvil, 0);
    let contract = deploy(client.clone(), abi.clone(), bytecode).await;

    // We spawn the event listener:
    let event = contract.event::<ValueChanged>();
    let mut stream = event.stream().await.unwrap();

    // Also set up a subscription for the same thing
    let ws = Provider::<Ws>::connect(anvil.ws_endpoint()).await.unwrap();
    let contract2 = ethers_contract::Contract::new(contract.address(), abi, ws.into());
    let event2 = contract2.event::<ValueChanged>();
    let mut subscription = event2.subscribe().await.unwrap();

    let mut subscription_meta = event2.subscribe().await.unwrap().with_meta();

    let num_calls = 3u64;

    // and we make a few calls
    let num = client.get_block_number().await.unwrap();
    for i in 0..num_calls {
        let call = contract.method::<_, H256>("setValue", i.to_string()).unwrap().legacy();
        let pending_tx = call.send().await.unwrap();
        let _receipt = pending_tx.await.unwrap();
    }

    for i in 0..num_calls {
        // unwrap the option of the stream, then unwrap the decoding result
        let log = stream.next().await.unwrap().unwrap();
        let log2 = subscription.next().await.unwrap().unwrap();
        let (log3, meta) = subscription_meta.next().await.unwrap().unwrap();
        assert_eq!(log.new_value, log3.new_value);
        assert_eq!(log.new_value, log2.new_value);
        assert_eq!(log.new_value, i.to_string());
        assert_eq!(meta.block_number, num + i + 1);
        let hash = client.get_block(num + i + 1).await.unwrap().unwrap().hash.unwrap();
        assert_eq!(meta.block_hash, hash);
    }
}

#[tokio::test]
async fn watch_subscription_events_multiple_addresses() {
    let (abi, bytecode) = get_contract("SimpleStorage.json");
    let anvil = Anvil::new().spawn();
    let client = connect(&anvil, 0);
    let contract_1 = deploy(client.clone(), abi.clone(), bytecode.clone()).await;
    let contract_2 = deploy(client.clone(), abi.clone(), bytecode).await;

    let ws = Provider::<Ws>::connect(anvil.ws_endpoint()).await.unwrap();
    let filter = Filter::new()
        .address(ValueOrArray::Array(vec![contract_1.address(), contract_2.address()]));
    let mut stream = ws.subscribe_logs(&filter).await.unwrap();

    // and we make a few calls
    let call = contract_1.method::<_, H256>("setValue", "1".to_string()).unwrap().legacy();
    let pending_tx = call.send().await.unwrap();
    let _receipt = pending_tx.await.unwrap();

    let call = contract_2.method::<_, H256>("setValue", "2".to_string()).unwrap().legacy();
    let pending_tx = call.send().await.unwrap();
    let _receipt = pending_tx.await.unwrap();

    // unwrap the option of the stream, then unwrap the decoding result
    let log_1 = stream.next().await.unwrap();
    let log_2 = stream.next().await.unwrap();
    assert_eq!(log_1.address, contract_1.address());
    assert_eq!(log_2.address, contract_2.address());
}

#[tokio::test]
async fn build_event_of_type() {
    abigen!(
        AggregatorInterface,
        r#"[
                event AnswerUpdated(int256 indexed current, uint256 indexed roundId, uint256 updatedAt)
            ]"#,
    );

    let anvil = Anvil::new().spawn();
    let client = connect(&anvil, 0);
    let event = ethers_contract::Contract::event_of_type::<AnswerUpdatedFilter>(client);
    assert_eq!(event.filter, Filter::new().event(&AnswerUpdatedFilter::abi_signature()));
}

#[tokio::test]
async fn signer_on_node() {
    let (abi, bytecode) = get_contract("SimpleStorage.json");
    // spawn anvil
    let anvil = Anvil::new().spawn();

    // connect
    let provider = Provider::<Http>::try_from(anvil.endpoint())
        .unwrap()
        .interval(std::time::Duration::from_millis(50u64));

    // get the first account
    let deployer = provider.get_accounts().await.unwrap()[0];
    let client = Arc::new(provider.with_sender(deployer));

    let contract = deploy(client, abi, bytecode).await;

    // make a call without the signer
    let _receipt = contract
        .method::<_, H256>("setValue", "hi".to_owned())
        .unwrap()
        .legacy()
        .send()
        .await
        .unwrap()
        .await
        .unwrap();
    let value: String = contract.method::<_, String>("getValue", ()).unwrap().call().await.unwrap();
    assert_eq!(value, "hi");
}

#[tokio::test]
async fn multicall_aggregate() {
    // get ABI and bytecode for the Multicall contract
    let (multicall_abi, multicall_bytecode) = get_contract("Multicall.json");

    // get ABI and bytecode for the NotSoSimpleStorage contract
    let (not_so_simple_abi, not_so_simple_bytecode) = get_contract("NotSoSimpleStorage.json");

    // get ABI and bytecode for the SimpleStorage contract
    let (abi, bytecode) = get_contract("SimpleStorage.json");

    // launch anvil
    let anvil = Anvil::new().spawn();

    // Instantiate the clients. We assume that clients consume the provider and the wallet
    // (which makes sense), so for multi-client tests, you must clone the provider.
    // `client` is used to deploy the Multicall contract
    // `client2` is used to deploy the first SimpleStorage contract
    // `client3` is used to deploy the second SimpleStorage contract
    // `client4` is used to make the aggregate call
    let addrs = anvil.addresses().to_vec();
    let addr2 = addrs[1];
    let addr3 = addrs[2];
    let client = connect(&anvil, 0);
    let client2 = connect(&anvil, 1);
    let client3 = connect(&anvil, 2);
    let client4 = connect(&anvil, 3);

    // create a factory which will be used to deploy instances of the contract
    let multicall_factory = ContractFactory::new(multicall_abi, multicall_bytecode, client.clone());
    let simple_factory = ContractFactory::new(abi.clone(), bytecode.clone(), client2.clone());
    let not_so_simple_factory =
        ContractFactory::new(not_so_simple_abi, not_so_simple_bytecode, client3.clone());

    let multicall_contract = multicall_factory.deploy(()).unwrap().legacy().send().await.unwrap();
    let addr = multicall_contract.address();

    let simple_contract =
        simple_factory.deploy("the first one".to_string()).unwrap().legacy().send().await.unwrap();
    let not_so_simple_contract = not_so_simple_factory
        .deploy("the second one".to_string())
        .unwrap()
        .legacy()
        .send()
        .await
        .unwrap();

    // Client2 and Client3 broadcast txs to set the values for both contracts
    simple_contract
        .connect(client2.clone())
        .method::<_, H256>("setValue", "reset first".to_owned())
        .unwrap()
        .legacy()
        .send()
        .await
        .unwrap();

    not_so_simple_contract
        .connect(client3.clone())
        .method::<_, H256>("setValue", "reset second".to_owned())
        .unwrap()
        .legacy()
        .send()
        .await
        .unwrap();

    // get the calls for `value` and `last_sender` for both SimpleStorage contracts
    let value = simple_contract.method::<_, String>("getValue", ()).unwrap();
    let value2 = not_so_simple_contract.method::<_, (String, Address)>("getValues", ()).unwrap();
    let last_sender = simple_contract.method::<_, Address>("lastSender", ()).unwrap();
    let last_sender2 = not_so_simple_contract.method::<_, Address>("lastSender", ()).unwrap();

    // initiate the Multicall instance and add calls one by one in builder style
    let mut multicall = Multicall::new(client4.clone(), Some(addr)).await.unwrap();

    // Set version to 1
    multicall = multicall.version(MulticallVersion::Multicall);

    multicall
        .add_call(value, false)
        .add_call(value2, false)
        .add_call(last_sender, false)
        .add_call(last_sender2, false);

    let return_data: (String, (String, Address), Address, Address) =
        multicall.call().await.unwrap();

    assert_eq!(return_data.0, "reset first");
    assert_eq!((return_data.1).0, "reset second");
    assert_eq!((return_data.1).1, addr3);
    assert_eq!(return_data.2, addr2);
    assert_eq!(return_data.3, addr3);

    // construct broadcast transactions that will be batched and broadcast via Multicall
    let broadcast = simple_contract
        .connect(client4.clone())
        .method::<_, H256>("setValue", "first reset again".to_owned())
        .unwrap();
    let broadcast2 = not_so_simple_contract
        .connect(client4.clone())
        .method::<_, H256>("setValue", "second reset again".to_owned())
        .unwrap();

    // use the already initialised Multicall instance, clearing the previous calls and adding
    // new calls. Previously we used the `.call()` functionality to do a batch of calls in one
    // go. Now we will use the `.send()` functionality to broadcast a batch of transactions
    // in one go
    let mut multicall_send = multicall.clone();
    multicall_send.clear_calls().add_call(broadcast, false).add_call(broadcast2, false);

    // broadcast the transaction and wait for it to be mined
    let _tx_receipt = multicall_send.legacy().send().await.unwrap().await.unwrap();

    // Do another multicall to check the updated return values
    // The `getValue` calls should return the last value we set in the batched broadcast
    // The `lastSender` calls should return the address of the Multicall contract, as it is
    // the one acting as proxy and calling our SimpleStorage contracts (msg.sender)
    let return_data: (String, (String, Address), Address, Address) =
        multicall.call().await.unwrap();

    assert_eq!(return_data.0, "first reset again");
    assert_eq!((return_data.1).0, "second reset again");
    assert_eq!((return_data.1).1, multicall_contract.address());
    assert_eq!(return_data.2, multicall_contract.address());
    assert_eq!(return_data.3, multicall_contract.address());

    let addrs = anvil.addresses();
    // query ETH balances of multiple addresses
    // these keys haven't been used to do any tx
    // so should have 100 ETH
    multicall
        .clear_calls()
        .add_get_eth_balance(addrs[4], false)
        .add_get_eth_balance(addrs[5], false)
        .add_get_eth_balance(addrs[6], false);

    let valid_balances = [
        U256::from(10_000_000_000_000_000_000_000u128),
        U256::from(10_000_000_000_000_000_000_000u128),
        U256::from(10_000_000_000_000_000_000_000u128),
    ];

    let balances: (U256, U256, U256) = multicall.call().await.unwrap();
    assert_eq!(balances.0, valid_balances[0]);
    assert_eq!(balances.1, valid_balances[1]);
    assert_eq!(balances.2, valid_balances[2]);

    // call_array
    multicall
        .clear_calls()
        .add_get_eth_balance(addrs[4], false)
        .add_get_eth_balance(addrs[5], false)
        .add_get_eth_balance(addrs[6], false);

    let balances: Vec<U256> = multicall.call_array().await.unwrap();
    assert_eq!(balances, Vec::from_iter(valid_balances.iter().copied()));

    // clear multicall so we can test `call_raw` w/ >16 calls
    multicall.clear_calls();

    // clear the current value
    simple_contract
        .connect(client2.clone())
        .method::<_, H256>("setValue", "many".to_owned())
        .unwrap()
        .legacy()
        .send()
        .await
        .unwrap();

    multicall.add_calls(
        false,
        std::iter::repeat(simple_contract.method::<_, String>("getValue", ()).unwrap()).take(17),
    );

    let tokens = multicall.call_raw().await.unwrap();
    let results: Vec<String> = tokens
        .into_iter()
        .map(|result| {
            // decode manually using Tokenizable method
            String::from_token(result.unwrap()).unwrap()
        })
        .collect();
    assert_eq!(results, ["many"; 17]);

    // test version 2
    multicall = multicall.version(MulticallVersion::Multicall2);

    // deploy contract with reverting methods
    let reverting_contract = {
        let (abi, bytecode) = get_contract("SimpleRevertingStorage.json");
        let f = ContractFactory::new(abi, bytecode, client.clone());
        f.deploy("This contract can revert".to_string()).unwrap().send().await.unwrap()
    };

    // reset value
    reverting_contract
        .connect(client2.clone())
        .method::<_, H256>("setValue", ("reset third".to_owned(), false))
        .unwrap()
        .send()
        .await
        .unwrap();

    // create calls
    let set_value_call = reverting_contract
        .connect(client.clone())
        .method::<_, H256>("setValue", ("this didn't revert".to_owned(), false))
        .unwrap();
    let set_value_reverting_call = reverting_contract
        .connect(client3.clone())
        .method::<_, H256>("setValue", ("this reverted".to_owned(), true))
        .unwrap();
    let get_value_call =
        reverting_contract.connect(client2.clone()).method::<_, String>("getValue", false).unwrap();
    let get_value_reverting_call =
        reverting_contract.connect(client.clone()).method::<_, String>("getValue", true).unwrap();

    // .send reverts
    // don't allow revert
    multicall
        .clear_calls()
        .add_call(set_value_reverting_call.clone(), false)
        .add_call(set_value_call.clone(), false);
    multicall.send().await.unwrap_err();

    // value has not changed
    assert_eq!(get_value_call.clone().call().await.unwrap(), "reset third");

    // allow revert
    multicall
        .clear_calls()
        .add_call(set_value_reverting_call.clone(), true)
        .add_call(set_value_call.clone(), false);
    multicall.send().await.unwrap();

    // value has changed
    assert_eq!(get_value_call.clone().call().await.unwrap(), "this didn't revert");

    // reset value again
    reverting_contract
        .connect(client2.clone())
        .method::<_, H256>("setValue", ("reset third again".to_owned(), false))
        .unwrap()
        .send()
        .await
        .unwrap();

    // .call reverts
    // don't allow revert
    multicall
        .clear_calls()
        .add_call(get_value_reverting_call.clone(), false)
        .add_call(get_value_call.clone(), false);
    let res = multicall.call::<(String, String)>().await;
    let err = res.unwrap_err();

    assert!(err.is_revert());
    let message = err.decode_revert::<String>().unwrap();
    assert!(message.contains("Multicall3: call failed"));

    // allow revert -> call doesn't revert, but returns Err(_) in raw tokens
    let expected = Bytes::from_static(b"getValue revert").encode();
    multicall.clear_calls().add_call(get_value_reverting_call.clone(), true);
    assert_eq!(multicall.call_raw().await.unwrap()[0].as_ref().unwrap_err()[4..], expected[..]);
    assert_eq!(
        multicall.call::<(String,)>().await.unwrap_err().as_revert().unwrap()[4..],
        expected[..]
    );

    // v2 illegal revert
    multicall
        .clear_calls()
        .add_call(get_value_reverting_call.clone(), false) // don't allow revert
        .add_call(get_value_call.clone(), true); // true here will result in `tryAggregate(false, ...)`
    assert!(matches!(
        multicall.call::<(String, String)>().await.unwrap_err(),
        MulticallError::IllegalRevert
    ));

    // test version 3
    // aggregate3 is the same as try_aggregate except with allowing failure on a per-call basis.
    // no need to test that
    multicall = multicall.version(MulticallVersion::Multicall3);

    // .send with value
    let amount = U256::from(100);
    let value_tx = reverting_contract.method::<_, H256>("deposit", ()).unwrap().value(amount);
    let rc_addr = reverting_contract.address();

    let (bal_before,): (U256,) =
        multicall.clear_calls().add_get_eth_balance(rc_addr, false).call().await.unwrap();

    // send 2 value_tx
    multicall.clear_calls().add_call(value_tx.clone(), false).add_call(value_tx.clone(), false);
    multicall.send().await.unwrap();

    let (bal_after,): (U256,) =
        multicall.clear_calls().add_get_eth_balance(rc_addr, false).call().await.unwrap();

    assert_eq!(bal_after, bal_before + U256::from(2) * amount);

    // test specific revert cases
    // empty revert
    let empty_revert = reverting_contract.method::<_, H256>("emptyRevert", ()).unwrap();
    multicall.clear_calls().add_call(empty_revert.clone(), true);
    assert!(multicall.call::<(String,)>().await.unwrap_err().as_revert().unwrap().is_empty());

    // string revert
    let string_revert =
        reverting_contract.method::<_, H256>("stringRevert", "String".to_string()).unwrap();
    multicall.clear_calls().add_call(string_revert, true);
    assert_eq!(
        multicall.call::<(String,)>().await.unwrap_err().as_revert().unwrap()[4..],
        Bytes::from_static(b"String").encode()[..]
    );

    // custom error revert
    let custom_error = reverting_contract.method::<_, H256>("customError", ()).unwrap();
    multicall.clear_calls().add_call(custom_error, true);
    assert_eq!(
        multicall.call::<(Bytes,)>().await.unwrap_err().as_revert().unwrap()[..],
        keccak256("CustomError()")[..4]
    );

    // custom error with data revert
    let custom_error_with_data =
        reverting_contract.method::<_, H256>("customErrorWithData", "Data".to_string()).unwrap();
    multicall.clear_calls().add_call(custom_error_with_data, true);
    let err = multicall.call::<(Bytes,)>().await.unwrap_err();
    let bytes = err.as_revert().unwrap();
    assert_eq!(bytes[..4], keccak256("CustomErrorWithData(string)")[..4]);
    assert_eq!(bytes[4..], encode(&[Token::String("Data".to_string())]));
}

#[tokio::test]
async fn test_multicall_state_overrides() {
    // get ABI and bytecode for the Multicall contract
    let (multicall_abi, multicall_bytecode) = get_contract("Multicall.json");

    // get ABI and bytecode for the NotSoSimpleStorage contract
    let (slot_storage_abi, slot_storage_bytecode) = get_contract("SlotStorage.json");

    // launch anvil
    let anvil = Anvil::new().spawn();

    let client = connect(&anvil, 0);
    let client2 = connect(&anvil, 1);

    // create a factory which will be used to deploy instances of the contract
    let multicall_factory = ContractFactory::new(multicall_abi, multicall_bytecode, client.clone());
    let slot_storage_factory =
        ContractFactory::new(slot_storage_abi, slot_storage_bytecode, client2.clone());

    let multicall_contract = multicall_factory.deploy(()).unwrap().legacy().send().await.unwrap();
    let multicall_addr = multicall_contract.address();

    let value: H256 =
        "0x312c22f60e0b666af7fce7332bfbe2a3247e19b8d612289c16b8f2e37516de36".parse().unwrap();
    let addr = "0x851a842060FC8ae05848d08872653E30FD4c9829".parse().unwrap();
    let slot: H256 =
        "0xa35a6bd95953594c6d23a75dc715af91915e970ba4d87f1141e13b915e0201a3".parse().unwrap();

    let slot_storage_contract =
        slot_storage_factory.deploy(value).unwrap().legacy().send().await.unwrap();

    // initiate the Multicall instance and add calls one by one in builder style
    let mut multicall =
        Multicall::<Provider<Http>>::new(client.clone(), Some(multicall_addr)).await.unwrap();

    // test balance override
    multicall = multicall.version(MulticallVersion::Multicall3);

    let balance = 100.into();
    let mut state = spoof::state();
    state.account(addr).balance(balance);

    multicall = multicall.state(state);
    let (get_balance,): (U256,) =
        multicall.clear_calls().add_get_eth_balance(addr, true).call().await.unwrap();
    assert_eq!(get_balance, balance);

    // test code override
    let deployed_bytecode = client.get_code(slot_storage_contract.address(), None).await.unwrap();
    state = spoof::state();
    state.account(addr).code(deployed_bytecode);

    multicall = multicall.state(state);
    let new_value: H256 =
        "0x5d2c59f6581053209078988fe8cad8edb594bad62e570e99ad4f5ea38049677b".parse().unwrap();
    let (get_old_value, get_value): (H256, H256) = multicall
        .clear_calls()
        .add_call(
            slot_storage_contract.at(addr).method::<_, H256>("setValue", new_value).unwrap(),
            false,
        )
        .add_call(slot_storage_contract.at(addr).method::<_, H256>("getValue", ()).unwrap(), false)
        .call()
        .await
        .unwrap();
    assert_eq!(get_old_value, H256::default());
    assert_eq!(get_value, new_value);

    // test slot override
    let deployed_bytecode = client.get_code(slot_storage_contract.address(), None).await.unwrap();
    let old_value =
        "0xfce2394e4cb6779bdacc1983fb24636007e9c843211586811e46b52c86d97c34".parse().unwrap();
    state = spoof::state();
    state.account(addr).code(deployed_bytecode).store(slot, old_value);

    multicall = multicall.state(state);
    let new_value: H256 =
        "0x5d2c59f6581053209078988fe8cad8edb594bad62e570e99ad4f5ea38049677b".parse().unwrap();
    let (get_old_value, get_value): (H256, H256) = multicall
        .clear_calls()
        .add_call(
            slot_storage_contract.at(addr).method::<_, H256>("setValue", new_value).unwrap(),
            false,
        )
        .add_call(slot_storage_contract.at(addr).method::<_, H256>("getValue", ()).unwrap(), false)
        .call()
        .await
        .unwrap();
    assert_eq!(get_old_value, old_value);
    assert_eq!(get_value, new_value);
}
