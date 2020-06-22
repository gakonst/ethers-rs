use ethers::{contract::ContractFactory, types::H256};

mod common;
pub use common::*;

#[cfg(not(feature = "celo"))]
mod eth_tests {
    use super::*;
    use ethers::{
        providers::{Http, Provider, StreamExt},
        signers::Client,
        types::Address,
        utils::Ganache,
    };
    use std::{convert::TryFrom, sync::Arc};

    #[tokio::test]
    async fn deploy_and_call_contract() {
        let (abi, bytecode) = compile();

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
    }

    #[tokio::test]
    async fn get_past_events() {
        let (abi, bytecode) = compile();
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
        let (abi, bytecode) = compile();
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
        let (abi, bytecode) = compile();
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
        let (abi, bytecode) = compile();

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
