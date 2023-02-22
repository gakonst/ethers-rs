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
