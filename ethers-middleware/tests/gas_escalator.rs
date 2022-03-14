#![cfg(not(target_arch = "wasm32"))]
use ethers_core::types::*;
use ethers_middleware::{
    gas_escalator::{Frequency, GasEscalatorMiddleware, GeometricGasPrice},
    signer::SignerMiddleware,
};
use ethers_providers::Middleware;
use ethers_signers::{LocalWallet, Signer};
use std::time::Duration;

#[tokio::test]
#[ignore]
async fn gas_escalator_live() {
    // connect to ropsten for getting bad block times
    let provider = ethers_providers::ROPSTEN.ws().await;
    let provider = provider.interval(Duration::from_millis(2000u64));
    let wallet = "fdb33e2105f08abe41a8ee3b758726a31abdd57b7a443f470f23efce853af169"
        .parse::<LocalWallet>()
        .unwrap();
    let address = wallet.address();
    let provider = SignerMiddleware::new(provider, wallet);

    let escalator = GeometricGasPrice::new(5.0, 10u64, Some(2_000_000_000_000u64));

    let provider = GasEscalatorMiddleware::new(provider, escalator, Frequency::Duration(3000));

    let nonce = provider.get_transaction_count(address, None).await.unwrap();
    let tx = TransactionRequest::pay(Address::zero(), 1u64).gas_price(10_000_000);

    // broadcast 3 txs
    provider.send_transaction(tx.clone().nonce(nonce), None).await.unwrap();
    provider.send_transaction(tx.clone().nonce(nonce + 1), None).await.unwrap();
    provider.send_transaction(tx.clone().nonce(nonce + 2), None).await.unwrap();

    // Wait a bunch of seconds and refresh etherscan to see the transactions get bumped
    tokio::time::sleep(std::time::Duration::from_secs(100)).await;

    // TODO: Figure out how to test this behavior properly in a local network. If the gas price was
    // bumped then the tx hash will be different
}
