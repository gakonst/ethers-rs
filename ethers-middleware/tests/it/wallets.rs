static WALLETS: Lazy<TestWallets> = Lazy::new(|| {
    TestWallets {
        mnemonic: MnemonicBuilder::default()
            // Please don't drain this :)
            .phrase("impose air often almost medal sudden finish quote dwarf devote theme layer"),
        next: Default::default(),
    }
});

#[derive(Debug, Default)]
struct TestWallets {
    mnemonic: MnemonicBuilder<English>,
    next: AtomicU8,
}

impl TestWallets {
    /// Helper for funding the wallets with an instantiated provider
    #[allow(unused)]
    pub async fn fund<T: JsonRpcClient, U: Into<u32>>(&self, provider: &Provider<T>, n: U) {
        let addrs = (0..n.into()).map(|i| self.get(i).address()).collect::<Vec<_>>();
        // hardcoded funder address private key, GOERLI
        let signer = "9867bd0f8d9e16c57f5251b35a73f6f903eb8eee1bdc7f15256d0dc09d1945fb"
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(provider.get_chainid().await.unwrap().as_u64());
        let provider = SignerMiddleware::new(provider, signer);
        let addr = provider.address();

        let mut nonce = provider.get_transaction_count(addr, None).await.unwrap();
        let mut pending_txs = Vec::new();
        for addr in addrs {
            println!("Funding wallet {addr:?}");
            let tx = TransactionRequest::new()
                .nonce(nonce)
                .to(addr)
                // 0.1 eth per wallet
                .value(parse_ether("1").unwrap());
            pending_txs.push(
                provider.send_transaction(tx, Some(BlockNumber::Pending.into())).await.unwrap(),
            );
            nonce += 1.into();
        }

        futures_util::future::join_all(pending_txs).await;
    }

    pub fn next(&self) -> LocalWallet {
        let idx = self.next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // println!("Got wallet {:?}", wallet.address());
        self.get(idx)
    }

    fn get<T: Into<u32>>(&self, idx: T) -> LocalWallet {
        self.mnemonic
            .clone()
            .index(idx)
            .expect("index not found")
            .build()
            .expect("cannot build wallet")
    }
}
