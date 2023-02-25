use crate::spawn_anvil;
use ethers_core::types::*;
use ethers_middleware::MiddlewareBuilder;
use ethers_providers::Middleware;

#[tokio::test]
async fn nonce_manager() {
    let (provider, anvil) = spawn_anvil();
    let address = anvil.addresses()[0];
    let to = anvil.addresses()[1];

    // the nonce manager must be over the Client so that it overrides the nonce
    // before the client gets it
    let provider = provider.nonce_manager(address);

    let nonce = provider
        .get_transaction_count(address, Some(BlockNumber::Pending.into()))
        .await
        .unwrap()
        .as_u64();

    let num_tx = 3;
    let mut tx_hashes = Vec::with_capacity(num_tx);
    for _ in 0..num_tx {
        let tx = provider
            .send_transaction(TransactionRequest::new().from(address).to(to).value(100u64), None)
            .await
            .unwrap();
        tx_hashes.push(*tx);
    }

    let mut nonces = Vec::with_capacity(num_tx);
    for tx_hash in tx_hashes {
        nonces.push(provider.get_transaction(tx_hash).await.unwrap().unwrap().nonce.as_u64());
    }

    assert_eq!(nonces, (nonce..nonce + num_tx as u64).collect::<Vec<_>>());
}
