use ethers_core::{types::*, utils::Anvil};
use ethers_middleware::{
    gas_escalator::{Frequency, GasEscalatorMiddleware, GeometricGasPrice},
    MiddlewareBuilder,
};
use ethers_providers::{Http, Middleware, Provider};
use ethers_signers::{LocalWallet, Signer};

#[tokio::test]
#[ignore]
async fn gas_escalator() {
    let anvil = Anvil::new().block_time(2u64).spawn();
    let chain_id = anvil.chain_id();
    let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();

    // wrap with signer
    let wallet: LocalWallet = anvil.keys().first().unwrap().clone().into();
    let wallet = wallet.with_chain_id(chain_id);
    let address = wallet.address();
    let provider = provider.with_signer(wallet);

    // wrap with escalator
    let escalator = GeometricGasPrice::new(5.0, 10u64, Some(2_000_000_000_000u64));
    let provider = GasEscalatorMiddleware::new(provider, escalator, Frequency::Duration(300));

    let nonce = provider.get_transaction_count(address, None).await.unwrap();
    // 1 gwei default base fee
    let gas_price = U256::from(1_000_000_000_u64);
    let tx = TransactionRequest::pay(Address::zero(), 1u64)
        .gas_price(gas_price)
        .nonce(nonce)
        .chain_id(chain_id);

    eprintln!("sending");
    let pending = provider.send_transaction(tx, None).await.expect("could not send");
    eprintln!("waiting");
    let receipt = pending.await.expect("reverted").expect("dropped");
    assert_eq!(receipt.from, address);
    assert_eq!(receipt.to, Some(Address::zero()));
    assert!(receipt.effective_gas_price.unwrap() > gas_price * 2, "{receipt:?}");
}
