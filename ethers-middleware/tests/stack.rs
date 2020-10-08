#[tokio::test]
#[cfg(not(feature = "celo"))]
async fn can_stack_middlewares() {
    use ethers_core::{types::TransactionRequest, utils::Ganache};
    use ethers_middleware::{
        gas_escalator::{Frequency, GasEscalatorMiddleware, GeometricGasPrice},
        gas_oracle::{GasCategory, GasNow, GasOracleMiddleware},
        nonce_manager::NonceManagerMiddleware,
        signer::SignerMiddleware,
    };
    use ethers_providers::{Http, Middleware, Provider};
    use ethers_signers::LocalWallet;
    use std::convert::TryFrom;

    let ganache = Ganache::new().block_time(5u64).spawn();
    let gas_oracle = GasNow::new().category(GasCategory::SafeLow);
    let signer: LocalWallet = ganache.keys()[0].clone().into();
    let address = signer.address();

    // the base provider
    let provider = Provider::<Http>::try_from(ganache.endpoint()).unwrap();
    let provider_clone = provider.clone();

    // the Gas Price escalator middleware is the first middleware above the provider,
    // so that it receives the transaction last, after all the other middleware
    // have modified it accordingly
    let escalator = GeometricGasPrice::new(1.125, 60u64, None::<u64>);
    let provider = GasEscalatorMiddleware::new(provider, escalator, Frequency::PerBlock);

    // The gas price middleware MUST be below the signing middleware for things to work
    let provider = GasOracleMiddleware::new(provider, gas_oracle);

    // The signing middleware signs txs
    let provider = SignerMiddleware::new(provider, signer);

    // The nonce manager middleware MUST be above the signing middleware so that it overrides
    // the nonce and the signer does not make any eth_getTransaction count calls
    let provider = NonceManagerMiddleware::new(provider, address);

    let tx = TransactionRequest::new();
    let mut tx_hash = None;
    for _ in 0..10 {
        tx_hash = Some(provider.send_transaction(tx.clone(), None).await.unwrap());
        dbg!(
            provider
                .get_transaction(tx_hash.unwrap())
                .await
                .unwrap()
                .unwrap()
                .gas_price
        );
    }

    let receipt = provider_clone
        .pending_transaction(tx_hash.unwrap())
        .await
        .unwrap();

    dbg!(receipt);
}
