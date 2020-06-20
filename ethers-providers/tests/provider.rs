use ethers::providers::{Http, Provider, StreamExt, Ws};
use std::convert::TryFrom;

#[cfg(not(feature = "celo"))]
mod eth_tests {
    use super::*;
    use async_tungstenite::tokio::connect_async;
    use ethers::{
        providers::FilterStream,
        types::{TransactionRequest, H256},
        utils::{parse_ether, Ganache},
    };
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn watch_blocks_websocket() {
        let _ganache = Ganache::new().block_time(2u64).spawn();
        let (ws, _) = connect_async("ws://localhost:8545").await.unwrap();
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
        let accounts = provider.get_accounts().await.unwrap();

        let tx = TransactionRequest::pay(accounts[1], parse_ether(1u64).unwrap()).from(accounts[0]);
        let pending_tx = provider.send_transaction(tx).await.unwrap();
        let hash = *pending_tx;
        let receipt = pending_tx.confirmations(5).await.unwrap();

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
