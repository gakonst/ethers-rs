#![cfg(all(not(target_arch = "wasm32"), not(feature = "celo")))]

use ethers_core::types::*;
use ethers_middleware::{nonce_manager::NonceManagerMiddleware, signer::SignerMiddleware};
use ethers_providers::Middleware;
use ethers_signers::{LocalWallet, Signer};
use std::time::Duration;

#[tokio::test]
async fn nonce_manager() {
    let provider = ethers_providers::SEPOLIA.provider().interval(Duration::from_millis(2000));
    let chain_id = provider.get_chainid().await.unwrap().as_u64();

    let wallet = std::env::var("SEPOLIA_PRIVATE_KEY")
        .expect("SEPOLIA_PRIVATE_KEY is not defined")
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id);
    let address = wallet.address();

    let provider = SignerMiddleware::new(provider, wallet);

    // the nonce manager must be over the Client so that it overrides the nonce
    // before the client gets it
    let provider = NonceManagerMiddleware::new(provider, address);

    let nonce = provider
        .get_transaction_count(address, Some(BlockNumber::Pending.into()))
        .await
        .unwrap()
        .as_u64();

    let num_tx = 3;
    let mut tx_hashes = Vec::with_capacity(num_tx);
    for _ in 0..num_tx {
        let tx = provider
            .send_transaction(
                Eip1559TransactionRequest::new().to(address).value(100u64).chain_id(chain_id),
                None,
            )
            .await
            .unwrap();
        tx_hashes.push(*tx);
    }

    tokio::time::sleep(Duration::from_secs(15)).await;

    let mut nonces = Vec::with_capacity(num_tx);
    for tx_hash in tx_hashes {
        nonces.push(provider.get_transaction(tx_hash).await.unwrap().unwrap().nonce.as_u64());
    }

    assert_eq!(nonces, (nonce..nonce + num_tx as u64).collect::<Vec<_>>())
}
