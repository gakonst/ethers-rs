#![allow(clippy::return_self_not_must_use)]

use crate::{
    event_core::parse_log, log::LogMeta, stream::EventStream, ContractError, EthLogDecode,
};
use ethers_core::{
    abi::Address,
    types::{BlockNumber, Filter, Log, Topic, ValueOrArray, H256},
};
use ethers_providers::{FilterWatcher, Middleware, PubsubClient, SubscriptionStream};
use std::{borrow::Borrow, marker::PhantomData};

/// Helper for managing the event filter before querying or streaming its logs
#[derive(Debug)]
#[must_use = "event filters do nothing unless you `query` or `stream` them"]
pub struct Event<B, M, D> {
    /// The event filter's state
    pub filter: Filter,
    pub(crate) provider: B,
    /// Stores the event datatype
    pub(crate) datatype: PhantomData<D>,
    pub(crate) _m: PhantomData<M>,
}

// TODO: Improve these functions
impl<B, M, D> Event<B, M, D>
where
    B: Borrow<M>,
    M: Middleware,
    D: EthLogDecode,
{
    /// Sets the filter's `from` block
    #[allow(clippy::wrong_self_convention)]
    pub fn from_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.filter = self.filter.from_block(block);
        self
    }

    /// Sets the filter's `to` block
    #[allow(clippy::wrong_self_convention)]
    pub fn to_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.filter = self.filter.to_block(block);
        self
    }

    /// Sets the filter's `blockHash`. Setting this will override previously
    /// set `from_block` and `to_block` fields.
    #[allow(clippy::wrong_self_convention)]
    pub fn at_block_hash<T: Into<H256>>(mut self, hash: T) -> Self {
        self.filter = self.filter.at_block_hash(hash);
        self
    }

    /// Sets the filter's 0th topic (typically the event name for non-anonymous events)
    pub fn topic0<T: Into<Topic>>(mut self, topic: T) -> Self {
        self.filter.topics[0] = Some(topic.into());
        self
    }

    /// Sets the filter's 1st topic
    pub fn topic1<T: Into<Topic>>(mut self, topic: T) -> Self {
        self.filter.topics[1] = Some(topic.into());
        self
    }

    /// Sets the filter's 2nd topic
    pub fn topic2<T: Into<Topic>>(mut self, topic: T) -> Self {
        self.filter.topics[2] = Some(topic.into());
        self
    }

    /// Sets the filter's 3rd topic
    pub fn topic3<T: Into<Topic>>(mut self, topic: T) -> Self {
        self.filter.topics[3] = Some(topic.into());
        self
    }

    /// Sets the filter's address.
    pub fn address(mut self, address: ValueOrArray<Address>) -> Self {
        self.filter = self.filter.address(address);
        self
    }
}

impl<B, M, D> Event<B, M, D>
where
    B: Borrow<M>,
    M: Middleware,
    D: EthLogDecode,
{
    /// Turns this event filter into `Stream` that yields decoded events.
    ///
    /// This will first install a new logs filter via [`eth_newFilter`] using the configured
    /// `filter` object. See also [`FilterWatcher`].
    ///
    ///
    /// Once the filter is created, this will periodically call [`eth_getFilterChanges`] to get the
    /// newest logs and decode them.
    ///
    ///
    /// **Note:** Compared to [`Self::subscribe`], which is only available on `PubsubClient`s, such
    /// as Websocket, this is a poll-based subscription, as the node does not notify us when a new
    /// matching log is available, instead we have to actively ask for new logs using additional RPC
    /// requests, and this is done on an interval basis.
    ///
    /// [`eth_newFilter`]: https://docs.alchemy.com/alchemy/apis/ethereum/eth-newfilter
    /// [`eth_getFilterChanges`]: https://docs.alchemy.com/alchemy/apis/ethereum/eth-getfilterchanges
    ///
    /// # Example
    // Ignore because `ethers-contract-derive` macros do not work in doctests in `ethers-contract`.
    /// ```ignore
    /// # #[cfg(feature = "abigen")]
    /// # async fn test<M:ethers_providers::Middleware>(contract: ethers_contract::Contract<M>) {
    /// # use ethers_core::types::*;
    /// # use futures_util::stream::StreamExt;
    /// # use ethers_contract::{Contract, EthEvent};
    ///
    /// // The event we want to get
    /// #[derive(Clone, Debug, EthEvent)]
    /// pub struct Approval {
    ///     #[ethevent(indexed)]
    ///     pub token_owner: Address,
    ///     #[ethevent(indexed)]
    ///     pub spender: Address,
    ///     pub tokens: U256,
    /// }
    ///
    /// let ev = contract.event::<Approval>().from_block(1337).to_block(2000);
    /// let mut event_stream = ev.stream().await.unwrap();
    ///
    ///  while let Some(Ok(approval)) = event_stream.next().await {
    ///      let Approval{token_owner,spender,tokens} = approval;
    /// }
    /// # }
    /// ```
    pub async fn stream(
        &self,
    ) -> Result<
        // Wraps the FilterWatcher with a mapping to the event
        EventStream<'_, FilterWatcher<'_, M::Provider, Log>, D, ContractError<M>>,
        ContractError<M>,
    > {
        let filter = self
            .provider
            .borrow()
            .watch(&self.filter)
            .await
            .map_err(ContractError::from_middleware_error)?;
        Ok(EventStream::new(filter.id, filter, Box::new(move |log| Ok(parse_log(log)?))))
    }

    /// As [`Self::stream`], but does not discard [`Log`] metadata.
    pub async fn stream_with_meta(
        &self,
    ) -> Result<
        // Wraps the FilterWatcher with a mapping to the event
        EventStream<'_, FilterWatcher<'_, M::Provider, Log>, (D, LogMeta), ContractError<M>>,
        ContractError<M>,
    > {
        let filter = self
            .provider
            .borrow()
            .watch(&self.filter)
            .await
            .map_err(ContractError::from_middleware_error)?;
        Ok(EventStream::new(
            filter.id,
            filter,
            Box::new(move |log| {
                let meta = LogMeta::from(&log);
                Ok((parse_log(log)?, meta))
            }),
        ))
    }
}

impl<B, M, D> Event<B, M, D>
where
    B: Borrow<M>,
    M: Middleware,
    <M as Middleware>::Provider: PubsubClient,
    D: EthLogDecode,
{
    /// Returns a subscription for the event
    ///
    /// See also [Self::stream()].
    pub async fn subscribe(
        &self,
    ) -> Result<
        // Wraps the SubscriptionStream with a mapping to the event
        EventStream<'_, SubscriptionStream<'_, M::Provider, Log>, D, ContractError<M>>,
        ContractError<M>,
    > {
        let filter = self
            .provider
            .borrow()
            .subscribe_logs(&self.filter)
            .await
            .map_err(ContractError::from_middleware_error)?;
        Ok(EventStream::new(filter.id, filter, Box::new(move |log| Ok(parse_log(log)?))))
    }

    /// As [`Self::subscribe`], but includes event metadata
    pub async fn subscribe_with_meta(
        &self,
    ) -> Result<
        // Wraps the SubscriptionStream with a mapping to the event
        EventStream<'_, SubscriptionStream<'_, M::Provider, Log>, (D, LogMeta), ContractError<M>>,
        ContractError<M>,
    > {
        let filter = self
            .provider
            .borrow()
            .subscribe_logs(&self.filter)
            .await
            .map_err(ContractError::from_middleware_error)?;
        Ok(EventStream::new(
            filter.id,
            filter,
            Box::new(move |log| {
                let meta = LogMeta::from(&log);
                Ok((parse_log(log)?, meta))
            }),
        ))
    }
}

impl<B, M, D> Event<B, M, D>
where
    B: Borrow<M>,
    M: Middleware,
    D: EthLogDecode,
{
    /// Queries the blockchain for the selected filter and returns a vector of matching
    /// event logs
    pub async fn query(&self) -> Result<Vec<D>, ContractError<M>> {
        let logs = self
            .provider
            .borrow()
            .get_logs(&self.filter)
            .await
            .map_err(ContractError::from_middleware_error)?;
        let events = logs
            .into_iter()
            .map(|log| Ok(parse_log(log)?))
            .collect::<Result<Vec<_>, ContractError<M>>>()?;
        Ok(events)
    }

    /// Queries the blockchain for the selected filter and returns a vector of logs
    /// along with their metadata
    pub async fn query_with_meta(&self) -> Result<Vec<(D, LogMeta)>, ContractError<M>> {
        let logs = self
            .provider
            .borrow()
            .get_logs(&self.filter)
            .await
            .map_err(ContractError::from_middleware_error)?;
        let events = logs
            .into_iter()
            .map(|log| {
                let meta = LogMeta::from(&log);
                let event = parse_log(log)?;
                Ok((event, meta))
            })
            .collect::<Result<_, ContractError<M>>>()?;
        Ok(events)
    }
}
