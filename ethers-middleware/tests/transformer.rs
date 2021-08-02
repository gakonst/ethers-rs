use ethers_contract::{BaseContract, ContractFactory};
use ethers_core::{
    types::*,
    utils::{Ganache, Solc},
};
use ethers_middleware::{
    transformer::{DsProxy, TransformerMiddleware},
    SignerMiddleware,
};
use ethers_providers::{Http, Middleware, Provider};
use ethers_signers::{LocalWallet, Signer};
use rand::Rng;
use std::{convert::TryFrom, sync::Arc, time::Duration};

type HttpWallet = SignerMiddleware<Provider<Http>, LocalWallet>;

#[tokio::test]
#[cfg(not(feature = "celo"))]
async fn ds_proxy_transformer() {
    // randomness
    let mut rng = rand::thread_rng();

    // spawn ganache and instantiate a signer middleware.
    let ganache = Ganache::new().spawn();
    let wallet: LocalWallet = ganache.keys()[0].clone().into();
    let provider = Provider::<Http>::try_from(ganache.endpoint())
        .unwrap()
        .interval(Duration::from_millis(10u64));
    let chain_id = provider.get_chainid().await.unwrap().as_u64();
    let wallet = wallet.with_chain_id(chain_id);
    let signer_middleware = SignerMiddleware::new(provider.clone(), wallet);
    let wallet_addr = signer_middleware.address();
    let provider = Arc::new(signer_middleware.clone());

    // deploy DsProxyFactory which we'll use to deploy a new DsProxy contract.
    let compiled = Solc::new("./tests/solidity-contracts/DSProxy.sol")
        .build()
        .expect("could not compile DSProxyFactory");
    let contract = compiled
        .get("DSProxyFactory")
        .expect("could not find DSProxyFactory");
    let factory = ContractFactory::new(
        contract.abi.clone(),
        contract.bytecode.clone(),
        Arc::clone(&provider),
    );
    let ds_proxy_factory = factory.deploy(()).unwrap().legacy();
    let ds_proxy_factory = ds_proxy_factory.send().await.unwrap();

    // deploy a new DsProxy contract.
    let ds_proxy = DsProxy::build::<HttpWallet, Arc<HttpWallet>>(
        Arc::clone(&provider),
        Some(ds_proxy_factory.address()),
        provider.address(),
    )
    .await
    .unwrap();
    let ds_proxy_addr = ds_proxy.address();

    // deploy SimpleStorage and try to update its value via transformer middleware.
    let compiled = Solc::new("./tests/solidity-contracts/SimpleStorage.sol")
        .build()
        .expect("could not compile SimpleStorage");
    let contract = compiled
        .get("SimpleStorage")
        .expect("could not find SimpleStorage");
    let factory = ContractFactory::new(
        contract.abi.clone(),
        contract.bytecode.clone(),
        Arc::clone(&provider),
    );
    let deployer = factory.deploy(()).unwrap().legacy();
    let simple_storage = deployer.send().await.unwrap();

    // instantiate a new transformer middleware.
    let provider = TransformerMiddleware::new(signer_middleware, ds_proxy.clone());

    // broadcast the setValue tx via transformer middleware (first wallet).
    let expected_value: u64 = rng.gen();
    let calldata = simple_storage
        .encode("setValue", U256::from(expected_value))
        .expect("could not get ABI encoded data");
    let tx = TransactionRequest::new()
        .to(simple_storage.address())
        .data(calldata);
    provider
        .send_transaction(tx, None)
        .await
        .unwrap()
        .await
        .unwrap();

    // verify that DsProxy's state was updated.
    let last_sender = provider
        .get_storage_at(ds_proxy_addr, H256::zero(), None)
        .await
        .unwrap();
    let last_value = provider
        .get_storage_at(ds_proxy_addr, H256::from_low_u64_be(1u64), None)
        .await
        .unwrap();
    assert_eq!(last_sender, wallet_addr.into());
    assert_eq!(last_value, H256::from_low_u64_be(expected_value));
}

#[tokio::test]
#[cfg(not(feature = "celo"))]
async fn ds_proxy_code() {
    // randomness
    let mut rng = rand::thread_rng();

    // spawn ganache and instantiate a signer middleware.
    let ganache = Ganache::new().spawn();
    let wallet: LocalWallet = ganache.keys()[1].clone().into();
    let provider = Provider::<Http>::try_from(ganache.endpoint())
        .unwrap()
        .interval(Duration::from_millis(10u64));
    let chain_id = provider.get_chainid().await.unwrap().as_u64();
    let wallet = wallet.with_chain_id(chain_id);
    let signer_middleware = SignerMiddleware::new(provider.clone(), wallet);
    let wallet_addr = signer_middleware.address();
    let provider = Arc::new(signer_middleware.clone());

    // deploy DsProxyFactory which we'll use to deploy a new DsProxy contract.
    let compiled = Solc::new("./tests/solidity-contracts/DSProxy.sol")
        .build()
        .expect("could not compile DSProxyFactory");
    let contract = compiled
        .get("DSProxyFactory")
        .expect("could not find DSProxyFactory");
    let factory = ContractFactory::new(
        contract.abi.clone(),
        contract.bytecode.clone(),
        Arc::clone(&provider),
    );
    let ds_proxy_factory = factory.deploy(()).unwrap().legacy();
    let ds_proxy_factory = ds_proxy_factory.send().await.unwrap();

    // deploy a new DsProxy contract.
    let ds_proxy = DsProxy::build::<HttpWallet, Arc<HttpWallet>>(
        Arc::clone(&provider),
        Some(ds_proxy_factory.address()),
        provider.address(),
    )
    .await
    .unwrap();
    let ds_proxy_addr = ds_proxy.address();

    // compile the SimpleStorage contract which we will use to interact via DsProxy.
    let compiled = Solc::new("./tests/solidity-contracts/SimpleStorage.sol")
        .build()
        .expect("could not compile SimpleStorage");
    let ss = compiled
        .get("SimpleStorage")
        .expect("could not find SimpleStorage");
    let ss_base_contract: BaseContract = ss.abi.clone().into();
    let expected_value: u64 = rng.gen();
    let calldata = ss_base_contract
        .encode("setValue", U256::from(expected_value))
        .expect("could not get ABI encoded data");

    // execute code via the deployed DsProxy contract.
    ds_proxy
        .execute::<HttpWallet, Arc<HttpWallet>, Bytes>(
            Arc::clone(&provider),
            ss.bytecode.clone(),
            calldata,
        )
        .expect("could not construct DSProxy contract call")
        .legacy()
        .send()
        .await
        .unwrap();

    // verify that DsProxy's state was updated.
    let last_sender = provider
        .get_storage_at(ds_proxy_addr, H256::zero(), None)
        .await
        .unwrap();
    let last_value = provider
        .get_storage_at(ds_proxy_addr, H256::from_low_u64_be(1u64), None)
        .await
        .unwrap();
    assert_eq!(last_sender, wallet_addr.into());
    assert_eq!(last_value, H256::from_low_u64_be(expected_value));
}
