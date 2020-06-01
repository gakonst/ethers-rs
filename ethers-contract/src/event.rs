use crate::ContractError;

use ethers_providers::{JsonRpcClient, Provider};

use ethers_core::{
    abi::{Detokenize, Event as AbiEvent, RawLog},
    types::{BlockNumber, Filter, ValueOrArray, H256},
};

use std::{collections::HashMap, marker::PhantomData};

pub struct Event<'a, 'b, P, D> {
    pub filter: Filter,
    pub(crate) provider: &'a Provider<P>,
    pub(crate) event: &'b AbiEvent,
    pub(crate) datatype: PhantomData<D>,
}

// TODO: Improve these functions
impl<'a, 'b, P, D: Detokenize> Event<'a, 'b, P, D> {
    #[allow(clippy::wrong_self_convention)]
    pub fn from_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.filter.from_block = Some(block.into());
        self
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.filter.to_block = Some(block.into());
        self
    }

    pub fn topic0<T: Into<ValueOrArray<H256>>>(mut self, topic: T) -> Self {
        self.filter.topics[0] = Some(topic.into());
        self
    }

    pub fn topic1<T: Into<ValueOrArray<H256>>>(mut self, topic: T) -> Self {
        self.filter.topics[1] = Some(topic.into());
        self
    }
}

impl<'a, 'b, P, D> Event<'a, 'b, P, D>
where
    P: JsonRpcClient,
    D: Detokenize + Clone,
{
    /// Queries the blockchain for the selected filter and returns a vector of matching
    /// event logs
    pub async fn query(self) -> Result<Vec<D>, ContractError> {
        Ok(self.query_with_hashes().await?.values().cloned().collect())
    }

    /// Queries the blockchain for the selected filter and returns a vector of matching
    /// event logs
    pub async fn query_with_hashes(self) -> Result<HashMap<H256, D>, ContractError> {
        // get the logs
        let logs = self.provider.get_logs(&self.filter).await?;

        let events = logs
            .into_iter()
            .map(|log| {
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
                Ok::<_, ContractError>((
                    log.transaction_hash.expect("should have tx hash"),
                    D::from_tokens(tokens)?,
                ))
            })
            .collect::<Result<HashMap<H256, D>, _>>()?;

        Ok(events)
    }

    // TODO: Add filter watchers
}
