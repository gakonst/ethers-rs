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
    use serial_test::serial;
    use std::convert::TryFrom;

    #[tokio::test]
    #[serial]
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

    #[tokio::test]
    #[serial]
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

    #[tokio::test]
    #[serial]
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

    #[tokio::test]
    #[serial]
    async fn signer_on_node() {
        let (abi, bytecode) = compile();
        let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
        let deployer = "3cDB3d9e1B74692Bb1E3bb5fc81938151cA64b02"
            .parse::<Address>()
            .unwrap();
        let client = Client::from(provider).with_sender(deployer);
        let (_ganache, contract) = deploy(&client, abi, bytecode).await;

        // make a call without the signer
        let _tx = contract
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
}

#[cfg(feature = "celo")]
mod celo_tests {
    use super::*;
    use ethers::{
        providers::{Http, Provider},
        signers::Wallet,
        types::BlockNumber,
    };
    use std::convert::TryFrom;

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
            .connect(provider);

        let factory = ContractFactory::new(abi, bytecode, &client);
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
        let pending_tx = contract
            .method::<_, H256>("setValue", "hi".to_owned())
            .unwrap()
            .send()
            .await
            .unwrap();
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
