use crate::{base::decode_event, ContractError, EventStream};

use ethers_core::{
    abi::{Detokenize, Event as AbiEvent},
    types::{BlockNumber, Filter, Log, TxHash, ValueOrArray, H256, U64},
};
use ethers_providers::{FilterWatcher, Middleware, PubsubClient, SubscriptionStream};
use std::marker::PhantomData;

/// Helper for managing the event filter before querying or streaming its logs
#[derive(Debug)]
#[must_use = "event filters do nothing unless you `query` or `stream` them"]
pub struct Event<'a: 'b, 'b, M, D> {
    /// The event filter's state
    pub filter: Filter,
    /// The ABI of the event which is being filtered
    pub event: &'b AbiEvent,
    pub(crate) provider: &'a M,
    pub(crate) datatype: PhantomData<D>,
}

// TODO: Improve these functions
impl<M, D: Detokenize> Event<'_, '_, M, D> {
    /// Sets the filter's `from` block
    #[allow(clippy::wrong_self_convention)]
    pub fn from_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.filter.from_block = Some(block.into());
        self
    }

    /// Sets the filter's `to` block
    #[allow(clippy::wrong_self_convention)]
    pub fn to_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.filter.to_block = Some(block.into());
        self
    }

    /// Sets the filter's 0th topic (typically the event name for non-anonymous events)
    pub fn topic0<T: Into<ValueOrArray<H256>>>(mut self, topic: T) -> Self {
        self.filter.topics[0] = Some(topic.into());
        self
    }

    /// Sets the filter's 1st topic
    pub fn topic1<T: Into<ValueOrArray<H256>>>(mut self, topic: T) -> Self {
        self.filter.topics[1] = Some(topic.into());
        self
    }

    /// Sets the filter's 2nd topic
    pub fn topic2<T: Into<ValueOrArray<H256>>>(mut self, topic: T) -> Self {
        self.filter.topics[2] = Some(topic.into());
        self
    }

    /// Sets the filter's 3rd topic
    pub fn topic3<T: Into<ValueOrArray<H256>>>(mut self, topic: T) -> Self {
        self.filter.topics[3] = Some(topic.into());
        self
    }
}

impl<'a, 'b, M, D> Event<'a, 'b, M, D>
where
    M: Middleware,
    D: 'b + Detokenize + Clone,
{
    /// Returns a stream for the event
    pub async fn stream(
        &'a self,
    ) -> Result<
        // Wraps the FilterWatcher with a mapping to the event
        EventStream<'a, FilterWatcher<'a, M::Provider, Log>, D, ContractError<M>>,
        ContractError<M>,
    > {
        let filter = self
            .provider
            .watch(&self.filter)
            .await
            .map_err(ContractError::MiddlewareError)?;
        Ok(EventStream::new(
            filter.id,
            filter,
            Box::new(move |log| self.parse_log(log)),
        ))
    }
}

impl<'a, 'b, M, D> Event<'a, 'b, M, D>
where
    M: Middleware,
    <M as Middleware>::Provider: PubsubClient,
    D: 'b + Detokenize + Clone,
{
    /// Returns a subscription for the event
    pub async fn subscribe(
        &'a self,
    ) -> Result<
        // Wraps the SubscriptionStream with a mapping to the event
        EventStream<'a, SubscriptionStream<'a, M::Provider, Log>, D, ContractError<M>>,
        ContractError<M>,
    > {
        let filter = self
            .provider
            .subscribe_logs(&self.filter)
            .await
            .map_err(ContractError::MiddlewareError)?;
        Ok(EventStream::new(
            filter.id,
            filter,
            Box::new(move |log| self.parse_log(log)),
        ))
    }
}

impl<M, D> Event<'_, '_, M, D>
where
    M: Middleware,
    D: Detokenize + Clone,
{
    /// Queries the blockchain for the selected filter and returns a vector of matching
    /// event logs
    pub async fn query(&self) -> Result<Vec<D>, ContractError<M>> {
        let logs = self
            .provider
            .get_logs(&self.filter)
            .await
            .map_err(ContractError::MiddlewareError)?;
        let events = logs
            .into_iter()
            .map(|log| self.parse_log(log))
            .collect::<Result<Vec<_>, ContractError<M>>>()?;
        Ok(events)
    }

    /// Queries the blockchain for the selected filter and returns a vector of logs
    /// along with their metadata
    pub async fn query_with_meta(&self) -> Result<Vec<(D, LogMeta)>, ContractError<M>> {
        let logs = self
            .provider
            .get_logs(&self.filter)
            .await
            .map_err(ContractError::MiddlewareError)?;
        let events = logs
            .into_iter()
            .map(|log| {
                let meta = LogMeta::from(&log);
                let event = self.parse_log(log)?;
                Ok((event, meta))
            })
            .collect::<Result<_, ContractError<M>>>()?;
        Ok(events)
    }

    fn parse_log(&self, log: Log) -> Result<D, ContractError<M>> {
        Ok(decode_event(self.event, log.topics, log.data)?)
    }
}

/// Metadata inside a log
#[derive(Clone, Debug, PartialEq)]
pub struct LogMeta {
    /// The block in which the log was emitted
    pub block_number: U64,

    /// The transaction hash in which the log was emitted
    pub transaction_hash: TxHash,
}

impl From<&Log> for LogMeta {
    fn from(src: &Log) -> Self {
        LogMeta {
            block_number: src.block_number.expect("should have a block number"),
            transaction_hash: src.transaction_hash.expect("should have a tx hash"),
        }
    }
}
