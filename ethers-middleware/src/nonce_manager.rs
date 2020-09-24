use async_trait::async_trait;
use ethers_core::types::*;
use ethers_providers::{FilterKind, FilterWatcher, Middleware, PendingTransaction};
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Debug)]
pub struct NonceManager<M> {
    pub inner: M,
    pub initialized: AtomicBool,
    pub nonce: AtomicU64,
    pub address: Address,
}

impl<M> NonceManager<M>
where
    M: Middleware,
{
    /// Instantiates the nonce manager with a 0 nonce.
    pub fn new(inner: M, address: Address) -> Self {
        NonceManager {
            initialized: false.into(),
            nonce: 0.into(),
            inner,
            address,
        }
    }

    /// Returns the next nonce to be used
    pub fn next(&self) -> U256 {
        let nonce = self.nonce.fetch_add(1, Ordering::SeqCst);
        nonce.into()
    }

    async fn get_transaction_count_with_manager(
        &self,
        block: Option<BlockNumber>,
    ) -> Result<U256, M::Error> {
        // initialize the nonce the first time the manager is called
        if !self.initialized.load(Ordering::SeqCst) {
            let nonce = self
                .inner
                .get_transaction_count(self.address, block)
                .await?;
            self.nonce.store(nonce.as_u64(), Ordering::SeqCst);
            self.initialized.store(true, Ordering::SeqCst);
        }

        return Ok(self.next());
    }
}

#[async_trait(?Send)]
impl<M> Middleware for NonceManager<M>
where
    M: Middleware,
{
    type Error = M::Error;
    type Provider = M::Provider;

    /// Signs and broadcasts the transaction. The optional parameter `block` can be passed so that
    /// gas cost and nonce calculations take it into account. For simple transactions this can be
    /// left to `None`.
    async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<TxHash, Self::Error> {
        if tx.nonce.is_none() {
            tx.nonce = Some(self.get_transaction_count_with_manager(block).await?);
        }

        let mut tx_clone = tx.clone();
        match self.inner.send_transaction(tx, block).await {
            Ok(tx_hash) => Ok(tx_hash),
            Err(err) => {
                let nonce = self.get_transaction_count(self.address, block).await?;
                if nonce != self.nonce.load(Ordering::SeqCst).into() {
                    // try re-submitting the transaction with the correct nonce if there
                    // was a nonce mismatch
                    self.nonce.store(nonce.as_u64(), Ordering::SeqCst);
                    tx_clone.nonce = Some(nonce);
                    self.inner.send_transaction(tx_clone, block).await
                } else {
                    // propagate the error otherwise
                    return Err(err);
                }
            }
        }
    }

    // DELEGATED

    async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockNumber>,
    ) -> Result<U256, Self::Error> {
        self.inner.get_transaction_count(from, block).await
    }

    /// Signs a message with the internal signer, or if none is present it will make a call to
    /// the connected node's `eth_call` API.
    async fn sign<T: Into<Bytes> + Send + Sync>(
        &self,
        data: T,
        address: &Address,
    ) -> Result<Signature, Self::Error> {
        self.inner.sign(data, address).await
    }

    async fn get_gas_price(&self) -> Result<U256, Self::Error> {
        self.inner.get_gas_price().await
    }

    async fn get_block_number(&self) -> Result<U64, Self::Error> {
        self.inner.get_block_number().await
    }

    async fn resolve_name(&self, ens_name: &str) -> Result<Address, Self::Error> {
        self.inner.resolve_name(ens_name).await
    }

    async fn lookup_address(&self, address: Address) -> Result<String, Self::Error> {
        self.inner.lookup_address(address).await
    }

    async fn get_block<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<TxHash>>, Self::Error> {
        self.inner.get_block(block_hash_or_number).await
    }

    async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<Transaction>>, Self::Error> {
        self.inner.get_block_with_txs(block_hash_or_number).await
    }

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<U256, Self::Error> {
        self.inner.estimate_gas(tx).await
    }

    async fn call(
        &self,
        tx: &TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<Bytes, Self::Error> {
        self.inner.call(tx, block).await
    }

    async fn get_chainid(&self) -> Result<U256, Self::Error> {
        self.inner.get_chainid().await
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockNumber>,
    ) -> Result<U256, Self::Error> {
        self.inner.get_balance(from, block).await
    }

    async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<Transaction>, Self::Error> {
        self.inner.get_transaction(transaction_hash).await
    }

    async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        self.inner.get_transaction_receipt(transaction_hash).await
    }

    async fn get_accounts(&self) -> Result<Vec<Address>, Self::Error> {
        self.inner.get_accounts().await
    }

    async fn send_raw_transaction(&self, tx: &Transaction) -> Result<TxHash, Self::Error> {
        self.inner.send_raw_transaction(tx).await
    }

    async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, Self::Error> {
        self.inner.get_logs(filter).await
    }

    async fn new_filter(&self, filter: FilterKind<'_>) -> Result<U256, Self::Error> {
        self.inner.new_filter(filter).await
    }

    async fn uninstall_filter<T: Into<U256> + Send + Sync>(
        &self,
        id: T,
    ) -> Result<bool, Self::Error> {
        self.inner.uninstall_filter(id).await
    }

    async fn watch<'a>(
        &'a self,
        filter: &Filter,
    ) -> Result<FilterWatcher<'a, Self::Provider, Log>, Self::Error> {
        self.inner.watch(filter).await
    }

    async fn watch_pending_transactions(
        &self,
    ) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
        self.inner.watch_pending_transactions().await
    }

    async fn get_filter_changes<T, R>(&self, id: T) -> Result<Vec<R>, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        R: for<'a> Deserialize<'a> + Send + Sync,
    {
        self.inner.get_filter_changes(id).await
    }

    async fn watch_blocks(&self) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
        self.inner.watch_blocks().await
    }

    async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<BlockNumber>,
    ) -> Result<Bytes, Self::Error> {
        self.inner.get_code(at, block).await
    }

    async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: H256,
        block: Option<BlockNumber>,
    ) -> Result<H256, Self::Error> {
        self.inner.get_storage_at(from, location, block).await
    }

    fn pending_transaction(&self, tx_hash: TxHash) -> PendingTransaction<'_, Self::Provider> {
        self.inner.pending_transaction(tx_hash)
    }
}
