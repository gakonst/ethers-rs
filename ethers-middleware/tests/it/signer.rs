use crate::{get_wallet, spawn_anvil, spawn_anvil_ws};
use ethers_core::types::*;
use ethers_middleware::{signer::SignerMiddleware, MiddlewareBuilder};
use ethers_providers::{JsonRpcClient, Middleware};
use ethers_signers::{LocalWallet, Signer};

#[tokio::test]
async fn send_eth() {
    let (provider, anvil) = spawn_anvil();
    let wallet = get_wallet(&anvil, 0);
    let address = wallet.address();
    let provider = provider.with_signer(wallet);

    let to = anvil.addresses()[1];

    // craft the transaction
    let tx = TransactionRequest::new().to(to).value(10000);

    let balance_before = provider.get_balance(address, None).await.unwrap();

    // send it!
    provider.send_transaction(tx, None).await.unwrap().await.unwrap().unwrap();

    let balance_after = provider.get_balance(address, None).await.unwrap();

    assert!(balance_before > balance_after);
}

#[tokio::test]
async fn pending_txs_with_confirmations_testnet() {
    let (provider, anvil) = spawn_anvil();
    let wallet = get_wallet(&anvil, 0);
    let address = wallet.address();
    let provider = provider.with_signer(wallet);
    generic_pending_txs_test(provider, address).await;
}

#[tokio::test]
async fn websocket_pending_txs_with_confirmations_testnet() {
    let (provider, anvil) = spawn_anvil_ws().await;
    let wallet = get_wallet(&anvil, 0);
    let address = wallet.address();
    let provider = provider.with_signer(wallet);
    generic_pending_txs_test(provider, address).await;
}

#[tokio::test]
async fn typed_txs() {
    let (provider, anvil) = spawn_anvil();
    let wallet = get_wallet(&anvil, 0);
    let address = wallet.address();
    let provider = provider.with_signer(wallet);

    let nonce = provider.get_transaction_count(address, None).await.unwrap();
    let bn = Some(BlockNumber::Pending.into());
    let gas_price = provider.get_gas_price().await.unwrap() * 125 / 100;

    let tx = TransactionRequest::new().from(address).to(address).nonce(nonce).gas_price(gas_price);
    let tx1 = provider.send_transaction(tx.clone(), bn).await.unwrap();

    let tx = tx.from(address).to(address).nonce(nonce + 1).with_access_list(vec![]);
    let tx2 = provider.send_transaction(tx, bn).await.unwrap();

    let tx = Eip1559TransactionRequest::new()
        .from(address)
        .to(address)
        .nonce(nonce + 2)
        .max_fee_per_gas(gas_price)
        .max_priority_fee_per_gas(gas_price);
    let tx3 = provider.send_transaction(tx, bn).await.unwrap();

    futures_util::join!(check_tx(tx1, 0), check_tx(tx2, 1), check_tx(tx3, 2));
}

#[tokio::test]
async fn send_transaction_handles_tx_from_field() {
    // launch anvil
    let (provider, anvil) = spawn_anvil_ws().await;

    // grab 2 wallets
    let signer: LocalWallet = anvil.keys()[0].clone().into();
    let other: LocalWallet = anvil.keys()[1].clone().into();

    // connect to the network
    let provider =
        SignerMiddleware::new_with_provider_chain(provider, signer.clone()).await.unwrap();

    // sending a TransactionRequest with a from field of None should result
    // in a transaction from the signer address
    let request_from_none = TransactionRequest::new();
    let receipt =
        provider.send_transaction(request_from_none, None).await.unwrap().await.unwrap().unwrap();
    let sent_tx = provider.get_transaction(receipt.transaction_hash).await.unwrap().unwrap();

    assert_eq!(sent_tx.from, signer.address());

    // sending a TransactionRequest with the signer as the from address should
    // result in a transaction from the signer address
    let request_from_signer = TransactionRequest::new().from(signer.address());
    let receipt =
        provider.send_transaction(request_from_signer, None).await.unwrap().await.unwrap().unwrap();
    let sent_tx = provider.get_transaction(receipt.transaction_hash).await.unwrap().unwrap();

    assert_eq!(sent_tx.from, signer.address());

    // sending a TransactionRequest with a from address that is not the signer
    // should result in a transaction from the specified address
    let request_from_other = TransactionRequest::new().from(other.address());
    let receipt =
        provider.send_transaction(request_from_other, None).await.unwrap().await.unwrap().unwrap();
    let sent_tx = provider.get_transaction(receipt.transaction_hash).await.unwrap().unwrap();

    assert_eq!(sent_tx.from, other.address());
}

async fn generic_pending_txs_test<M: Middleware>(provider: M, who: Address) {
    let tx = TransactionRequest::new().to(who).from(who);
    let pending_tx = provider.send_transaction(tx, None).await.unwrap();
    let tx_hash = *pending_tx;
    let receipt = pending_tx.confirmations(1).await.unwrap().unwrap();
    // got the correct receipt
    assert_eq!(receipt.transaction_hash, tx_hash);
}

async fn check_tx<P: JsonRpcClient + Clone>(
    pending_tx: ethers_providers::PendingTransaction<'_, P>,
    expected: u64,
) {
    let provider = pending_tx.provider();
    let receipt = pending_tx.await.unwrap().unwrap();
    let tx = provider.get_transaction(receipt.transaction_hash).await.unwrap().unwrap();

    let expected = U64::from(expected);
    for ty in [receipt.transaction_type, tx.transaction_type] {
        // legacy can be either None or Some(0)
        if expected.is_zero() {
            assert!(ty.is_none() || ty == Some(0.into()));
        } else {
            assert_eq!(ty, Some(expected));
        }
    }
}
