#![allow(unused_braces)]
use ethers::providers::{Http, Provider};
use std::convert::TryFrom;

#[cfg(not(feature = "celo"))]
mod eth_tests {
    use super::*;
    use ethers::{
        providers::JsonRpcClient,
        types::TransactionRequest,
        utils::{parse_ether, Ganache},
    };
    use serial_test::serial;

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
    #[serial]
    #[cfg(feature = "tokio-runtime")]
    async fn watch_blocks_websocket() {
        use ethers::{
            providers::{FilterStream, StreamExt, Ws},
            types::H256,
        };

        let _ganache = Ganache::new().block_time(2u64).spawn();
        let (ws, _) = async_tungstenite::tokio::connect_async("ws://localhost:8545")
            .await
            .unwrap();
        let provider = Provider::new(Ws::new(ws));

        let stream = provider
            .watch_blocks()
            .await
            .unwrap()
            .interval(2000u64)
            .stream();

        let _blocks = stream.take(3usize).collect::<Vec<H256>>().await;
        let _number = provider.get_block_number().await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn pending_txs_with_confirmations_ganache() {
        let _ganache = Ganache::new().block_time(2u64).spawn();
        let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
        generic_pending_txs_test(provider).await;
    }

    #[tokio::test]
    #[serial]
    #[cfg(any(feature = "tokio-runtime", feature = "tokio-tls"))]
    async fn websocket_pending_txs_with_confirmations_ganache() {
        use ethers::providers::Ws;
        let _ganache = Ganache::new().block_time(2u64).port(8546u64).spawn();
        let ws = Ws::connect("ws://localhost:8546").await.unwrap();
        let provider = Provider::new(ws);
        generic_pending_txs_test(provider).await;
    }

    async fn generic_pending_txs_test<P: JsonRpcClient>(provider: Provider<P>) {
        let accounts = provider.get_accounts().await.unwrap();

        let tx = TransactionRequest::pay(accounts[0], parse_ether(1u64).unwrap()).from(accounts[0]);
        let pending_tx = provider.send_transaction(tx).await.unwrap();
        let hash = *pending_tx;
        let receipt = pending_tx.interval(500u64).confirmations(5).await.unwrap();

        // got the correct receipt
        assert_eq!(receipt.transaction_hash, hash);
    }
}

#[cfg(feature = "celo")]
mod celo_tests {
    use super::*;
    use ethers::{providers::FilterStream, types::H256};
    use futures_util::stream::StreamExt;

    #[tokio::test]
    // https://alfajores-blockscout.celo-testnet.org/tx/0x544ea96cddb16aeeaedaf90885c1e02be4905f3eb43d6db3f28cac4dbe76a625/internal_transactions
    async fn get_transaction() {
        let provider =
            Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org").unwrap();

        let tx_hash = "544ea96cddb16aeeaedaf90885c1e02be4905f3eb43d6db3f28cac4dbe76a625"
            .parse::<H256>()
            .unwrap();
        let tx = provider.get_transaction(tx_hash).await.unwrap();
        assert!(tx.gateway_fee_recipient.is_none());
        assert_eq!(tx.gateway_fee.unwrap(), 0.into());
        assert_eq!(tx.hash, tx_hash);
        assert_eq!(tx.block_number.unwrap(), 1100845.into())
    }

    #[tokio::test]
    async fn watch_blocks() {
        let provider =
            Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org").unwrap();

        let stream = provider
            .watch_blocks()
            .await
            .unwrap()
            .interval(2000u64)
            .stream();

        let _blocks = stream.take(3usize).collect::<Vec<H256>>().await;
    }
}
