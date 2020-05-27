use crate::{ContractCall, Event};

use ethers_abi::{Abi, Detokenize, Error, EventExt, Function, FunctionExt, Tokenize};
use ethers_providers::{networks::Network, JsonRpcClient};
use ethers_signers::{Client, Signer};
use ethers_types::{Address, Filter, Selector, TransactionRequest};

use rustc_hex::ToHex;
use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

/// Represents a contract instance at an address. Provides methods for
/// contract interaction.
// TODO: Should we separate the lifetimes for the two references?
// https://stackoverflow.com/a/29862184
#[derive(Debug, Clone)]
pub struct Contract<'a, P, N, S> {
    client: &'a Client<'a, P, N, S>,
    abi: &'a Abi,
    address: Address,

    /// A mapping from method signature to a name-index pair for accessing
    /// functions in the contract ABI. This is used to avoid allocation when
    /// searching for matching functions by signature.
    // Adapted from: https://github.com/gnosis/ethcontract-rs/blob/master/src/contract.rs
    methods: HashMap<Selector, (String, usize)>,
}

impl<'a, P: JsonRpcClient, N: Network, S: Signer> Contract<'a, P, N, S> {
    /// Creates a new contract from the provided client, abi and address
    pub fn new(client: &'a Client<'a, P, N, S>, abi: &'a Abi, address: Address) -> Self {
        let methods = create_mapping(&abi.functions, |function| function.selector());

        Self {
            client,
            abi,
            address,
            methods,
        }
    }

    /// Returns an `Event` builder for the provided event name. If there are
    /// multiple functions with the same name due to overloading, consider using
    /// the `method_hash` method instead, since this will use the first match.
    pub fn event<'b, D: Detokenize>(&'a self, name: &str) -> Result<Event<'a, 'b, P, N, D>, Error>
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
    ) -> Result<ContractCall<'a, P, N, S, D>, Error> {
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
    ) -> Result<ContractCall<'a, P, N, S, D>, Error> {
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
    ) -> Result<ContractCall<'a, P, N, S, D>, Error> {
        // create the calldata
        let data = function.encode_input(&args.into_tokens())?;

        // create the tx object
        let tx = TransactionRequest {
            to: Some(self.address),
            data: Some(data.into()),
            ..Default::default()
        };

        Ok(ContractCall {
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
