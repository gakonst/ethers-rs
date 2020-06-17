use ethers::{
    providers::{Http, Provider},
    types::TransactionRequest,
    utils::{parse_ether, Ganache},
};
use std::convert::TryFrom;

#[tokio::test]
async fn pending_txs_with_confirmations_ganache() {
    let _ganache = Ganache::new().block_time(2u64).spawn();
    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let accounts = provider.get_accounts().await.unwrap();

    let tx = TransactionRequest::pay(accounts[1], parse_ether(1u64).unwrap()).from(accounts[0]);
    let pending_tx = provider.send_transaction(tx).await.unwrap();
    let hash = *pending_tx;
    let receipt = pending_tx.confirmations(5).await.unwrap();

    // got the correct receipt
    assert_eq!(receipt.transaction_hash, hash);
}
