use ethers_abi::{
    Abi, Detokenize, Error, Event as AbiEvent, EventExt, Function, FunctionExt, RawLog, Tokenize,
};
use ethers_providers::{JsonRpcClient, Provider};
use ethers_signers::{Client, Signer};
use ethers_types::{
    Address, BlockNumber, Filter, Selector, TransactionRequest, ValueOrArray, H256, U256,
};

use rustc_hex::ToHex;
use serde::Deserialize;
use std::{collections::HashMap, fmt::Debug, hash::Hash};

/// Represents a contract instance at an address. Provides methods for
/// contract interaction.
#[derive(Debug, Clone)]
pub struct Contract<'a, S, P> {
    client: &'a Client<'a, S, P>,
    abi: Abi,
    address: Address,

    /// A mapping from method signature to a name-index pair for accessing
    /// functions in the contract ABI. This is used to avoid allocation when
    /// searching for matching functions by signature.
    // Adapted from: https://github.com/gnosis/ethcontract-rs/blob/master/src/contract.rs
    methods: HashMap<Selector, (String, usize)>,
}

impl<'a, S: Signer, P: JsonRpcClient> Contract<'a, S, P> {
    /// Creates a new contract from the provided client, abi and address
    pub fn new(client: &'a Client<'a, S, P>, abi: Abi, address: Address) -> Self {
        let methods = create_mapping(&abi.functions, |function| function.selector());

        Self {
            client,
            abi,
            address,
            methods,
        }
    }

    /// Returns a transaction builder for the provided function name. If there are
    /// multiple functions with the same name due to overloading, consider using
    /// the `method_hash` method instead, since this will use the first match.
    pub fn event<'b>(&'a self, name: &str) -> Result<Event<'a, 'b, P>, Error>
    where
        'a: 'b,
    {
        // get the event's full name
        let event = self.abi.event(name)?;
        Ok(Event {
            provider: &self.client.provider(),
            filter: Filter::new().event(&event.abi_signature()),
            event: &event,
        })
    }

    /// Returns a transaction builder for the provided function name. If there are
    /// multiple functions with the same name due to overloading, consider using
    /// the `method_hash` method instead, since this will use the first match.
    pub fn method<T: Tokenize>(&self, name: &str, args: T) -> Result<Sender<'a, S, P>, Error> {
        // get the function
        let function = self.abi.function(name)?;
        self.method_func(function, args)
    }

    /// Returns a transaction builder for the selected function signature. This should be
    /// preferred if there are overloaded functions in your smart contract
    pub fn method_hash<T: Tokenize>(
        &self,
        signature: Selector,
        args: T,
    ) -> Result<Sender<'a, S, P>, Error> {
        let function = self
            .methods
            .get(&signature)
            .map(|(name, index)| &self.abi.functions[name][*index])
            .ok_or_else(|| Error::InvalidName(signature.to_hex::<String>()))?;
        self.method_func(function, args)
    }

    fn method_func<T: Tokenize>(
        &self,
        function: &Function,
        args: T,
    ) -> Result<Sender<'a, S, P>, Error> {
        // create the calldata
        let data = function.encode_input(&args.into_tokens())?;

        // create the tx object
        let tx = TransactionRequest {
            to: Some(self.address),
            data: Some(data.into()),
            ..Default::default()
        };

        Ok(Sender {
            tx,
            client: self.client,
            block: None,
        })
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn abi(&self) -> &Abi {
        &self.abi
    }
}

pub struct Sender<'a, S, P> {
    tx: TransactionRequest,
    client: &'a Client<'a, S, P>,
    block: Option<BlockNumber>,
}

impl<'a, S, P> Sender<'a, S, P> {
    /// Sets the `from` field in the transaction to the provided value
    pub fn from<T: Into<Address>>(mut self, from: T) -> Self {
        self.tx.from = Some(from.into());
        self
    }

    /// Sets the `gas` field in the transaction to the provided value
    pub fn gas<T: Into<U256>>(mut self, gas: T) -> Self {
        self.tx.gas = Some(gas.into());
        self
    }

    /// Sets the `gas_price` field in the transaction to the provided value
    pub fn gas_price<T: Into<U256>>(mut self, gas_price: T) -> Self {
        self.tx.gas_price = Some(gas_price.into());
        self
    }

    /// Sets the `value` field in the transaction to the provided value
    pub fn value<T: Into<U256>>(mut self, value: T) -> Self {
        self.tx.value = Some(value.into());
        self
    }
}

impl<'a, S: Signer, P: JsonRpcClient> Sender<'a, S, P> {
    pub async fn call<T: for<'b> Deserialize<'b>>(self) -> Result<T, P::Error> {
        self.client.call(self.tx).await
    }

    pub async fn send(self) -> Result<H256, P::Error> {
        self.client.send_transaction(self.tx, self.block).await
    }
}

pub struct Event<'a, 'b, P> {
    filter: Filter,
    provider: &'a Provider<P>,
    event: &'b AbiEvent,
}

// copy of the builder pattern from Filter
impl<'a, 'b, P> Event<'a, 'b, P> {
    pub fn from_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.filter.from_block = Some(block.into());
        self
    }

    pub fn to_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.filter.to_block = Some(block.into());
        self
    }

    pub fn topic<T: Into<ValueOrArray<H256>>>(mut self, topic: T) -> Self {
        self.filter.topics.push(topic.into());
        self
    }

    pub fn topics(mut self, topics: &[ValueOrArray<H256>]) -> Self {
        self.filter.topics.extend_from_slice(topics);
        self
    }
}

impl<'a, 'b, P: JsonRpcClient> Event<'a, 'b, P> {
    pub async fn query<T: Detokenize>(self) -> Result<Vec<T>, P::Error> {
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
                    })
                    .unwrap() // TODO: remove
                    .params
                    .into_iter()
                    .map(|param| param.value)
                    .collect::<Vec<_>>();

                // convert the tokens to the requested datatype
                T::from_tokens(tokens).unwrap()
            })
            .collect::<Vec<T>>();

        Ok(events)
    }
}

/// Utility function for creating a mapping between a unique signature and a
/// name-index pair for accessing contract ABI items.
fn create_mapping<T, S, F>(
    elements: &HashMap<String, Vec<T>>,
    signature: F,
) -> HashMap<S, (String, usize)>
where
    S: Hash + Eq,
    F: Fn(&T) -> S,
{
    let signature = &signature;
    elements
        .iter()
        .flat_map(|(name, sub_elements)| {
            sub_elements
                .iter()
                .enumerate()
                .map(move |(index, element)| (signature(element), (name.to_owned(), index)))
        })
        .collect()
}
