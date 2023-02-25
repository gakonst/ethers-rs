use crate::compile_contract;
use ethers::prelude::*;
use std::{sync::Arc, time::Duration};

#[tokio::test]
async fn test_send_transaction() {
    // Celo testnet
    let provider = Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org")
        .unwrap()
        .interval(Duration::from_millis(3000u64));
    let chain_id = provider.get_chainid().await.unwrap().as_u64();

    // Funded with https://celo.org/developers/faucet
    // Please do not drain this account :)
    let wallet = "d652abb81e8c686edba621a895531b1f291289b63b5ef09a94f686a5ecdd5db1"
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id);
    let client = SignerMiddleware::new(provider, wallet);

    let balance_before = client.get_balance(client.address(), None).await.unwrap();
    let tx = TransactionRequest::pay(client.address(), 100);
    let _receipt = client.send_transaction(tx, None).await.unwrap().confirmations(3).await.unwrap();
    let balance_after = client.get_balance(client.address(), None).await.unwrap();
    assert!(balance_before > balance_after);
}

#[tokio::test]
async fn deploy_and_call_contract() {
    // compiles the given contract and returns the ABI and Bytecode
    let (abi, bytecode) = compile_contract("SimpleStorage.sol", "SimpleStorage");

    // Celo testnet
    let provider = Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org")
        .unwrap()
        .interval(Duration::from_millis(6000));
    let chain_id = provider.get_chainid().await.unwrap().as_u64();

    // Funded with https://celo.org/developers/faucet
    let wallet = "58ea5643a78c36926ad5128a6b0d8dfcc7fc705788a993b1c724be3469bc9697"
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id);
    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await.unwrap();
    let client = Arc::new(client);

    let factory = ContractFactory::new(abi, bytecode, client);
    let deployer = factory.deploy(()).unwrap().legacy();
    let contract = deployer.block(BlockNumber::Pending).send().await.unwrap();

    let value: U256 = contract.method("value", ()).unwrap().call().await.unwrap();
    assert_eq!(value, 0.into());

    // make a state mutating transaction
    // gas estimation costs are sometimes under-reported on celo,
    // so we manually set it to avoid failures
    let call = contract.method::<_, H256>("setValue", U256::from(1)).unwrap().gas(100000);
    let pending_tx = call.send().await.unwrap();
    let _receipt = pending_tx.await.unwrap();

    let value: U256 = contract.method("value", ()).unwrap().call().await.unwrap();
    assert_eq!(value, 1.into());
}
