use ethers::{
    providers::{
        gas_oracle::{Etherchain, GasCategory, GasOracle},
        Http, Provider,
    },
    signers::Wallet,
    types::TransactionRequest,
};
use std::{convert::TryFrom, time::Duration};

#[cfg(not(feature = "celo"))]
mod eth_tests {
    use super::*;
    use ethers::{
        types::BlockNumber,
        utils::{parse_ether, Ganache},
    };

    #[tokio::test]
    async fn pending_txs_with_confirmations_rinkeby_infura() {
        let provider = Provider::<Http>::try_from(
            "https://rinkeby.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
        )
        .unwrap()
        .interval(Duration::from_millis(2000u64));

        // pls do not drain this key :)
        // note: this works even if there's no EIP-155 configured!
        let client = "FF7F80C6E9941865266ED1F481263D780169F1D98269C51167D20C630A5FDC8A"
            .parse::<Wallet>()
            .unwrap()
            .connect(provider);

        let tx = TransactionRequest::pay(client.address(), parse_ether(1u64).unwrap());
        let tx_hash = client
            .send_transaction(tx, Some(BlockNumber::Pending))
            .await
            .unwrap();
        let receipt = client
            .pending_transaction(tx_hash)
            .confirmations(3)
            .await
            .unwrap();

        // got the correct receipt
        assert_eq!(receipt.transaction_hash, tx_hash);
    }

    #[tokio::test]
    async fn send_eth() {
        let ganache = Ganache::new().spawn();

        // this private key belongs to the above mnemonic
        let wallet: Wallet = ganache.keys()[0].clone().into();
        let wallet2: Wallet = ganache.keys()[1].clone().into();

        // connect to the network
        let provider = Provider::<Http>::try_from(ganache.endpoint())
            .unwrap()
            .interval(Duration::from_millis(10u64));

        // connect the wallet to the provider
        let client = wallet.connect(provider);

        // craft the transaction
        let tx = TransactionRequest::new().to(wallet2.address()).value(10000);

        let balance_before = client.get_balance(client.address(), None).await.unwrap();

        // send it!
        client.send_transaction(tx, None).await.unwrap();

        let balance_after = client.get_balance(client.address(), None).await.unwrap();

        assert!(balance_before > balance_after);
    }

    #[tokio::test]
    async fn nonce_manager() {
        let provider = Provider::<Http>::try_from(
            "https://rinkeby.infura.io/v3/fd8b88b56aa84f6da87b60f5441d6778",
        )
        .unwrap()
        .interval(Duration::from_millis(2000u64));

        let client = "59c37cb6b16fa2de30675f034c8008f890f4b2696c729d6267946d29736d73e4"
            .parse::<Wallet>()
            .unwrap()
            .connect(provider)
            .with_nonce_manager();

        let nonce = client
            .get_transaction_count(Some(BlockNumber::Pending))
            .await
            .unwrap()
            .as_u64();

        let mut tx_hashes = Vec::new();
        for _ in 0..10 {
            let tx = client
                .send_transaction(
                    TransactionRequest::pay(client.address(), 100u64),
                    Some(BlockNumber::Pending),
                )
                .await
                .unwrap();
            tx_hashes.push(tx);
        }

        let mut nonces = Vec::new();
        for tx_hash in tx_hashes {
            nonces.push(
                client
                    .get_transaction(tx_hash)
                    .await
                    .unwrap()
                    .unwrap()
                    .nonce
                    .as_u64(),
            );
        }

        assert_eq!(nonces, (nonce..nonce + 10).collect::<Vec<_>>())
    }

    #[tokio::test]
    async fn using_gas_oracle() {
        let ganache = Ganache::new().spawn();

        // this private key belongs to the above mnemonic
        let wallet: Wallet = ganache.keys()[0].clone().into();
        let wallet2: Wallet = ganache.keys()[1].clone().into();

        // connect to the network
        let provider = Provider::<Http>::try_from(ganache.endpoint())
            .unwrap()
            .interval(Duration::from_millis(10u64));

        // connect the wallet to the provider
        let client = wallet.connect(provider);

        // assign a gas oracle to use
        let gas_oracle = Etherchain::new().category(GasCategory::Fastest);
        let expected_gas_price = gas_oracle.fetch().await.unwrap();

        let client = client.gas_oracle(Box::new(gas_oracle));

        // broadcast a transaction
        let tx = TransactionRequest::new().to(wallet2.address()).value(10000);
        let tx_hash = client.send_transaction(tx, None).await.unwrap();

        let tx = client.get_transaction(tx_hash).await.unwrap().unwrap();
        assert_eq!(tx.gas_price, expected_gas_price);
    }
}

#[cfg(feature = "celo")]
mod celo_tests {
    use super::*;

    #[tokio::test]
    async fn test_send_transaction() {
        // Celo testnet
        let provider = Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org")
            .unwrap()
            .interval(Duration::from_millis(3000u64));

        // Funded with https://celo.org/developers/faucet
        // Please do not drain this account :)
        let client = "d652abb81e8c686edba621a895531b1f291289b63b5ef09a94f686a5ecdd5db1"
            .parse::<Wallet>()
            .unwrap()
            .connect(provider);

        let balance_before = client.get_balance(client.address(), None).await.unwrap();
        let tx = TransactionRequest::pay(client.address(), 100);
        let tx_hash = client.send_transaction(tx, None).await.unwrap();
        let _receipt = client
            .pending_transaction(tx_hash)
            .confirmations(3)
            .await
            .unwrap();
        let balance_after = client.get_balance(client.address(), None).await.unwrap();
        assert!(balance_before > balance_after);
    }
}
