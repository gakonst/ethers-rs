use async_trait::async_trait;
use ethers_core::types::{
    transaction::eip2718::TypedTransaction, BlockId, BlockNumber, FilterBlockOption,
    TransactionReceipt,
};
use std::sync::Arc;
use thiserror::Error;

use ethers_providers::{maybe, FromErr, Middleware};

type TimeLagResult<T, M> = Result<T, TimeLagError<M>>;

/// TimeLag Provider Errors
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
            FilterBlockOption::Range {
                from_block: _,
                to_block: None,
            } => Ok(block_option.set_to_block(self.get_block_number().await?.into())),
            _ => Ok(block_option),
        }
    }
}

#[async_trait]
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
        let block_number = self
            .inner()
            .get_block_number()
            .await
            .map(|num| num - K)
            .map_err(ethers_providers::FromErr::from)?;
        Ok(block_number - K)
    }

    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<ethers_providers::PendingTransaction<'_, Self::Provider>, Self::Error> {
        let block = self.normalize_block_id(block).await?;
        self.inner()
            .send_transaction(tx, block)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn get_block<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<ethers_core::types::Block<ethers_core::types::TxHash>>, Self::Error> {
        // unwrap here is safe, as passing in Some will always return Some
        let block_hash_or_number = self
            .normalize_block_id(Some(block_hash_or_number.into()))
            .await?
            .unwrap();

        self.inner()
            .get_block(block_hash_or_number)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<ethers_core::types::Block<ethers_core::types::Transaction>>, Self::Error>
    {
        // unwrap here is safe, as passing in Some will always return Some
        let block_hash_or_number = self
            .normalize_block_id(Some(block_hash_or_number.into()))
            .await?
            .unwrap();

        self.inner()
            .get_block_with_txs(block_hash_or_number)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn get_uncle_count<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<ethers_core::types::U256, Self::Error> {
        // unwrap here is safe, as passing in Some will always return Some
        let block_hash_or_number = self
            .normalize_block_id(Some(block_hash_or_number.into()))
            .await?
            .unwrap();

        self.inner()
            .get_uncle_count(block_hash_or_number)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn get_uncle<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
        idx: ethers_core::types::U64,
    ) -> Result<Option<ethers_core::types::Block<ethers_core::types::TxHash>>, Self::Error> {
        // unwrap here is safe, as passing in Some will always return Some
        let block_hash_or_number = self
            .normalize_block_id(Some(block_hash_or_number.into()))
            .await?
            .unwrap();

        self.inner()
            .get_uncle(block_hash_or_number, idx)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn get_transaction_count<T: Into<ethers_core::types::NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<ethers_core::types::U256, Self::Error> {
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
    ) -> Result<ethers_core::types::Bytes, Self::Error> {
        let block = self.normalize_block_id(block).await?;

        self.inner()
            .call(tx, block)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn get_balance<T: Into<ethers_core::types::NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<ethers_core::types::U256, Self::Error> {
        let block = self.normalize_block_id(block).await?;
        self.inner()
            .get_balance(from, block)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn get_transaction_receipt<T: Send + Sync + Into<ethers_core::types::TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        let receipt = self
            .inner()
            .get_transaction_receipt(transaction_hash)
            .await
            .map_err(ethers_providers::FromErr::from)?;

        if let Some(receipt) = receipt {
            if let Some(number) = receipt.block_number {
                if number > self.get_block_number().await? {
                    Ok(None)
                } else {
                    Ok(Some(receipt))
                }
            } else {
                Ok(Some(receipt))
            }
        } else {
            Ok(None)
        }
    }

    async fn get_code<T: Into<ethers_core::types::NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<BlockId>,
    ) -> Result<ethers_core::types::Bytes, Self::Error> {
        let block = self.normalize_block_id(block).await?;

        self.inner()
            .get_code(at, block)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn get_storage_at<T: Into<ethers_core::types::NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: ethers_core::types::TxHash,
        block: Option<BlockId>,
    ) -> Result<ethers_core::types::TxHash, Self::Error> {
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

        let tx_clone = tx.clone();

        // TODO: Maybe deduplicate the code in a nice way
        match tx {
            TypedTransaction::Legacy(ref mut inner) => {
                if let Some(ethers_core::types::NameOrAddress::Name(ref ens_name)) = inner.to {
                    let addr = self.resolve_name(ens_name).await?;
                    inner.to = Some(addr.into());
                };

                if inner.from.is_none() {
                    inner.from = self.default_sender();
                }

                let (gas_price, gas) = futures_util::try_join!(
                    maybe(inner.gas_price, self.get_gas_price()),
                    maybe(inner.gas, self.estimate_gas(&tx_clone)),
                )?;
                inner.gas = Some(gas);
                inner.gas_price = Some(gas_price);
            }
            TypedTransaction::Eip2930(inner) => {
                if let Ok(lst) = self.create_access_list(&tx_clone, block).await {
                    inner.access_list = lst.access_list;
                }

                if let Some(ethers_core::types::NameOrAddress::Name(ref ens_name)) = inner.tx.to {
                    let addr = self.resolve_name(ens_name).await?;
                    inner.tx.to = Some(addr.into());
                };

                if inner.tx.from.is_none() {
                    inner.tx.from = self.default_sender();
                }

                let (gas_price, gas) = futures_util::try_join!(
                    maybe(inner.tx.gas_price, self.get_gas_price()),
                    maybe(inner.tx.gas, self.estimate_gas(&tx_clone)),
                )?;
                inner.tx.gas = Some(gas);
                inner.tx.gas_price = Some(gas_price);
            }
            TypedTransaction::Eip1559(inner) => {
                if let Ok(lst) = self.create_access_list(&tx_clone, block).await {
                    inner.access_list = lst.access_list;
                }

                if let Some(ethers_core::types::NameOrAddress::Name(ref ens_name)) = inner.to {
                    let addr = self.resolve_name(ens_name).await?;
                    inner.to = Some(addr.into());
                };

                if inner.from.is_none() {
                    inner.from = self.default_sender();
                }

                let gas = ethers_providers::maybe(inner.gas, self.estimate_gas(&tx_clone)).await?;
                inner.gas = Some(gas);

                if inner.max_fee_per_gas.is_none() || inner.max_priority_fee_per_gas.is_none() {
                    let (max_fee_per_gas, max_priority_fee_per_gas) =
                        self.estimate_eip1559_fees(None).await?;
                    if inner.max_fee_per_gas.is_none() {
                        inner.max_fee_per_gas = Some(max_fee_per_gas);
                    }
                    if inner.max_priority_fee_per_gas.is_none() {
                        inner.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
                    }
                }
            }
        };

        Ok(())
    }

    async fn get_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>, Self::Error> {
        let block: BlockNumber = block.into();
        // unwrap here is safe, as passing in Some will always return Some
        let block = self.normalize_block_number(Some(block)).await?.unwrap();

        self.inner()
            .get_block_receipts(block)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn get_logs(
        &self,
        filter: &ethers_core::types::Filter,
    ) -> Result<Vec<ethers_core::types::Log>, Self::Error> {
        let mut filter = filter.clone();
        filter.block_option = self.normalize_filter_range(filter.block_option).await?;

        self.inner()
            .get_logs(&filter)
            .await
            .map_err(ethers_providers::FromErr::from)
    }

    async fn new_filter(
        &self,
        _filter: ethers_providers::FilterKind<'_>,
    ) -> Result<ethers_core::types::U256, Self::Error> {
        Err(TimeLagError::Unsupported)
    }

    async fn get_filter_changes<T, R>(&self, _id: T) -> Result<Vec<R>, Self::Error>
    where
        T: Into<ethers_core::types::U256> + Send + Sync,
        R: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + std::fmt::Debug,
    {
        Err(TimeLagError::Unsupported)
    }

    async fn watch_blocks(
        &self,
    ) -> Result<
        ethers_providers::FilterWatcher<'_, Self::Provider, ethers_core::types::TxHash>,
        Self::Error,
    > {
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
        T: Into<ethers_core::types::U256> + Send + Sync,
        Self::Provider: ethers_providers::PubsubClient,
    {
        Err(TimeLagError::Unsupported)
    }

    async fn subscribe_blocks(
        &self,
    ) -> Result<
        ethers_providers::SubscriptionStream<
            '_,
            Self::Provider,
            ethers_core::types::Block<ethers_core::types::TxHash>,
        >,
        Self::Error,
    >
    where
        Self::Provider: ethers_providers::PubsubClient,
    {
        Err(TimeLagError::Unsupported)
    }

    async fn subscribe_pending_txs(
        &self,
    ) -> Result<
        ethers_providers::SubscriptionStream<'_, Self::Provider, ethers_core::types::TxHash>,
        Self::Error,
    >
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
