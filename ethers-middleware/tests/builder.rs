#![cfg(not(target_arch = "wasm32"))]
#[cfg(not(feature = "celo"))]
mod tests {
    use ethers_core::{rand::thread_rng, types::U64};
    use ethers_middleware::{
        builder::MiddlewareBuilder,
        gas_escalator::{Frequency, GasEscalatorMiddleware, GeometricGasPrice},
        gas_oracle::{EthGasStation, GasOracleMiddleware},
        nonce_manager::NonceManagerMiddleware,
        signer::SignerMiddleware,
    };
    use ethers_providers::{Middleware, Provider};
    use ethers_signers::{LocalWallet, Signer};

    #[tokio::test]
    async fn build_raw_middleware_stack() {
        let (provider, mock) = Provider::mocked();

        let signer = LocalWallet::new(&mut thread_rng());
        let address = signer.address();
        let escalator = GeometricGasPrice::new(1.125, 60u64, None::<u64>);

        let provider = provider
            .wrap_into(|p| GasEscalatorMiddleware::new(p, escalator, Frequency::PerBlock))
            .wrap_into(|p| GasOracleMiddleware::new(p, EthGasStation::new(None)))
            .wrap_into(|p| SignerMiddleware::new(p, signer))
            .wrap_into(|p| NonceManagerMiddleware::new(p, address));

        // push a response
        mock.push(U64::from(12u64)).unwrap();
        let block: U64 = provider.get_block_number().await.unwrap();
        assert_eq!(block.as_u64(), 12);

        provider.get_block_number().await.unwrap_err();

        // 2 calls were made
        mock.assert_request("eth_blockNumber", ()).unwrap();
        mock.assert_request("eth_blockNumber", ()).unwrap();
        mock.assert_request("eth_blockNumber", ()).unwrap_err();
    }

    #[tokio::test]
    async fn build_declarative_middleware_stack() {
        let (provider, mock) = Provider::mocked();

        let signer = LocalWallet::new(&mut thread_rng());
        let address = signer.address();
        let escalator = GeometricGasPrice::new(1.125, 60u64, None::<u64>);
        let gas_oracle = EthGasStation::new(None);

        let provider = provider
            .wrap_into(|p| GasEscalatorMiddleware::new(p, escalator, Frequency::PerBlock))
            .gas_oracle(gas_oracle)
            .with_signer(signer)
            .nonce_manager(address);

        // push a response
        mock.push(U64::from(12u64)).unwrap();
        let block: U64 = provider.get_block_number().await.unwrap();
        assert_eq!(block.as_u64(), 12);

        provider.get_block_number().await.unwrap_err();

        // 2 calls were made
        mock.assert_request("eth_blockNumber", ()).unwrap();
        mock.assert_request("eth_blockNumber", ()).unwrap();
        mock.assert_request("eth_blockNumber", ()).unwrap_err();
    }
}
