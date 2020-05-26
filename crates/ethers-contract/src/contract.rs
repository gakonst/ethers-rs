use ethers_abi::{
    Abi, Detokenize, Error, Event as AbiEvent, EventExt, Function, FunctionExt, RawLog, Tokenize,
};
use ethers_providers::{JsonRpcClient, Provider};
use ethers_signers::{Client, Signer};
use ethers_types::{
    Address, BlockNumber, Filter, Selector, TransactionRequest, ValueOrArray, H256, U256,
};

use rustc_hex::ToHex;
use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

use thiserror::Error as ThisError;

/// Represents a contract instance at an address. Provides methods for
/// contract interaction.
#[derive(Debug, Clone)]
pub struct Contract<'a, S, P> {
    client: &'a Client<'a, S, P>,
    abi: &'a Abi,
    address: Address,

    /// A mapping from method signature to a name-index pair for accessing
    /// functions in the contract ABI. This is used to avoid allocation when
    /// searching for matching functions by signature.
    // Adapted from: https://github.com/gnosis/ethcontract-rs/blob/master/src/contract.rs
    methods: HashMap<Selector, (String, usize)>,
}

impl<'a, S: Signer, P: JsonRpcClient> Contract<'a, S, P> {
    /// Creates a new contract from the provided client, abi and address
    pub fn new(client: &'a Client<'a, S, P>, abi: &'a Abi, address: Address) -> Self {
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
    pub fn event<'b, D: Detokenize>(&'a self, name: &str) -> Result<Event<'a, 'b, P, D>, Error>
    where
        'a: 'b,
    {
        // get the event's full name
        let event = self.abi.event(name)?;
        Ok(Event {
            provider: &self.client.provider(),
            filter: Filter::new().event(&event.abi_signature()),
            event: &event,
            datatype: PhantomData,
        })
    }

    /// Returns a transaction builder for the provided function name. If there are
    /// multiple functions with the same name due to overloading, consider using
    /// the `method_hash` method instead, since this will use the first match.
    pub fn method<T: Tokenize, D: Detokenize>(
        &self,
        name: &str,
        args: T,
    ) -> Result<Sender<'a, S, P, D>, Error> {
        // get the function
        let function = self.abi.function(name)?;
        self.method_func(function, args)
    }

    /// Returns a transaction builder for the selected function signature. This should be
    /// preferred if there are overloaded functions in your smart contract
    pub fn method_hash<T: Tokenize, D: Detokenize>(
        &self,
        signature: Selector,
        args: T,
    ) -> Result<Sender<'a, S, P, D>, Error> {
        let function = self
            .methods
            .get(&signature)
            .map(|(name, index)| &self.abi.functions[name][*index])
            .ok_or_else(|| Error::InvalidName(signature.to_hex::<String>()))?;
        self.method_func(function, args)
    }

    fn method_func<T: Tokenize, D: Detokenize>(
        &self,
        function: &Function,
        args: T,
    ) -> Result<Sender<'a, S, P, D>, Error> {
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
            function: function.to_owned(),
            datatype: PhantomData,
        })
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn abi(&self) -> &Abi {
        &self.abi
    }
}

pub struct Sender<'a, S, P, D> {
    tx: TransactionRequest,
    function: Function,
    client: &'a Client<'a, S, P>,
    block: Option<BlockNumber>,
    datatype: PhantomData<D>,
}

impl<'a, S, P, D: Detokenize> Sender<'a, S, P, D> {
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

#[derive(ThisError, Debug)]
// TODO: Can we get rid of this static?
pub enum ContractError<P: JsonRpcClient>
where
    P::Error: 'static,
{
    #[error(transparent)]
    DecodingError(#[from] ethers_abi::Error),
    #[error(transparent)]
    DetokenizationError(#[from] ethers_abi::InvalidOutputType),
    #[error(transparent)]
    CallError(P::Error),
}

impl<'a, S: Signer, P: JsonRpcClient, D: Detokenize> Sender<'a, S, P, D>
where
    P::Error: 'static,
{
    pub async fn call(self) -> Result<D, ContractError<P>> {
        let bytes = self
            .client
            .call(self.tx, self.block)
            .await
            .map_err(ContractError::CallError)?;

        let tokens = self.function.decode_output(&bytes.0)?;

        let data = D::from_tokens(tokens)?;

        Ok(data)
    }

    pub async fn send(self) -> Result<H256, P::Error> {
        self.client.send_transaction(self.tx, self.block).await
    }
}

pub struct Event<'a, 'b, P, D> {
    pub filter: Filter,
    provider: &'a Provider<P>,
    event: &'b AbiEvent,
    datatype: PhantomData<D>,
}

impl<'a, 'b, P, D: Detokenize> Event<'a, 'b, P, D> {
    pub fn from_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.filter.from_block = Some(block.into());
        self
    }

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
impl<'a, 'b, P: JsonRpcClient, D: Detokenize> Event<'a, 'b, P, D>
where
    P::Error: 'static,
{
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
