use super::{GasOracle, GasOracleError};
use ethers_core::types::*;
use ethers_providers::{FilterKind, FilterWatcher, Middleware, PendingTransaction};

use async_trait::async_trait;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug)]
pub struct GasOracleMiddleware<M, G> {
    inner: M,
    gas_oracle: G,
}

impl<M, G> GasOracleMiddleware<M, G>
where
    M: Middleware,
    G: GasOracle,
{
    pub fn new(inner: M, gas_oracle: G) -> Self {
        Self { inner, gas_oracle }
    }
}

#[derive(Error, Debug)]
pub enum MiddlewareError<M: Middleware> {
    #[error(transparent)]
    GasOracleError(#[from] GasOracleError),

    #[error("{0}")]
    MiddlewareError(M::Error),
}

#[async_trait(?Send)]
impl<M, G> Middleware for GasOracleMiddleware<M, G>
where
    M: Middleware,
    G: GasOracle,
{
    type Error = MiddlewareError<M>;
    type Provider = M::Provider;

    // OVERRIDEN METHODS

    async fn get_gas_price(&self) -> Result<U256, Self::Error> {
        Ok(self.gas_oracle.fetch().await?)
    }

    // DELEGATED METHODS

    async fn get_block_number(&self) -> Result<U64, Self::Error> {
        self.inner
            .get_block_number()
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<TxHash, Self::Error> {
        if tx.gas_price.is_none() {
            tx.gas_price = Some(self.get_gas_price().await?);
        }
        self.inner
            .send_transaction(tx, block)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn resolve_name(&self, ens_name: &str) -> Result<Address, Self::Error> {
        self.inner
            .resolve_name(ens_name)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn lookup_address(&self, address: Address) -> Result<String, Self::Error> {
        self.inner
            .lookup_address(address)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn get_block<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<TxHash>>, Self::Error> {
        self.inner
            .get_block(block_hash_or_number)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<Transaction>>, Self::Error> {
        self.inner
            .get_block_with_txs(block_hash_or_number)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockNumber>,
    ) -> Result<U256, Self::Error> {
        self.inner
            .get_transaction_count(from, block)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<U256, Self::Error> {
        self.inner
            .estimate_gas(tx)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn call(
        &self,
        tx: &TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<Bytes, Self::Error> {
        self.inner
            .call(tx, block)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn get_chainid(&self) -> Result<U256, Self::Error> {
        self.inner
            .get_chainid()
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockNumber>,
    ) -> Result<U256, Self::Error> {
        self.inner
            .get_balance(from, block)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<Transaction>, Self::Error> {
        self.inner
            .get_transaction(transaction_hash)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        self.inner
            .get_transaction_receipt(transaction_hash)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn get_accounts(&self) -> Result<Vec<Address>, Self::Error> {
        self.inner
            .get_accounts()
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn send_raw_transaction(&self, tx: &Transaction) -> Result<TxHash, Self::Error> {
        self.inner
            .send_raw_transaction(tx)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn sign<T: Into<Bytes> + Send + Sync>(
        &self,
        data: T,
        from: &Address,
    ) -> Result<Signature, Self::Error> {
        self.inner
            .sign(data, from)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    ////// Contract state

    async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, Self::Error> {
        self.inner
            .get_logs(filter)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn new_filter(&self, filter: FilterKind<'_>) -> Result<U256, Self::Error> {
        self.inner
            .new_filter(filter)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn uninstall_filter<T: Into<U256> + Send + Sync>(
        &self,
        id: T,
    ) -> Result<bool, Self::Error> {
        self.inner
            .uninstall_filter(id)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn watch<'a>(
        &'a self,
        filter: &Filter,
    ) -> Result<FilterWatcher<'a, Self::Provider, Log>, Self::Error> {
        self.inner
            .watch(filter)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn watch_pending_transactions(
        &self,
    ) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
        self.inner
            .watch_pending_transactions()
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn get_filter_changes<T, R>(&self, id: T) -> Result<Vec<R>, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        R: for<'a> Deserialize<'a> + Send + Sync,
    {
        self.inner
            .get_filter_changes(id)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn watch_blocks(&self) -> Result<FilterWatcher<'_, Self::Provider, H256>, Self::Error> {
        self.inner
            .watch_blocks()
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<BlockNumber>,
    ) -> Result<Bytes, Self::Error> {
        self.inner
            .get_code(at, block)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: H256,
        block: Option<BlockNumber>,
    ) -> Result<H256, Self::Error> {
        self.inner
            .get_storage_at(from, location, block)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }

    fn pending_transaction(&self, tx_hash: TxHash) -> PendingTransaction<'_, Self::Provider> {
        self.inner.pending_transaction(tx_hash)
    }
}
