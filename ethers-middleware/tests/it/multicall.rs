use std::sync::Arc;

use crate::spawn_anvil;
use ethers_core::types::*;
use ethers_middleware::{MulticallMiddleware, SignerMiddleware};

use ethers_contract::{
    abigen,
    multicall::constants::{DEPLOYER_ADDRESS, MULTICALL_ADDRESS, SIGNED_DEPLOY_MULTICALL_TX},
};
use ethers_providers::Middleware;
use ethers_signers::{LocalWallet, Signer};

abigen!(
    SimpleRevertingStorage,
    "../ethers-contract/tests/solidity-contracts/SimpleRevertingStorage.json"
);
abigen!(
    SimpleStorage,
    "../ethers-contract/tests/solidity-contracts/SimpleStorage.json"
);

#[tokio::test]
async fn multicall() {
    let (provider, anvil) = spawn_anvil();
    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet.with_chain_id(anvil.chain_id())));

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

    // 2. instantiate the multicall middleware
    let (multicall_provider, multicall_processor) = MulticallMiddleware::new(
        client,
        vec![SIMPLEREVERTINGSTORAGE_ABI.to_owned(), SIMPLESTORAGE_ABI.to_owned()],
        10,
        Some(MULTICALL_ADDRESS),
    );
    let multicall_client = Arc::new(multicall_provider);

    // 3. deploy a contract to interact with
    let value = "multicall!".to_string();
    let simple = SimpleStorage::deploy(multicall_client.clone(), value.clone()).unwrap().send().await.unwrap();
    let simple_reverting = SimpleRevertingStorage::deploy(multicall_client.clone(), value.clone()).unwrap().send().await.unwrap();

    // 4. spawn the multicall processor
    tokio::spawn(async move {
        let _ = multicall_processor.run().await;
    });

    // 5. make some calls in parallel
    tokio::join!(
        async {
            let val: String = simple.get_value().call().await.unwrap();
            assert_eq!(val, value);
        },
        async {
            let e = simple_reverting.get_value(true).call().await.unwrap_err();
            assert!(e.to_string().contains("call reverted"));
        },
        async {
            let bal = multicall_client.get_balance(DEPLOYER_ADDRESS, None).await.unwrap();
            assert!(bal > U256::zero());
        },
        async {
            let block = multicall_client.get_block_number().await.unwrap();
            assert!(block > U64::zero());
        }
    );
}
