use crate::ContractError;

use ethers_providers::{networks::Network, JsonRpcClient, Provider};

use ethers_types::{
    abi::{Detokenize, Event as AbiEvent, RawLog},
    BlockNumber, Filter, ValueOrArray, H256,
};

use std::marker::PhantomData;

pub struct Event<'a, 'b, P, N, D> {
    pub filter: Filter,
    pub(crate) provider: &'a Provider<P, N>,
    pub(crate) event: &'b AbiEvent,
    pub(crate) datatype: PhantomData<D>,
}

// TODO: Improve these functions
impl<'a, 'b, P, N, D: Detokenize> Event<'a, 'b, P, N, D> {
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
        self.filter.topics.push(topic.into());
        self
    }

    pub fn topics(mut self, topics: &[ValueOrArray<H256>]) -> Self {
        self.filter.topics.extend_from_slice(topics);
        self
    }
}

// TODO: Can we get rid of the static?
impl<'a, 'b, P: JsonRpcClient, N: Network, D: Detokenize> Event<'a, 'b, P, N, D>
where
    P::Error: 'static,
{
    /// Queries the blockchain for the selected filter and returns a vector of matching
    /// event logs
    pub async fn query(self) -> Result<Vec<D>, ContractError<P>> {
        // get the logs
        let logs = self
            .provider
            .get_logs(&self.filter)
            .await
            .map_err(ContractError::CallError)?;

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
                Ok::<_, ContractError<P>>(D::from_tokens(tokens)?)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(events)
    }

    // TODO: Add filter watchers
}
