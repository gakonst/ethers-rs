use async_trait::async_trait;
use ethers_core::types::{
    transaction::eip2718::TypedTransaction, Block, BlockId, BlockNumber, Bytes, FilterBlockOption,
    NameOrAddress, Transaction, TransactionReceipt, TxHash, U256,
};
use std::sync::Arc;
use thiserror::Error;

use ethers_providers::{FromErr, Middleware};

type TimeLagResult<T, M> = Result<T, TimeLagError<M>>;

/// TimeLage Provider Errors
#[derive(Error, Debug)]
pub enum TimeLagError<M>
where
    M: Middleware,
{
    #[error("{0}")]
    /// Thrown when an internal middleware errors
    MiddlewareError(M::Error),

    #[error("Unsupported RPC. Timelag provider does not support filters or subscriptions.")]
    Unsupported,
}

// Boilerplate
impl<M: Middleware> FromErr<M::Error> for TimeLagError<M> {
    fn from(src: M::Error) -> TimeLagError<M> {
        TimeLagError::MiddlewareError(src)
    }
}

/// TimeLag Provider
#[derive(Debug)]
pub struct TimeLag<M, const K: u8> {
    inner: Arc<M>,
}

impl<M, const K: u8> TimeLag<M, K>
where
    M: Middleware,
{
    /// Instantiates TimeLag provider
    pub fn new(inner: M) -> Self {
        Self { inner: inner.into() }
    }
}

impl<M, const K: u8> TimeLag<M, K>
where
    M: Middleware,
{
    async fn normalize_block_id(&self, id: Option<BlockId>) -> TimeLagResult<Option<BlockId>, M> {
        match id {
            Some(BlockId::Number(n)) => {
                Ok(self.normalize_block_number(Some(n)).await?.map(Into::into))
            }
            _ => Ok(id),
        }
    }

    async fn normalize_block_number(
        &self,
        number: Option<BlockNumber>,
    ) -> TimeLagResult<Option<BlockNumber>, M> {
        let tip = self.get_block_number().await?;
        match number {
            Some(BlockNumber::Latest) => Ok(Some(BlockNumber::Number(tip))),
            Some(BlockNumber::Number(n)) => {
                if n > tip {
                    Ok(Some(BlockNumber::Latest))
                } else {
                    Ok(number)
                }
            }
            _ => Ok(number),
        }
    }

    async fn normalize_filter_range(
        &self,
        block_option: FilterBlockOption,
    ) -> TimeLagResult<FilterBlockOption, M> {
        match block_option {
            FilterBlockOption::Range { from_block: _, to_block: None } => {
                Ok(block_option.set_to_block(self.get_block_number().await?.into()))
            }
            _ => Ok(block_option),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<M, const K: u8> Middleware for TimeLag<M, K>
where
    M: Middleware,
{
    type Error = TimeLagError<M>;

    type Provider = M::Provider;

    type Inner = M;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    async fn get_block_number(&self) -> Result<ethers_core::types::U64, Self::Error> {
        self.inner()
            .get_block_number()
            .await
            .map(|num| num - K)
            .map_err(ethers_providers::FromErr::from)
    }

    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<ethers_providers::PendingTransaction<'_, Self::Provider>, Self::Error> {
        let block = self.normalize_block_id(block).await?;
        self.inner().send_transaction(tx, block).await.map_err(ethers_providers::FromErr::from)
    }

    async fn get_block<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<TxHash>>, Self::Error> {
        let block_hash_or_number = self
            .normalize_block_id(Some(block_hash_or_number.into()))
            .await?
            .expect("Cannot return None if Some is passed in");

        self.inner().get_block(block_hash_or_number).await.map_err(ethers_providers::FromErr::from)
    }

    async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<Transaction>>, Self::Error> {
        let block_hash_or_number = self
            .normalize_block_id(Some(block_hash_or_number.into()))
            .await?
            .expect("Cannot return None if Some is passed in");

        self.inner()
            .get_block_with_txs(block_hash_or_number)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn get_uncle_count<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<U256, Self::Error> {
        let block_hash_or_number = self
            .normalize_block_id(Some(block_hash_or_number.into()))
            .await?
            .expect("Cannot return None if Some is passed in");

        self.inner()
            .get_uncle_count(block_hash_or_number)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn get_uncle<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
        idx: ethers_core::types::U64,
    ) -> Result<Option<Block<TxHash>>, Self::Error> {
        let block_hash_or_number = self
            .normalize_block_id(Some(block_hash_or_number.into()))
            .await?
            .expect("Cannot return None if Some is passed in");

        self.inner()
            .get_uncle(block_hash_or_number, idx)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        let block = self.normalize_block_id(block).await?;

        self.inner()
            .get_transaction_count(from, block)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        let block = self.normalize_block_id(block).await?;

        self.inner().call(tx, block).await.map_err(ethers_providers::FromErr::from)
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        let block = self.normalize_block_id(block).await?;
        self.inner().get_balance(from, block).await.map_err(ethers_providers::FromErr::from)
    }

    async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        let receipt = self
            .inner()
            .get_transaction_receipt(transaction_hash)
            .await
            .map_err(ethers_providers::FromErr::from)?;

        if receipt.is_none() {
            return Ok(None)
        }

        let receipt = receipt.expect("checked is_none");
        if receipt.block_number.is_none() {
            return Ok(Some(receipt))
        }

        let number = receipt.block_number.expect("checked is_none");
        if number <= self.get_block_number().await? {
            Ok(Some(receipt))
        } else {
            // Pretend it hasn't confirmed yet.
            Ok(None)
        }
    }

    async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        let block = self.normalize_block_id(block).await?;

        self.inner().get_code(at, block).await.map_err(ethers_providers::FromErr::from)
    }

    async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: TxHash,
        block: Option<BlockId>,
    ) -> Result<TxHash, Self::Error> {
        let block = self.normalize_block_id(block).await?;
        self.inner()
            .get_storage_at(from, location, block)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn fill_transaction(
        &self,
        tx: &mut TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<(), Self::Error> {
        let block = self.normalize_block_id(block).await?;
        self.inner().fill_transaction(tx, block).await.map_err(ethers_providers::FromErr::from)
    }

    async fn get_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>, Self::Error> {
        let block: BlockNumber = block.into();
        let block = self
            .normalize_block_number(Some(block))
            .await?
            .expect("Cannot return None if Some is passed in");

        self.inner().get_block_receipts(block).await.map_err(ethers_providers::FromErr::from)
    }

    async fn get_logs(
        &self,
        filter: &ethers_core::types::Filter,
    ) -> Result<Vec<ethers_core::types::Log>, Self::Error> {
        let mut filter = filter.clone();
        filter.block_option = self.normalize_filter_range(filter.block_option).await?;

        self.inner().get_logs(&filter).await.map_err(ethers_providers::FromErr::from)
    }

    async fn new_filter(
        &self,
        _filter: ethers_providers::FilterKind<'_>,
    ) -> Result<U256, Self::Error> {
        Err(TimeLagError::Unsupported)
    }

    async fn get_filter_changes<T, R>(&self, _id: T) -> Result<Vec<R>, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        R: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + std::fmt::Debug,
    {
        Err(TimeLagError::Unsupported)
    }

    async fn watch_blocks(
        &self,
    ) -> Result<ethers_providers::FilterWatcher<'_, Self::Provider, TxHash>, Self::Error> {
        Err(TimeLagError::Unsupported)
    }

    async fn subscribe<T, R>(
        &self,
        _params: T,
    ) -> Result<ethers_providers::SubscriptionStream<'_, Self::Provider, R>, Self::Error>
    where
        T: std::fmt::Debug + serde::Serialize + Send + Sync,
        R: serde::de::DeserializeOwned + Send + Sync,
        Self::Provider: ethers_providers::PubsubClient,
    {
        Err(TimeLagError::Unsupported)
    }

    async fn unsubscribe<T>(&self, _id: T) -> Result<bool, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        Self::Provider: ethers_providers::PubsubClient,
    {
        Err(TimeLagError::Unsupported)
    }

    async fn subscribe_blocks(
        &self,
    ) -> Result<ethers_providers::SubscriptionStream<'_, Self::Provider, Block<TxHash>>, Self::Error>
    where
        Self::Provider: ethers_providers::PubsubClient,
    {
        Err(TimeLagError::Unsupported)
    }

    async fn subscribe_pending_txs(
        &self,
    ) -> Result<ethers_providers::SubscriptionStream<'_, Self::Provider, TxHash>, Self::Error>
    where
        Self::Provider: ethers_providers::PubsubClient,
    {
        Err(TimeLagError::Unsupported)
    }

    async fn subscribe_logs<'a>(
        &'a self,
        _filter: &ethers_core::types::Filter,
    ) -> Result<
        ethers_providers::SubscriptionStream<'a, Self::Provider, ethers_core::types::Log>,
        Self::Error,
    >
    where
        Self::Provider: ethers_providers::PubsubClient,
    {
        Err(TimeLagError::Unsupported)
    }
}
