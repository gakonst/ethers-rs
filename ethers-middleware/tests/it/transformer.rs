use crate::{get_wallet, spawn_anvil};
use ethers_contract::abigen;
use ethers_core::{abi::AbiEncode, types::*};
use ethers_middleware::{
    transformer::{ds_proxy::factory::DsProxyFactory, DsProxy, TransformerMiddleware},
    MiddlewareBuilder, SignerMiddleware,
};
use ethers_providers::{Http, Middleware, Provider};
use ethers_signers::{LocalWallet, Signer};
use rand::Rng;
use std::sync::Arc;

type HttpWallet = SignerMiddleware<Provider<Http>, LocalWallet>;

abigen!(SimpleStorage, "../tests/testdata/SimpleStorage.json");

#[tokio::test]
#[ignore]
async fn ds_proxy_transformer() {
    // randomness
    let mut rng = rand::thread_rng();

    // spawn anvil and instantiate a signer middleware.
    let (provider, anvil) = spawn_anvil();
    let wallet = get_wallet(&anvil, 0);
    let address = wallet.address();
    let provider = Arc::new(provider.with_signer(wallet));

    // deploy DsProxyFactory which we'll use to deploy a new DsProxy contract.
    let deploy_tx = DsProxyFactory::deploy(provider.clone(), ()).unwrap();
    let ds_proxy_factory = deploy_tx.send().await.unwrap();

    // deploy a new DsProxy contract.
    let ds_proxy = DsProxy::build::<HttpWallet, Arc<HttpWallet>>(
        provider.clone(),
        Some(ds_proxy_factory.address()),
        provider.address(),
    )
    .await
    .unwrap();
    let ds_proxy_addr = ds_proxy.address();

    // deploy SimpleStorage and try to update its value via transformer middleware.
    let deploy_tx = SimpleStorage::deploy(provider.clone(), ()).unwrap();
    let simple_storage = deploy_tx.send().await.unwrap();

    // instantiate a new transformer middleware.
    let provider = TransformerMiddleware::new(provider, ds_proxy);

    // broadcast the setValue tx via transformer middleware (first wallet).
    let expected_value: u64 = rng.gen();
    let _receipt = simple_storage
        .set_value(expected_value.into())
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    // verify that DsProxy's state was updated.
    let last_sender = provider.get_storage_at(ds_proxy_addr, H256::zero(), None).await.unwrap();
    let last_value =
        provider.get_storage_at(ds_proxy_addr, H256::from_low_u64_be(1u64), None).await.unwrap();
    assert_eq!(last_sender, address.into());
    assert_eq!(last_value, H256::from_low_u64_be(expected_value));
}

#[tokio::test]
async fn ds_proxy_code() {
    // randomness
    let mut rng = rand::thread_rng();

    // spawn anvil and instantiate a signer middleware.
    let (provider, anvil) = spawn_anvil();
    let wallet = get_wallet(&anvil, 0);
    let address = wallet.address();
    let provider = Arc::new(provider.with_signer(wallet));

    // deploy DsProxyFactory which we'll use to deploy a new DsProxy contract.
    let deploy_tx = DsProxyFactory::deploy(provider.clone(), ()).unwrap();
    let ds_proxy_factory = deploy_tx.send().await.unwrap();

    // deploy a new DsProxy contract.
    let ds_proxy = DsProxy::build::<HttpWallet, Arc<HttpWallet>>(
        provider.clone(),
        Some(ds_proxy_factory.address()),
        provider.address(),
    )
    .await
    .unwrap();
    let ds_proxy_addr = ds_proxy.address();

    // encode the calldata
    let expected_value: u64 = rng.gen();
    let calldata = SetValueCall { value: expected_value.into() }.encode();

    // execute code via the deployed DsProxy contract.
    ds_proxy
        .execute::<HttpWallet, Arc<HttpWallet>, Bytes>(
            Arc::clone(&provider),
            SIMPLESTORAGE_BYTECODE.clone(),
            calldata.into(),
        )
        .expect("could not construct DSProxy contract call")
        .legacy()
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    // verify that DsProxy's state was updated.
    let last_sender = provider.get_storage_at(ds_proxy_addr, H256::zero(), None).await.unwrap();
    let last_value =
        provider.get_storage_at(ds_proxy_addr, H256::from_low_u64_be(1u64), None).await.unwrap();
    assert_eq!(last_sender, address.into());
    assert_eq!(last_value, H256::from_low_u64_be(expected_value));
}
