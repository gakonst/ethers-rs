#![allow(unused_braces)]
use ethers::providers::{Http, Provider};
use std::{convert::TryFrom, time::Duration};

#[cfg(not(feature = "celo"))]
mod eth_tests {
    use super::*;
    use ethers::{
        providers::JsonRpcClient,
        types::TransactionRequest,
        utils::{parse_ether, Ganache},
    };

    // Without TLS this would error with "TLS Support not compiled in"
    #[test]
    #[cfg(any(feature = "async-std-tls", feature = "tokio-tls"))]
    fn ssl_websocket() {
        // this is extremely ugly but I couldn't figure out a better way of having
        // a shared async test for both runtimes
        #[cfg(feature = "async-std-tls")]
        let block_on = async_std::task::block_on;
        #[cfg(feature = "tokio-tls")]
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        #[cfg(feature = "tokio-tls")]
        let mut block_on = |x| runtime.block_on(x);

        use ethers::providers::Ws;
        block_on(async move {
            let ws = Ws::connect("wss://rinkeby.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27")
                .await
                .unwrap();
            let provider = Provider::new(ws);
            let _number = provider.get_block_number().await.unwrap();
        });
    }

    #[tokio::test]
    #[cfg(feature = "tokio-runtime")]
    async fn watch_blocks_websocket() {
        use ethers::{
            providers::{FilterStream, StreamExt, Ws},
            types::H256,
        };

        let ganache = Ganache::new().block_time(2u64).spawn();
        let (ws, _) = async_tungstenite::tokio::connect_async(ganache.ws_endpoint())
            .await
            .unwrap();
        let provider = Provider::new(Ws::new(ws)).interval(Duration::from_millis(500u64));

        let stream = provider.watch_blocks().await.unwrap().stream();

        let _blocks = stream.take(3usize).collect::<Vec<H256>>().await;
        let _number = provider.get_block_number().await.unwrap();
    }

    #[tokio::test]
    async fn pending_txs_with_confirmations_ganache() {
        let ganache = Ganache::new().block_time(2u64).spawn();
        let provider = Provider::<Http>::try_from(ganache.endpoint())
            .unwrap()
            .interval(Duration::from_millis(500u64));
        generic_pending_txs_test(provider).await;
    }

    #[tokio::test]
    #[cfg(any(feature = "tokio-runtime", feature = "tokio-tls"))]
    async fn websocket_pending_txs_with_confirmations_ganache() {
        use ethers::providers::Ws;
        let ganache = Ganache::new().block_time(2u64).spawn();
        let ws = Ws::connect(ganache.ws_endpoint()).await.unwrap();
        let provider = Provider::new(ws);
        generic_pending_txs_test(provider).await;
    }

    async fn generic_pending_txs_test<P: JsonRpcClient>(provider: Provider<P>) {
        let accounts = provider.get_accounts().await.unwrap();

        let tx = TransactionRequest::pay(accounts[0], parse_ether(1u64).unwrap()).from(accounts[0]);
        let tx_hash = provider.send_transaction(tx).await.unwrap();
        let pending_tx = provider.pending_transaction(tx_hash);
        let receipt = pending_tx.confirmations(5).await.unwrap();

        // got the correct receipt
        assert_eq!(receipt.transaction_hash, tx_hash);
    }
}

#[cfg(feature = "celo")]
mod celo_tests {
    use super::*;
    use ethers::{
        providers::FilterStream,
        types::{Randomness, H256},
    };
    use futures_util::stream::StreamExt;
    use rustc_hex::FromHex;

    #[tokio::test]
    // https://alfajores-blockscout.celo-testnet.org/tx/0x544ea96cddb16aeeaedaf90885c1e02be4905f3eb43d6db3f28cac4dbe76a625/internal_transactions
    async fn get_transaction() {
        let provider =
            Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org").unwrap();

        let tx_hash = "c8496681d0ade783322980cce00c89419fce4b484635d9e09c79787a0f75d450"
            .parse::<H256>()
            .unwrap();
        let tx = provider.get_transaction(tx_hash).await.unwrap();
        assert!(tx.gateway_fee_recipient.is_none());
        assert_eq!(tx.gateway_fee.unwrap(), 0.into());
        assert_eq!(tx.hash, tx_hash);
        assert_eq!(tx.block_number.unwrap(), 447181.into())
    }

    #[tokio::test]
    async fn get_block() {
        let provider =
            Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org").unwrap();

        let block = provider.get_block(447254).await.unwrap();
        assert_eq!(
            block.randomness,
            Randomness {
                committed: "003e12deb86292844274493e9ab6e57ed1e276202c16799d97af723eb0d3253f"
                    .from_hex::<Vec<u8>>()
                    .unwrap()
                    .into(),
                revealed: "1333b3b45e0385da48a01b4459aeda7607867ef6a41167cfdeefa49b9fdce6d7"
                    .from_hex::<Vec<u8>>()
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
