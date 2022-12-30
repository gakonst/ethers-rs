#![cfg(not(target_arch = "wasm32"))]
use ethers_providers::{Http, Middleware, Provider};
use std::{convert::TryFrom, time::Duration};

#[cfg(not(feature = "celo"))]
mod eth_tests {
    use super::*;
    use ethers_core::{
        types::{Address, BlockId, TransactionRequest, H256},
        utils::Anvil,
    };
    use ethers_providers::GOERLI;

    #[tokio::test]
    async fn non_existing_data_works() {
        let provider = GOERLI.provider();

        assert!(provider.get_transaction(H256::zero()).await.unwrap().is_none());
        assert!(provider.get_transaction_receipt(H256::zero()).await.unwrap().is_none());
        assert!(provider.get_block(BlockId::Hash(H256::zero())).await.unwrap().is_none());
        assert!(provider.get_block_with_txs(BlockId::Hash(H256::zero())).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn client_version() {
        let provider = GOERLI.provider();

        // e.g., Geth/v1.10.6-omnibus-1af33248/linux-amd64/go1.16.6
        assert!(provider
            .client_version()
            .await
            .expect("Could not make web3_clientVersion call to provider")
            .starts_with("Geth/v"));
    }

    // Without TLS this would error with "TLS Support not compiled in"
    #[tokio::test]
    async fn ssl_websocket() {
        let provider = GOERLI.ws().await;
        let _number = provider.get_block_number().await.unwrap();
    }

    #[tokio::test]
    async fn watch_blocks_websocket() {
        use ethers_core::types::H256;
        use ethers_providers::{StreamExt, Ws};

        let anvil = Anvil::new().block_time(2u64).spawn();
        let (ws, _) = tokio_tungstenite::connect_async(anvil.ws_endpoint()).await.unwrap();
        let provider = Provider::new(Ws::new(ws)).interval(Duration::from_millis(500u64));

        let stream = provider.watch_blocks().await.unwrap().stream();

        let _blocks = stream.take(3usize).collect::<Vec<H256>>().await;
        let _number = provider.get_block_number().await.unwrap();
    }

    #[tokio::test]
    async fn pending_txs_with_confirmations_anvil() {
        let anvil = Anvil::new().block_time(2u64).spawn();
        let provider = Provider::<Http>::try_from(anvil.endpoint())
            .unwrap()
            .interval(Duration::from_millis(500u64));
        let accounts = provider.get_accounts().await.unwrap();
        generic_pending_txs_test(provider, accounts[0]).await;
    }

    #[tokio::test]
    async fn websocket_pending_txs_with_confirmations_anvil() {
        use ethers_providers::Ws;
        let anvil = Anvil::new().block_time(2u64).spawn();
        let ws = Ws::connect(anvil.ws_endpoint()).await.unwrap();
        let provider = Provider::new(ws);
        let accounts = provider.get_accounts().await.unwrap();
        generic_pending_txs_test(provider, accounts[0]).await;
    }

    async fn generic_pending_txs_test<M: Middleware>(provider: M, who: Address) {
        let tx = TransactionRequest::new().to(who).from(who);
        let pending_tx = provider.send_transaction(tx, None).await.unwrap();
        let tx_hash = *pending_tx;
        let receipt = pending_tx.confirmations(3).await.unwrap().unwrap();
        // got the correct receipt
        assert_eq!(receipt.transaction_hash, tx_hash);
    }

    #[tokio::test]
    async fn eip1559_fee_estimation() {
        let provider = ethers_providers::MAINNET.provider();

        let (_max_fee_per_gas, _max_priority_fee_per_gas) =
            provider.estimate_eip1559_fees(None).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_hardhat_compatibility() {
        use ethers_providers::RetryClient;

        async fn send_zst_requests<M: Middleware>(provider: M) {
            let _ = provider.get_chainid().await.unwrap();
            let _ = provider.get_block_number().await.unwrap();
            let _ = provider.get_gas_price().await.unwrap();
            let _ = provider.get_accounts().await.unwrap();
            let _ = provider.get_net_version().await.unwrap();
        }

        let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
        send_zst_requests(provider).await;

        let provider =
            Provider::<RetryClient<Http>>::new_client("http://localhost:8545", 10, 200).unwrap();

        send_zst_requests(provider).await;
    }
}

#[cfg(feature = "celo")]
mod celo_tests {
    use super::*;
    use ethers_core::types::{Randomness, H256};
    use futures_util::stream::StreamExt;

    #[tokio::test]
    async fn get_block() {
        let provider =
            Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org").unwrap();

        let block = provider.get_block(447254).await.unwrap().unwrap();
        assert_eq!(
            block.randomness,
            Randomness {
                committed: hex::decode(
                    "003e12deb86292844274493e9ab6e57ed1e276202c16799d97af723eb0d3253f"
                )
                .unwrap()
                .into(),
                revealed: hex::decode(
                    "1333b3b45e0385da48a01b4459aeda7607867ef6a41167cfdeefa49b9fdce6d7"
                )
                .unwrap()
                .into(),
            }
        );
    }

    #[tokio::test]
    async fn watch_blocks() {
        let provider = Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org")
            .unwrap()
            .interval(Duration::from_millis(2000u64));

        let stream = provider.watch_blocks().await.unwrap().stream();

        let _blocks = stream.take(3usize).collect::<Vec<H256>>().await;
    }
}
