use crate::ContractError;

use ethers_providers::{JsonRpcClient, Provider};

use ethers_core::{
    abi::{Detokenize, Event as AbiEvent, RawLog},
    types::{BlockNumber, Filter, Log, TxHash, ValueOrArray, H256, U64},
};

use futures::stream::{Stream, StreamExt};
use std::marker::PhantomData;

/// Helper for managing the event filter before querying or streaming its logs
#[derive(Debug)]
#[must_use = "event filters do nothing unless you `query` or `stream` them"]
pub struct Event<'a: 'b, 'b, P, D> {
    /// The event filter's state
    pub filter: Filter,
    /// The ABI of the event which is being filtered
    pub event: &'b AbiEvent,
    pub(crate) provider: &'a Provider<P>,
    pub(crate) datatype: PhantomData<D>,
}

// TODO: Improve these functions
impl<P, D: Detokenize> Event<'_, '_, P, D> {
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

impl<'a, 'b, P, D> Event<'a, 'b, P, D>
where
    P: JsonRpcClient,
    D: 'b + Detokenize + Clone,
    'a: 'b,
{
    /// Returns a stream for the event
    pub async fn stream(
        self,
    ) -> Result<impl Stream<Item = Result<D, ContractError>> + 'b, ContractError> {
        let filter = self.provider.watch(&self.filter).await?;
        Ok(filter.stream().map(move |log| self.parse_log(log)))
    }
}

impl<P, D> Event<'_, '_, P, D>
where
    P: JsonRpcClient,
    D: Detokenize + Clone,
{
    /// Queries the blockchain for the selected filter and returns a vector of matching
    /// event logs
    pub async fn query(&self) -> Result<Vec<D>, ContractError> {
        let logs = self.provider.get_logs(&self.filter).await?;
        let events = logs
            .into_iter()
            .map(|log| self.parse_log(log))
            .collect::<Result<Vec<_>, ContractError>>()?;
        Ok(events)
    }

    /// Queries the blockchain for the selected filter and returns a vector of logs
    /// along with their metadata
    pub async fn query_with_meta(&self) -> Result<Vec<(D, LogMeta)>, ContractError> {
        let logs = self.provider.get_logs(&self.filter).await?;
        let events = logs
            .into_iter()
            .map(|log| {
                let meta = LogMeta::from(&log);
                let event = self.parse_log(log)?;
                Ok((event, meta))
            })
            .collect::<Result<_, ContractError>>()?;
        Ok(events)
    }

    fn parse_log(&self, log: Log) -> Result<D, ContractError> {
        // ethabi parses the unindexed and indexed logs together to a
        // vector of tokens
        let tokens = self
            .event
            .parse_log(RawLog {
                topics: log.topics,
                data: log.data.0,
            })?
            .params
            .into_iter()
            .map(|param| param.value)
            .collect::<Vec<_>>();
        // convert the tokens to the requested datatype
        Ok(D::from_tokens(tokens)?)
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
