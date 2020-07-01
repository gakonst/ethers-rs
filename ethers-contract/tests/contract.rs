use ethers::{
    contract::{ContractFactory, Multicall},
    types::H256,
};

mod common;
pub use common::*;

#[cfg(not(feature = "celo"))]
mod eth_tests {
    use super::*;
    use ethers::{
        providers::{Http, Provider, StreamExt},
        signers::Client,
        types::{Address, U256},
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
        let deployer = factory.deploy("initial value".to_string()).unwrap();
        let contract = deployer.clone().send().await.unwrap();

        let get_value = contract.method::<_, String>("getValue", ()).unwrap();
        let last_sender = contract.method::<_, Address>("lastSender", ()).unwrap();

        // the initial value must be the one set in the constructor
        let value = get_value.clone().call().await.unwrap();
        assert_eq!(value, "initial value");

        // need to declare the method first, and only then send it
        // this is because it internally clones an Arc which would otherwise
        // get immediately dropped
        let _tx_hash = contract
            .connect(client2.clone())
            .method::<_, H256>("setValue", "hi".to_owned())
            .unwrap()
            .send()
            .await
            .unwrap();
        assert_eq!(last_sender.clone().call().await.unwrap(), client2.address());
        assert_eq!(get_value.clone().call().await.unwrap(), "hi");

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
            .send()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_past_events() {
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");
        let ganache = Ganache::new().spawn();
        let client = connect(&ganache, 0);
        let contract = deploy(client.clone(), abi, bytecode).await;

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

    #[tokio::test]
    async fn watch_events() {
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");
        let ganache = Ganache::new().spawn();
        let client = connect(&ganache, 0);
        let contract = deploy(client, abi, bytecode).await;

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
            let tx_hash = contract
                .method::<_, H256>("setValue", i.to_string())
                .unwrap()
                .send()
                .await
                .unwrap();
            let _receipt = contract.pending_transaction(tx_hash).await.unwrap();
        }

        for i in 0..num_calls {
            // unwrap the option of the stream, then unwrap the decoding result
            let log = stream.next().await.unwrap().unwrap();
            assert_eq!(log.new_value, i.to_string());
        }
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
        let client = Arc::new(Client::from(provider).with_sender(deployer));

        let contract = deploy(client, abi, bytecode).await;

        // make a call without the signer
        let tx_hash = contract
            .method::<_, H256>("setValue", "hi".to_owned())
            .unwrap()
            .send()
            .await
            .unwrap();
        let _receipt = contract.pending_transaction(tx_hash).await.unwrap();
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

        // `send` consumes the deployer so it must be cloned for later re-use
        // (practically it's not expected that you'll need to deploy multiple instances of
        // the _same_ deployer, so it's fine to clone here from a dev UX vs perf tradeoff)
        let multicall_deployer = multicall_factory.deploy(()).unwrap();
        let multicall_contract = multicall_deployer.clone().send().await.unwrap();
        let simple_deployer = simple_factory.deploy("the first one".to_string()).unwrap();
        let simple_contract = simple_deployer.clone().send().await.unwrap();
        let not_so_simple_deployer = not_so_simple_factory
            .deploy("the second one".to_string())
            .unwrap();
        let not_so_simple_contract = not_so_simple_deployer.clone().send().await.unwrap();

        // need to declare the method first, and only then send it
        // this is because it internally clones an Arc which would otherwise
        // get immediately dropped
        simple_contract
            .connect(client2.clone())
            .method::<_, H256>("setValue", "reset first".to_owned())
            .unwrap()
            .send()
            .await
            .unwrap();
        not_so_simple_contract
            .connect(client3.clone())
            .method::<_, H256>("setValue", "reset second".to_owned())
            .unwrap()
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

        // initiate the Multicall instance
        let multicall = Multicall::new(Some(multicall_contract.address()), None, client4.clone());

        // add calls to multicall
        let multicall = multicall.add_call(value);
        let multicall = multicall.add_calls(vec![value2]);
        let multicall = multicall.add_calls(vec![last_sender, last_sender2]);

        let return_data: (U256, (String, (String, Address), Address, Address)) =
            multicall.call().await.unwrap();

        let return_data = return_data.1;
        assert_eq!(return_data.0, "reset first");
        assert_eq!((return_data.1).0, "reset second");
        assert_eq!((return_data.1).1, client3.address());
        assert_eq!(return_data.2, client2.address());
        assert_eq!(return_data.3, client3.address());
    }
}

#[cfg(feature = "celo")]
mod celo_tests {
    use super::*;
    use ethers::{
        providers::{Http, Provider},
        signers::Wallet,
        types::BlockNumber,
    };
    use std::{convert::TryFrom, sync::Arc, time::Duration};

    #[tokio::test]
    async fn deploy_and_call_contract() {
        let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");

        // Celo testnet
        let provider =
            Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org").unwrap();

        // Funded with https://celo.org/developers/faucet
        let client = "d652abb81e8c686edba621a895531b1f291289b63b5ef09a94f686a5ecdd5db1"
            .parse::<Wallet>()
            .unwrap()
            .connect(provider)
            .interval(Duration::from_millis(6000));
        let client = Arc::new(client);

        let factory = ContractFactory::new(abi, bytecode, client);
        let deployer = factory.deploy("initial value".to_string()).unwrap();
        let contract = deployer.block(BlockNumber::Pending).send().await.unwrap();

        let value: String = contract
            .method("getValue", ())
            .unwrap()
            .call()
            .await
            .unwrap();
        assert_eq!(value, "initial value");

        // make a state mutating transaction
        let tx_hash = contract
            .method::<_, H256>("setValue", "hi".to_owned())
            .unwrap()
            .send()
            .await
            .unwrap();
        let _receipt = contract.pending_transaction(tx_hash).await.unwrap();

        let value: String = contract
            .method("getValue", ())
            .unwrap()
            .call()
            .await
            .unwrap();
        assert_eq!(value, "hi");
    }
}
