use ethers_contract::ContractFactory;
use ethers_contract_derive::abigen;
use ethers_core::{abi::Address, utils::Anvil};
use ethers_providers::{MiddlewareError, Provider};
use std::{
    convert::TryFrom,
    future::{Future, IntoFuture},
    sync::Arc,
};

use crate::common::compile_contract;

#[tokio::test]
async fn contract_call_into_future_is_send() {
    abigen!(DsProxyFactory, "ethers-middleware/contracts/DsProxyFactory.json");
    let (provider, _) = Provider::mocked();
    let client = Arc::new(provider);
    let contract = DsProxyFactory::new(Address::zero(), client);

    fn is_send<T: Future + Send + 'static>(future: T) -> T {
        future
    }

    is_send(contract.cache().into_future());
}

#[tokio::test]
async fn revert_data_is_captured() {
    abigen!(
        SimpleRevertingStorage,
        r#"[
        function emptyRevert(),
        function stringRevert(string),
        function customError()
        ]"#
    );

    // get ABI and bytecode for the SimpleRevertingStorage contract
    let (abi, bytecode) = compile_contract("SimpleRevertingStorage", "SimpleRevertingStorage.sol");

    // let anvil = Anvil::new().spawn();
    // let provider = Arc::new(Provider::try_from(anvil.endpoint()).unwrap());
    let provider = Arc::new(Provider::try_from("http://127.0.0.1:8545").unwrap());

    let factory = ContractFactory::new(abi.clone(), bytecode.clone(), provider.clone());

    let contract = factory
        .deploy("hello".to_string())
        .expect("deploy prep failed")
        .legacy()
        .send()
        .await
        .expect("deploy failed");

    let contract = SimpleRevertingStorage::new(contract.address(), provider.clone());

    let empty_err = contract.empty_revert().await.unwrap_err();
    assert!(empty_err.as_provider_error().unwrap().is_error_response(), "expected error response");

    let s_err = contract.string_revert("hello".to_owned()).await.unwrap_err();
    assert!(s_err.is_revert());
}
