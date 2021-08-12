use ethers::{
    contract::ContractFactory,
    types::{Filter, ValueOrArray, H256},
};

mod common;
pub use common::*;

#[cfg(not(feature = "celo"))]
mod eth_tests {
    use super::*;
    use ethers::{
        contract::{LogMeta, Multicall},
        providers::{Http, Middleware, PendingTransaction, Provider, StreamExt},
        types::{Address, BlockId, U256},
        utils::Ganache,
    };
    use std::{convert::TryFrom, sync::Arc};

    #[tokio::test]
    async fn deploy_and_call_contract() {
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");

        // launch ganache
        let ganache = Ganache::new().spawn();

        // Instantiate the clients. We assume that clients consume the provider and the wallet
        // (which makes sense), so for multi-client tests, you must clone the provider.
        let client = connect(&ganache, 0);
        let client2 = connect(&ganache, 1);

        // create a factory which will be used to deploy instances of the contract
        let factory = ContractFactory::new(abi, bytecode, client.clone());

        // `send` consumes the deployer so it must be cloned for later re-use
        // (practically it's not expected that you'll need to deploy multiple instances of
        // the _same_ deployer, so it's fine to clone here from a dev UX vs perf tradeoff)
        let deployer = factory
            .deploy("initial value".to_string())
            .unwrap()
            .legacy();
        let contract = deployer.clone().send().await.unwrap();

        let get_value = contract.method::<_, String>("getValue", ()).unwrap();
        let last_sender = contract.method::<_, Address>("lastSender", ()).unwrap();

        // the initial value must be the one set in the constructor
        let value = get_value.clone().call().await.unwrap();
        assert_eq!(value, "initial value");

        // need to declare the method first, and only then send it
        // this is because it internally clones an Arc which would otherwise
        // get immediately dropped
        let contract_call = contract
            .connect(client2.clone())
            .method::<_, H256>("setValue", "hi".to_owned())
            .unwrap();
        let calldata = contract_call.calldata().unwrap();
        let gas_estimate = contract_call.estimate_gas().await.unwrap();
        let contract_call = contract_call.legacy();
        let pending_tx = contract_call.send().await.unwrap();
        let tx = client.get_transaction(*pending_tx).await.unwrap().unwrap();
        let tx_receipt = pending_tx.await.unwrap().unwrap();
        assert_eq!(last_sender.clone().call().await.unwrap(), client2.address());
        assert_eq!(get_value.clone().call().await.unwrap(), "hi");
        assert_eq!(tx.input, calldata);
        assert_eq!(tx_receipt.gas_used.unwrap(), gas_estimate);

        // we can also call contract methods at other addresses with the `at` call
        // (useful when interacting with multiple ERC20s for example)
        let contract2_addr = deployer.send().await.unwrap().address();
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
    async fn get_past_events() {
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");
        let ganache = Ganache::new().spawn();
        let client = connect(&ganache, 0);
        let contract = deploy(client.clone(), abi, bytecode).await;

        // make a call with `client`
        let func = contract
            .method::<_, H256>("setValue", "hi".to_owned())
            .unwrap()
            .legacy();
        let tx = func.send().await.unwrap();
        let _receipt = tx.await.unwrap();

        // and we can fetch the events
        let logs: Vec<ValueChanged> = contract
            .event()
            .from_block(0u64)
            .topic1(client.address()) // Corresponds to the first indexed parameter
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
            .topic1(client.address()) // Corresponds to the first indexed parameter
            .query()
            .await
            .unwrap();
        assert_eq!(logs[0].new_value, "initial value");
        assert_eq!(logs.len(), 1);
    }

    #[tokio::test]
    async fn get_events_with_meta() {
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");
        let ganache = Ganache::new().spawn();
        let client = connect(&ganache, 0);
        let contract = deploy(client.clone(), abi, bytecode).await;

        // and we can fetch the events
        let logs: Vec<(ValueChanged, LogMeta)> = contract
            .event()
            .from_block(0u64)
            .topic1(client.address()) // Corresponds to the first indexed parameter
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
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");
        let ganache = Ganache::new().spawn();
        let client = connect(&ganache, 0);
        let contract = deploy(client.clone(), abi, bytecode).await;
        let deployed_block = client.get_block_number().await.unwrap();

        // assert initial state
        let value = contract
            .method::<_, String>("getValue", ())
            .unwrap()
            .legacy()
            .call()
            .await
            .unwrap();
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
        let value = contract
            .method::<_, String>("getValue", ())
            .unwrap()
            .legacy()
            .call()
            .await
            .unwrap();
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
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");
        let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
        let deployer = provider.get_accounts().await.unwrap()[0];

        let client = Arc::new(provider.with_sender(deployer));
        let contract = deploy(client.clone(), abi, bytecode).await;
        let deployed_block = client.get_block_number().await.unwrap();

        // assert initial state
        let value = contract
            .method::<_, String>("getValue", ())
            .unwrap()
            .call()
            .await
            .unwrap();
        assert_eq!(value, "initial value");

        // make a call with `client`
        let _tx_hash = *contract
            .method::<_, H256>("setValue", "hi".to_owned())
            .unwrap()
            .send()
            .await
            .unwrap();

        // assert new value
        let value = contract
            .method::<_, String>("getValue", ())
            .unwrap()
            .call()
            .await
            .unwrap();
        assert_eq!(value, "hi");

        // assert previous value using block hash
        let hash = client
            .get_block(deployed_block)
            .await
            .unwrap()
            .unwrap()
            .hash
            .unwrap();
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
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");
        let ganache = Ganache::new().spawn();
        let client = connect(&ganache, 0);
        let contract = deploy(client.clone(), abi.clone(), bytecode).await;

        // We spawn the event listener:
        let event = contract.event::<ValueChanged>();
        let mut stream = event.stream().await.unwrap();
        assert_eq!(stream.id, 1.into());

        // Also set up a subscription for the same thing
        let ws = Provider::connect(ganache.ws_endpoint()).await.unwrap();
        let contract2 = ethers_contract::Contract::new(contract.address(), abi, ws);
        let event2 = contract2.event::<ValueChanged>();
        let mut subscription = event2.subscribe().await.unwrap();
        assert_eq!(subscription.id, 2.into());

        let mut subscription_meta = event2.subscribe().await.unwrap().with_meta();
        assert_eq!(subscription_meta.0.id, 3.into());

        let num_calls = 3u64;

        // and we make a few calls
        let num = client.get_block_number().await.unwrap();
        for i in 0..num_calls {
            let call = contract
                .method::<_, H256>("setValue", i.to_string())
                .unwrap()
                .legacy();
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
            let hash = client
                .get_block(num + i + 1)
                .await
                .unwrap()
                .unwrap()
                .hash
                .unwrap();
            assert_eq!(meta.block_hash, hash);
        }
    }

    #[tokio::test]
    async fn watch_subscription_events_multiple_addresses() {
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");
        let ganache = Ganache::new().spawn();
        let client = connect(&ganache, 0);
        let contract_1 = deploy(client.clone(), abi.clone(), bytecode.clone()).await;
        let contract_2 = deploy(client.clone(), abi.clone(), bytecode).await;

        let ws = Provider::connect(ganache.ws_endpoint()).await.unwrap();
        let filter = Filter::new().address(ValueOrArray::Array(vec![
            contract_1.address(),
            contract_2.address(),
        ]));
        let mut stream = ws.subscribe_logs(&filter).await.unwrap();

        // and we make a few calls
        let call = contract_1
            .method::<_, H256>("setValue", "1".to_string())
            .unwrap()
            .legacy();
        let pending_tx = call.send().await.unwrap();
        let _receipt = pending_tx.await.unwrap();

        let call = contract_2
            .method::<_, H256>("setValue", "2".to_string())
            .unwrap()
            .legacy();
        let pending_tx = call.send().await.unwrap();
        let _receipt = pending_tx.await.unwrap();

        // unwrap the option of the stream, then unwrap the decoding result
        let log_1 = stream.next().await.unwrap();
        let log_2 = stream.next().await.unwrap();
        assert_eq!(log_1.address, contract_1.address());
        assert_eq!(log_2.address, contract_2.address());
    }

    #[tokio::test]
    async fn signer_on_node() {
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");
        // spawn ganache
        let ganache = Ganache::new().spawn();

        // connect
        let provider = Provider::<Http>::try_from(ganache.endpoint())
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
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
        let value: String = contract
            .method::<_, String>("getValue", ())
            .unwrap()
            .call()
            .await
            .unwrap();
        assert_eq!(value, "hi");
    }

    #[tokio::test]
    async fn multicall_aggregate() {
        // get ABI and bytecode for the Multcall contract
        let (multicall_abi, multicall_bytecode) = compile_contract("Multicall", "Multicall.sol");

        // get ABI and bytecode for the NotSoSimpleStorage contract
        let (not_so_simple_abi, not_so_simple_bytecode) =
            compile_contract("NotSoSimpleStorage", "NotSoSimpleStorage.sol");

        // get ABI and bytecode for the SimpleStorage contract
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");

        // launch ganache
        let ganache = Ganache::new().spawn();

        // Instantiate the clients. We assume that clients consume the provider and the wallet
        // (which makes sense), so for multi-client tests, you must clone the provider.
        // `client` is used to deploy the Multicall contract
        // `client2` is used to deploy the first SimpleStorage contract
        // `client3` is used to deploy the second SimpleStorage contract
        // `client4` is used to make the aggregate call
        let client = connect(&ganache, 0);
        let client2 = connect(&ganache, 1);
        let client3 = connect(&ganache, 2);
        let client4 = connect(&ganache, 3);

        // create a factory which will be used to deploy instances of the contract
        let multicall_factory =
            ContractFactory::new(multicall_abi, multicall_bytecode, client.clone());
        let simple_factory = ContractFactory::new(abi.clone(), bytecode.clone(), client2.clone());
        let not_so_simple_factory =
            ContractFactory::new(not_so_simple_abi, not_so_simple_bytecode, client3.clone());

        let multicall_contract = multicall_factory
            .deploy(())
            .unwrap()
            .legacy()
            .send()
            .await
            .unwrap();
        let addr = multicall_contract.address();

        let simple_contract = simple_factory
            .deploy("the first one".to_string())
            .unwrap()
            .legacy()
            .send()
            .await
            .unwrap();
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
        let value2 = not_so_simple_contract
            .method::<_, (String, Address)>("getValues", ())
            .unwrap();
        let last_sender = simple_contract
            .method::<_, Address>("lastSender", ())
            .unwrap();
        let last_sender2 = not_so_simple_contract
            .method::<_, Address>("lastSender", ())
            .unwrap();

        // initiate the Multicall instance and add calls one by one in builder style
        let mut multicall = Multicall::new(client4.clone(), Some(addr)).await.unwrap();

        multicall
            .add_call(value)
            .add_call(value2)
            .add_call(last_sender)
            .add_call(last_sender2);

        let return_data: (String, (String, Address), Address, Address) =
            multicall.call().await.unwrap();

        assert_eq!(return_data.0, "reset first");
        assert_eq!((return_data.1).0, "reset second");
        assert_eq!((return_data.1).1, client3.address());
        assert_eq!(return_data.2, client2.address());
        assert_eq!(return_data.3, client3.address());

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
        multicall_send
            .clear_calls()
            .add_call(broadcast)
            .add_call(broadcast2);

        // broadcast the transaction and wait for it to be mined
        let tx_hash = multicall_send.legacy().send().await.unwrap();
        let _tx_receipt = PendingTransaction::new(tx_hash, client.provider())
            .await
            .unwrap();

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

        let addrs = ganache.addresses();
        // query ETH balances of multiple addresses
        // these keys haven't been used to do any tx
        // so should have 100 ETH
        multicall
            .clear_calls()
            .eth_balance_of(addrs[4])
            .eth_balance_of(addrs[5])
            .eth_balance_of(addrs[6]);
        let balances: (U256, U256, U256) = multicall.call().await.unwrap();
        assert_eq!(balances.0, U256::from(100000000000000000000u128));
        assert_eq!(balances.1, U256::from(100000000000000000000u128));
        assert_eq!(balances.2, U256::from(100000000000000000000u128));
    }
}

#[cfg(feature = "celo")]
mod celo_tests {
    use super::*;
    use ethers::{
        middleware::signer::SignerMiddleware,
        providers::{Http, Middleware, Provider},
        signers::{LocalWallet, Signer},
        types::BlockNumber,
    };
    use std::{convert::TryFrom, sync::Arc, time::Duration};

    #[tokio::test]
    async fn deploy_and_call_contract() {
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");

        // Celo testnet
        let provider = Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org")
            .unwrap()
            .interval(Duration::from_millis(6000));
        let chain_id = provider.get_chainid().await.unwrap().as_u64();

        // Funded with https://celo.org/developers/faucet
        let wallet = "d652abb81e8c686edba621a895531b1f291289b63b5ef09a94f686a5ecdd5db1"
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(chain_id);

        let client = SignerMiddleware::new(provider, wallet);
        let client = Arc::new(client);

        let factory = ContractFactory::new(abi, bytecode, client);
        let deployer = factory
            .deploy("initial value".to_string())
            .unwrap()
            .legacy();
        let contract = deployer.block(BlockNumber::Pending).send().await.unwrap();

        let value: String = contract
            .method("getValue", ())
            .unwrap()
            .call()
            .await
            .unwrap();
        assert_eq!(value, "initial value");

        // make a state mutating transaction
        // gas estimation costs are sometimes under-reported on celo,
        // so we manually set it to avoid failures
        let call = contract
            .method::<_, H256>("setValue", "hi".to_owned())
            .unwrap()
            .gas(100000);
        let pending_tx = call.send().await.unwrap();
        let _receipt = pending_tx.await.unwrap();

        let value: String = contract
            .method("getValue", ())
            .unwrap()
            .call()
            .await
            .unwrap();
        assert_eq!(value, "hi");
    }
}
