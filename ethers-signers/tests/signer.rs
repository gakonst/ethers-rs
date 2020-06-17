use ethers::{
    providers::{Http, Provider},
    signers::Wallet,
    types::TransactionRequest,
};
use std::convert::TryFrom;

#[cfg(not(feature = "celo"))]
mod eth_tests {
    use super::*;
    use ethers::utils::{parse_ether, Ganache};

    #[tokio::test]
    async fn pending_txs_with_confirmations_rinkeby_infura() {
        let provider = Provider::<Http>::try_from(
            "https://rinkeby.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
        )
        .unwrap();

        // pls do not drain this key :)
        // note: this works even if there's no EIP-155 configured!
        let client = "FF7F80C6E9941865266ED1F481263D780169F1D98269C51167D20C630A5FDC8A"
            .parse::<Wallet>()
            .unwrap()
            .connect(provider);

        let tx = TransactionRequest::pay(client.address(), parse_ether(1u64).unwrap());
        let pending_tx = client.send_transaction(tx, None).await.unwrap();
        let hash = *pending_tx;
        dbg!(hash);
        let receipt = pending_tx.confirmations(3).await.unwrap();

        // got the correct receipt
        assert_eq!(receipt.transaction_hash, hash);
    }

    #[tokio::test]
    async fn send_eth() {
        let port = 8545u64;
        let url = format!("http://localhost:{}", port).to_string();
        let _ganache = Ganache::new()
            .port(port)
            .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
            .spawn();

        // this private key belongs to the above mnemonic
        let wallet: Wallet = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
            .parse()
            .unwrap();

        // connect to the network
        let provider = Provider::<Http>::try_from(url.as_str()).unwrap();

        // connect the wallet to the provider
        let client = wallet.connect(provider);

        // craft the transaction
        let tx = TransactionRequest::new()
            .send_to_str("986eE0C8B91A58e490Ee59718Cca41056Cf55f24")
            .unwrap()
            .value(10000);

        let balance_before = client.get_balance(client.address(), None).await.unwrap();

        // send it!
        client.send_transaction(tx, None).await.unwrap();

        let balance_after = client.get_balance(client.address(), None).await.unwrap();

        assert!(balance_before > balance_after);
    }
}

#[cfg(feature = "celo")]
mod celo_tests {
    use super::*;

    #[tokio::test]
    async fn test_send_transaction() {
        // Celo testnet
        let provider =
            Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org").unwrap();

        // Funded with https://celo.org/developers/faucet
        // Please do not drain this account :)
        let client = "d652abb81e8c686edba621a895531b1f291289b63b5ef09a94f686a5ecdd5db1"
            .parse::<Wallet>()
            .unwrap()
            .connect(provider);

        let balance_before = client.get_balance(client.address(), None).await.unwrap();
        let tx = TransactionRequest::pay(client.address(), 100);
        let pending_tx = client.send_transaction(tx, None).await.unwrap();
        let _receipt = pending_tx.confirmations(3).await.unwrap();
        let balance_after = client.get_balance(client.address(), None).await.unwrap();
        assert!(balance_before > balance_after);
    }
}
