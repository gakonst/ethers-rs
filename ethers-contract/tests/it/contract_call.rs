use std::future::Future;
use std::sync::Arc;
use ethers_contract_derive::abigen;
use ethers_core::abi::Address;
use ethers_providers::Provider;
use std::future::IntoFuture;

#[tokio::test]
async fn contract_call_into_future_is_send() {
    abigen!(DsProxyFactory, "ethers-middleware/contracts/DsProxyFactory.json");
    let (provider, _) = Provider::mocked();
    let client = Arc::new(provider);
    let contract = DsProxyFactory::new(Address::zero(), client);

    fn is_send<T: Future + Send + 'static>(future: T) -> bool { true }

    assert!(is_send(contract.cache().into_future()));
}
