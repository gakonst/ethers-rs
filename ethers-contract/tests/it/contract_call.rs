use ethers_contract_derive::abigen;
use ethers_core::abi::Address;
use ethers_providers::Provider;
use std::{
    future::{Future, IntoFuture},
    sync::Arc,
};

fn _contract_call_into_future_is_send() {
    abigen!(DsProxyFactory, "./../ethers-middleware/contracts/DSProxyFactory.json");
    let (provider, _) = Provider::mocked();
    let client = Arc::new(provider);
    let contract = DsProxyFactory::new(Address::zero(), client);

    fn is_send<T: Future + Send + 'static>(future: T) -> T {
        future
    }

    drop(is_send(contract.cache().into_future()));
}
