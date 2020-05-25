use crate::{
    abi::{Abi, Function, FunctionExt},
    providers::JsonRpcClient,
    signers::{Client, Signer},
    types::{Address, BlockNumber, Selector, TransactionRequest, H256, U256},
};

use rustc_hex::ToHex;
use serde::Deserialize;
use std::{collections::HashMap, hash::Hash};

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
    methods: HashMap<Selector, (String, usize)>,

    /// A mapping from event signature to a name-index pair for resolving
    /// events in the contract ABI.
    events: HashMap<H256, (String, usize)>,
}

impl<'a, S, P> Contract<'a, S, P> {
    /// Creates a new contract from the provided client, abi and address
    pub fn new(client: &'a Client<'a, S, P>, abi: Abi, address: Address) -> Self {
        let methods = create_mapping(&abi.functions, |function| function.selector());
        let events = create_mapping(&abi.events, |event| event.signature());

        Self {
            client,
            abi,
            address,
            methods,
            events,
        }
    }

    /// Returns a transaction builder for the provided function name. If there are
    /// multiple functions with the same name due to overloading, consider using
    /// the `method_hash` method instead, since this will use the first match.
    pub fn method(
        &self,
        name: &str,
        args: &[ethabi::Token],
    ) -> Result<Sender<'a, S, P>, ethabi::Error> {
        // get the function
        let function = self.abi.function(name)?;
        self.method_func(function, args)
    }

    /// Returns a transaction builder for the selected function signature. This should be
    /// preferred if there are overloaded functions in your smart contract
    pub fn method_hash(
        &self,
        signature: Selector,
        args: &[ethabi::Token],
    ) -> Result<Sender<'a, S, P>, ethabi::Error> {
        let function = self
            .methods
            .get(&signature)
            .map(|(name, index)| &self.abi.functions[name][*index])
            .ok_or_else(|| ethabi::Error::InvalidName(signature.to_hex::<String>()))?;
        self.method_func(function, args)
    }

    fn method_func(
        &self,
        function: &Function,
        args: &[ethabi::Token],
    ) -> Result<Sender<'a, S, P>, ethabi::Error> {
        // create the calldata
        let data = function.encode_input(args)?;

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

    // call events
    // deploy
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
