use std::sync::Arc;

use crate::spawn_anvil;
use ethers_core::{types::*, abi::AbiEncode};
use ethers_middleware::{MulticallMiddleware, SignerMiddleware, multicall::MulticallMiddlewareError};

use ethers_contract::{
    abigen,
    multicall::constants::{DEPLOYER_ADDRESS, MULTICALL_ADDRESS, SIGNED_DEPLOY_MULTICALL_TX}, ContractError, EthError,
};
use ethers_signers::{LocalWallet, Signer};
use instant::Duration;

abigen!(
    SimpleRevertingStorage,
    "../ethers-contract/tests/solidity-contracts/SimpleRevertingStorage.json"
);
abigen!(SimpleStorage, "../ethers-contract/tests/solidity-contracts/SimpleStorage.json");

#[tokio::test]
async fn multicall() {
    let (provider, anvil) = spawn_anvil();

    // 1. deploy multicall contract (if not already)
    provider
        .request::<(H160, U256), ()>(
            "anvil_setBalance",
            (DEPLOYER_ADDRESS, U256::from(1_000_000_000_000_000_000u64)),
        )
        .await
        .unwrap();
    provider
        .request::<[serde_json::Value; 1], H256>(
            "eth_sendRawTransaction",
            [SIGNED_DEPLOY_MULTICALL_TX.into()],
        )
        .await
        .unwrap();

    // 2. deploy some contracts to interact with
    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let client = Arc::new(SignerMiddleware::new(provider, wallet.with_chain_id(anvil.chain_id())));

    let value = "multicall!".to_string();
    let simple =
        SimpleStorage::deploy(client.clone(), value.clone()).unwrap().send().await.unwrap();
    let simple_reverting =
        SimpleRevertingStorage::deploy(client.clone(), value.clone()).unwrap().send().await.unwrap();

    // 3. instantiate the multicall middleware
    // TODO: get BaseContracts before deploying?
    let contracts = vec![simple.abi().clone().into(), simple_reverting.abi().to_owned().into()];
    let (multicall_provider, multicall_processor) = MulticallMiddleware::new(
        client,
        contracts,
        Duration::from_secs(1),
        Some(MULTICALL_ADDRESS),
    );

    let multicall_client = Arc::new(multicall_provider);

    // 4. reconnect contracts to the multicall provider
    let simple = SimpleStorage::new(simple.address(), multicall_client.clone());
    let simple_reverting =
        SimpleRevertingStorage::new(simple_reverting.address(), multicall_client);

    // 5. spawn the multicall processor
    tokio::spawn(async move {
        let _ = multicall_processor.run().await;
    });

    // 6. perform some calls in parallel
    tokio::spawn(async move {
        let simple_result = simple.get_value().call().await.unwrap();
        assert_eq!(simple_result, value);
    });

    let reverting_result = simple_reverting.get_value(true).call().await.unwrap_err().to_string();
    assert_eq!(
        reverting_result,
        "getValue revert"
    );
}
