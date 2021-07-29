use ethers_providers::{Http, Middleware, Provider};

use ethers_core::types::TransactionRequest;
use ethers_middleware::signer::SignerMiddleware;
use ethers_signers::{LocalWallet, Signer};
use std::{convert::TryFrom, time::Duration};

#[tokio::test]
#[cfg(not(feature = "celo"))]
async fn send_eth() {
    use ethers_core::utils::Ganache;

    let ganache = Ganache::new().spawn();

    // this private key belongs to the above mnemonic
    let wallet: LocalWallet = ganache.keys()[0].clone().into();
    let wallet2: LocalWallet = ganache.keys()[1].clone().into();

    // connect to the network
    let provider = Provider::<Http>::try_from(ganache.endpoint())
        .unwrap()
        .interval(Duration::from_millis(10u64));
    let chain_id = provider.get_chainid().await.unwrap().as_u64();
    let wallet = wallet.with_chain_id(chain_id);
    let provider = SignerMiddleware::new(provider, wallet);

    // craft the transaction
    let tx = TransactionRequest::new().to(wallet2.address()).value(10000);

    let balance_before = provider
        .get_balance(provider.address(), None)
        .await
        .unwrap();

    // send it!
    provider.send_transaction(tx, None).await.unwrap();

    let balance_after = provider
        .get_balance(provider.address(), None)
        .await
        .unwrap();

    assert!(balance_before > balance_after);
}

#[tokio::test]
#[cfg(feature = "celo")]
async fn test_send_transaction() {
    // Celo testnet
    let provider = Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org")
        .unwrap()
        .interval(Duration::from_millis(3000u64));
    let chain_id = provider.get_chainid().await.unwrap().as_u64();

    // Funded with https://celo.org/developers/faucet
    // Please do not drain this account :)
    let wallet = "d652abb81e8c686edba621a895531b1f291289b63b5ef09a94f686a5ecdd5db1"
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id);
    let client = SignerMiddleware::new(provider, wallet);

    let balance_before = client.get_balance(client.address(), None).await.unwrap();
    let tx = TransactionRequest::pay(client.address(), 100);
    let _receipt = client
        .send_transaction(tx, None)
        .await
        .unwrap()
        .confirmations(3)
        .await
        .unwrap();
    let balance_after = client.get_balance(client.address(), None).await.unwrap();
    assert!(balance_before > balance_after);
}

#[tokio::test]
#[cfg(not(feature = "celo"))]
async fn send_transaction_handles_tx_from_field() {
    use ethers_core::utils::Ganache;

    // launch ganache
    let ganache = Ganache::new().spawn();

    // grab 2 wallets
    let signer: LocalWallet = ganache.keys()[0].clone().into();
    let other: LocalWallet = ganache.keys()[1].clone().into();

    // connect to the network
    let provider = Provider::try_from(ganache.endpoint()).unwrap();
    let provider = SignerMiddleware::new(provider, signer.clone());

    // sending a TransactionRequest with a from field of None should result
    // in a transaction from the signer address
    let request_from_none = TransactionRequest::new();
    let receipt = provider
        .send_transaction(request_from_none, None)
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    let sent_tx = provider
        .get_transaction(receipt.transaction_hash)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(sent_tx.from, signer.address());

    // sending a TransactionRequest with the signer as the from address should
    // result in a transaction from the signer address
    let request_from_signer = TransactionRequest::new().from(signer.address());
    let receipt = provider
        .send_transaction(request_from_signer, None)
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    let sent_tx = provider
        .get_transaction(receipt.transaction_hash)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(sent_tx.from, signer.address());

    // sending a TransactionRequest with a from address that is not the signer
    // should result in a transaction from the specified address
    let request_from_other = TransactionRequest::new().from(other.address());
    let receipt = provider
        .send_transaction(request_from_other, None)
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    let sent_tx = provider
        .get_transaction(receipt.transaction_hash)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(sent_tx.from, other.address());
}
