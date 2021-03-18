use super::{
    base::{encode_function_data, AbiError, BaseContract},
    call::ContractCall,
    event::Event,
};

use ethers_core::{
    abi::{Abi, Detokenize, Error, EventExt, Function, Tokenize},
    types::{Address, Filter, NameOrAddress, Selector, TransactionRequest},
};
use ethers_providers::Middleware;

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

/// A Contract is an abstraction of an executable program on the Ethereum Blockchain.
/// It has code (called byte code) as well as allocated long-term memory
/// (called storage). Every deployed Contract has an address, which is used to connect
/// to it so that it may be sent messages to call its methods.
///
/// A Contract can emit Events, which can be efficiently observed by applications
/// to be notified when a contract has performed specific operation.
///
/// There are two types of methods that can be called on a Contract:
///
/// 1. A Constant method may not add, remove or change any data in the storage,
/// nor log any events, and may only call Constant methods on other contracts.
/// These methods are free (no Ether is required) to call. The result from them
/// may also be returned to the caller. Constant methods are marked as `pure` and
/// `view` in Solidity.
///
/// 2. A Non-Constant method requires a fee (in Ether) to be paid, but may perform
/// any state-changing operation desired, log events, send ether and call Non-Constant
/// methods on other Contracts. These methods cannot return their result to the caller.
/// These methods must be triggered by a transaction, sent by an Externally Owned Account
/// (EOA) either directly or indirectly (i.e. called from another contract), and are
/// required to be mined before the effects are present. Therefore, the duration
/// required for these operations can vary widely, and depend on the transaction
/// gas price, network congestion and miner priority heuristics.
///
/// The Contract API provides simple way to connect to a Contract and call its methods,
/// as functions on a Rust struct, handling all the binary protocol conversion,
/// internal name mangling and topic construction. This allows a Contract object
/// to be used like any standard Rust struct, without having to worry about the
/// low-level details of the Ethereum Virtual Machine or Blockchain.
///
/// The Contract definition (called an Application Binary Interface, or ABI) must
/// be provided to instantiate a contract and the available methods and events will
/// be made available to call by providing their name as a `str` via the [`method`]
/// and [`event`] methods. If non-existing names are given, the function/event call
/// will fail.
///
/// Alternatively, you can _and should_ use the [`abigen`] macro, or the [`Abigen` builder]
/// to generate type-safe bindings to your contracts.
///
/// # Example
///
/// Assuming we already have our contract deployed at `address`, we'll proceed to
/// interact with its methods and retrieve raw logs it has emitted.
///
/// ```no_run
/// use ethers::{
///     abi::Abi,
///     utils::Solc,
///     types::{Address, H256},
///     contract::Contract,
///     providers::{Provider, Http},
///     signers::Wallet,
/// };
/// use std::convert::TryFrom;
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// // this is a fake address used just for this example
/// let address = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse::<Address>()?;
///
/// // (ugly way to write the ABI inline, you can otherwise read it from a file)
/// let abi: Abi = serde_json::from_str(r#"[{"inputs":[{"internalType":"string","name":"value","type":"string"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"author","type":"address"},{"indexed":true,"internalType":"address","name":"oldAuthor","type":"address"},{"indexed":false,"internalType":"string","name":"oldValue","type":"string"},{"indexed":false,"internalType":"string","name":"newValue","type":"string"}],"name":"ValueChanged","type":"event"},{"inputs":[],"name":"getValue","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"lastSender","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"string","name":"value","type":"string"}],"name":"setValue","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#)?;
///
/// // connect to the network
/// let client = Provider::<Http>::try_from("http://localhost:8545").unwrap();
///
/// // create the contract object at the address
/// let contract = Contract::new(address, abi, client);
///
/// // Calling constant methods is done by calling `call()` on the method builder.
/// // (if the function takes no arguments, then you must use `()` as the argument)
/// let init_value: String = contract
///     .method::<_, String>("getValue", ())?
///     .call()
///     .await?;
///
/// // Non-constant methods are executed via the `send()` call on the method builder.
/// let call = contract
///     .method::<_, H256>("setValue", "hi".to_owned())?;
/// let pending_tx = call.send().await?;
///
/// // `await`ing on the pending transaction resolves to a transaction receipt
/// let receipt = pending_tx.confirmations(6).await?;
///
/// # Ok(())
/// # }
/// ```
///
/// # Event Logging
/// Querying structured logs requires you to have defined a struct with the expected
/// datatypes and to have implemented `Detokenize` for it. This boilerplate code
/// is generated for you via the [`abigen`] and [`Abigen` builder] utilities.
///
/// ```no_run
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// use ethers_core::{abi::Abi, types::Address};
/// use ethers_contract::Contract;
/// use ethers_providers::{Provider, Http, Middleware};
/// use ethers_signers::Wallet;
/// use std::convert::TryFrom;
/// use ethers_core::abi::{Detokenize, Token, InvalidOutputType};
/// # // this is a fake address used just for this example
/// # let address = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse::<Address>()?;
/// # let abi: Abi = serde_json::from_str(r#"[]"#)?;
/// # let client = Provider::<Http>::try_from("http://localhost:8545").unwrap();
/// # let contract = Contract::new(address, abi, client);
///
/// #[derive(Clone, Debug)]
/// struct ValueChanged {
///     old_author: Address,
///     new_author: Address,
///     old_value: String,
///     new_value: String,
/// }
///
/// impl Detokenize for ValueChanged {
///     fn from_tokens(tokens: Vec<Token>) -> Result<ValueChanged, InvalidOutputType> {
///         let old_author: Address = tokens[1].clone().into_address().unwrap();
///         let new_author: Address = tokens[1].clone().into_address().unwrap();
///         let old_value = tokens[2].clone().into_string().unwrap();
///         let new_value = tokens[3].clone().into_string().unwrap();
///
///         Ok(Self {
///             old_author,
///             new_author,
///             old_value,
///             new_value,
///         })
///     }
/// }
///
///
/// let logs: Vec<ValueChanged> = contract
///     .event("ValueChanged")?
///     .from_block(0u64)
///     .query()
///     .await?;
///
/// println!("{:?}", logs);
/// # Ok(())
/// # }
///
/// ```
///
/// _Disclaimer: these above docs have been adapted from the corresponding [ethers.js page](https://docs.ethers.io/ethers.js/html/api-contract.html)_
///
/// [`abigen`]: macro.abigen.html
/// [`Abigen` builder]: crate::Abigen
/// [`event`]: method@crate::Contract::event
/// [`method`]: method@crate::Contract::method
#[derive(Debug, Clone)]
pub struct Contract<M> {
    base_contract: BaseContract,
    client: Arc<M>,
    address: Address,
}

impl<M: Middleware> Contract<M> {
    /// Creates a new contract from the provided client, abi and address
    pub fn new(address: Address, abi: impl Into<BaseContract>, client: impl Into<Arc<M>>) -> Self {
        Self {
            base_contract: abi.into(),
            client: client.into(),
            address,
        }
    }

    /// Returns an [`Event`](crate::builders::Event) builder for the provided event name.
    /// TODO(mattsse) keep this but remove event
    pub fn event<D: Detokenize>(&self, name: &str) -> Result<Event<M, D>, Error> {
        // get the event's full name
        let event = self.base_contract.abi.event(name)?;
        Ok(Event {
            provider: &self.client,
            filter: Filter::new()
                .event(&event.abi_signature())
                .address(self.address),
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
    ) -> Result<ContractCall<M, D>, AbiError> {
        // get the function
        let function = self.base_contract.abi.function(name)?;
        self.method_func(function, args)
    }

    /// Returns a transaction builder for the selected function signature. This should be
    /// preferred if there are overloaded functions in your smart contract
    pub fn method_hash<T: Tokenize, D: Detokenize>(
        &self,
        signature: Selector,
        args: T,
    ) -> Result<ContractCall<M, D>, AbiError> {
        let function = self
            .base_contract
            .methods
            .get(&signature)
            .map(|(name, index)| &self.base_contract.abi.functions[name][*index])
            .ok_or_else(|| Error::InvalidName(hex::encode(signature)))?;
        self.method_func(function, args)
    }

    fn method_func<T: Tokenize, D: Detokenize>(
        &self,
        function: &Function,
        args: T,
    ) -> Result<ContractCall<M, D>, AbiError> {
        let data = encode_function_data(function, args)?;

        // create the tx object
        let tx = TransactionRequest {
            to: Some(NameOrAddress::Address(self.address)),
            data: Some(data),
            ..Default::default()
        };

        Ok(ContractCall {
            tx,
            client: Arc::clone(&self.client), // cheap clone behind the Arc
            block: None,
            function: function.to_owned(),
            datatype: PhantomData,
        })
    }

    /// Returns a new contract instance at `address`.
    ///
    /// Clones `self` internally
    pub fn at<T: Into<Address>>(&self, address: T) -> Self
    where
        M: Clone,
    {
        let mut this = self.clone();
        this.address = address.into();
        this
    }

    /// Returns a new contract instance using the provided client
    ///
    /// Clones `self` internally
    pub fn connect(&self, client: Arc<M>) -> Self
    where
        M: Clone,
    {
        let mut this = self.clone();
        this.client = client;
        this
    }

    /// Returns the contract's address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Returns a reference to the contract's ABI
    pub fn abi(&self) -> &Abi {
        &self.base_contract.abi
    }

    /// Returns a reference to the contract's client
    pub fn client(&self) -> &M {
        &self.client
    }
}

impl<M: Middleware> std::ops::Deref for Contract<M> {
    type Target = BaseContract;
    fn deref(&self) -> &Self::Target {
        &self.base_contract
    }
}
