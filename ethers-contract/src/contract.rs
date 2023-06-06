use crate::{
    base::{encode_function_data, AbiError, BaseContract},
    call::FunctionCall,
    event::{EthEvent, Event},
};
use ethers_core::{
    abi::{Abi, Detokenize, Error, EventExt, Function, Tokenize},
    types::{Address, Filter, Selector, ValueOrArray},
};
use ethers_providers::Middleware;
use std::{borrow::Borrow, fmt::Debug, marker::PhantomData, sync::Arc};

#[cfg(not(feature = "legacy"))]
use ethers_core::types::Eip1559TransactionRequest;
#[cfg(feature = "legacy")]
use ethers_core::types::TransactionRequest;

/// `Contract` is a [`ContractInstance`] object with an `Arc` middleware.
/// This type alias exists to preserve backwards compatibility with
/// less-abstract Contracts.
///
/// For full usage docs, see [`ContractInstance`].
pub type Contract<M> = ContractInstance<std::sync::Arc<M>, M>;

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
/// use ethers_core::{
///     abi::Abi,
///     types::{Address, H256},
/// };
/// use ethers_contract::Contract;
/// use ethers_providers::{Provider, Http};
/// use std::{convert::TryFrom, sync::Arc};
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
/// let contract = Contract::new(address, abi, Arc::new(client));
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
///
/// Querying structured logs requires you to have defined a struct with the expected
/// datatypes and to have implemented `Detokenize` for it. This boilerplate code
/// is generated for you via the [`abigen`] and [`Abigen` builder] utilities.
//
// Ignore because `ethers-contract-derive` macros do not work in doctests in `ethers-contract`.
/// ```ignore
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// use ethers_core::{abi::Abi, types::Address};
/// use ethers_contract::{Contract, EthEvent};
/// use ethers_providers::{Provider, Http, Middleware};
/// use std::{convert::TryFrom, sync::Arc};
/// use ethers_core::abi::{Detokenize, Token, InvalidOutputType};
/// # // this is a fake address used just for this example
/// # let address = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse::<Address>()?;
/// # let abi: Abi = serde_json::from_str(r#"[]"#)?;
/// # let client = Provider::<Http>::try_from("http://localhost:8545").unwrap();
/// # let contract = Contract::new(address, abi, Arc::new(client));
///
/// #[derive(Clone, Debug, EthEvent)]
/// struct ValueChanged {
///     old_author: Address,
///     new_author: Address,
///     old_value: String,
///     new_value: String,
/// }
///
/// let logs: Vec<ValueChanged> = contract
///     .event()
///     .from_block(0u64)
///     .query()
///     .await?;
///
/// println!("{:?}", logs);
/// # Ok(())
/// # }
/// ```
///
/// _Disclaimer: these above docs have been adapted from the corresponding [ethers.js page](https://docs.ethers.io/ethers.js/html/api-contract.html)_
///
/// # Usage Note
///
/// `ContractInternal` accepts any client that implements `B: Borrow<M>` where
/// `M :Middleware`. Previous `Contract` versions used only arcs, and relied
/// heavily on [`Arc`]. Due to constraints on the [`FunctionCall`] type,
/// calling contracts requires a `B: Borrow<M> + Clone`. This is fine for most
/// middlware. However, when `B` is an owned middleware that is not Clone, we
/// cannot issue contract calls. Some notable exceptions:
///
/// - `NonceManagerMiddleware`
/// - `SignerMiddleware` (when using a non-Clone Signer)
///
/// When using non-Clone middlewares, instead of instantiating a contract that
/// OWNS the middlware, pass the contract a REFERENCE to the middleware. This
/// will fix the trait bounds issue (as `&M` is always `Clone`).
///
/// We expect to fix this fully in a future version
///
/// [`abigen`]: macro.abigen.html
/// [`Abigen` builder]: struct.Abigen.html
/// [`event`]: method@crate::ContractInstance::event
/// [`method`]: method@crate::ContractInstance::method
#[derive(Debug)]
pub struct ContractInstance<B, M> {
    address: Address,
    base_contract: BaseContract,
    client: B,
    _m: PhantomData<M>,
}

impl<B, M> std::ops::Deref for ContractInstance<B, M>
where
    B: Borrow<M>,
{
    type Target = BaseContract;

    fn deref(&self) -> &Self::Target {
        &self.base_contract
    }
}

impl<B, M> Clone for ContractInstance<B, M>
where
    B: Clone + Borrow<M>,
{
    fn clone(&self) -> Self {
        ContractInstance {
            base_contract: self.base_contract.clone(),
            client: self.client.clone(),
            address: self.address,
            _m: self._m,
        }
    }
}

impl<B, M> ContractInstance<B, M>
where
    B: Borrow<M>,
{
    /// Returns the contract's address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Returns a reference to the contract's ABI.
    pub fn abi(&self) -> &Abi {
        &self.base_contract.abi
    }

    /// Returns a pointer to the contract's client.
    pub fn client(&self) -> B
    where
        B: Clone,
    {
        self.client.clone()
    }

    /// Returns a reference to the contract's client.
    pub fn client_ref(&self) -> &M {
        self.client.borrow()
    }
}

impl<B, M> ContractInstance<B, M>
where
    B: Borrow<M>,
    M: Middleware,
{
    /// Returns an [`Event`](crate::builders::Event) builder for the provided event.
    /// This function operates in a static context, then it does not require a `self`
    /// to reference to instantiate an [`Event`](crate::builders::Event) builder.
    pub fn event_of_type<D: EthEvent>(client: B) -> Event<B, M, D> {
        Event {
            provider: client,
            filter: Filter::new().event(&D::abi_signature()),
            datatype: PhantomData,
            _m: PhantomData,
        }
    }
}

impl<B, M> ContractInstance<B, M>
where
    B: Borrow<M>,
    M: Middleware,
{
    /// Creates a new contract from the provided client, abi and address
    pub fn new(address: impl Into<Address>, abi: impl Into<BaseContract>, client: B) -> Self {
        Self { base_contract: abi.into(), client, address: address.into(), _m: PhantomData }
    }

    /// Returns a new contract instance using the provided client
    ///
    /// Clones `self` internally
    #[must_use]
    pub fn connect<N>(&self, client: Arc<N>) -> ContractInstance<Arc<N>, N>
    where
        N: Middleware,
    {
        ContractInstance {
            base_contract: self.base_contract.clone(),
            client,
            address: self.address,
            _m: PhantomData,
        }
    }

    /// Returns a new contract instance using the provided client
    ///
    /// Clones `self` internally
    #[must_use]
    pub fn connect_with<C, N>(&self, client: C) -> ContractInstance<C, N>
    where
        C: Borrow<N>,
    {
        ContractInstance {
            base_contract: self.base_contract.clone(),
            client,
            address: self.address,
            _m: PhantomData,
        }
    }
}

impl<B, M> ContractInstance<B, M>
where
    B: Clone + Borrow<M>,
    M: Middleware,
{
    /// Returns an [`Event`](crate::builders::Event) builder with the provided filter.
    pub fn event_with_filter<D>(&self, filter: Filter) -> Event<B, M, D> {
        Event {
            provider: self.client.clone(),
            filter: filter.address(ValueOrArray::Value(self.address)),
            datatype: PhantomData,
            _m: PhantomData,
        }
    }

    /// Returns an [`Event`](crate::builders::Event) builder for the provided event.
    pub fn event<D: EthEvent>(&self) -> Event<B, M, D> {
        D::new(Filter::new(), self.client.clone())
    }

    /// Returns an [`Event`](crate::builders::Event) builder with the provided name.
    pub fn event_for_name<D>(&self, name: &str) -> Result<Event<B, M, D>, Error> {
        // get the event's full name
        let event = self.base_contract.abi.event(name)?;
        Ok(self.event_with_filter(Filter::new().event(&event.abi_signature())))
    }

    fn method_func<T: Tokenize, D: Detokenize>(
        &self,
        function: &Function,
        args: T,
    ) -> Result<FunctionCall<B, M, D>, AbiError> {
        let data = encode_function_data(function, args)?;

        #[cfg(feature = "legacy")]
        let tx = TransactionRequest {
            to: Some(self.address.into()),
            data: Some(data),
            ..Default::default()
        };
        #[cfg(not(feature = "legacy"))]
        let tx = Eip1559TransactionRequest {
            to: Some(self.address.into()),
            data: Some(data),
            ..Default::default()
        };

        let tx = tx.into();

        Ok(FunctionCall {
            tx,
            client: self.client.clone(),
            block: None,
            function: function.to_owned(),
            datatype: PhantomData,
            _m: self._m,
        })
    }

    /// Returns a transaction builder for the selected function signature. This should be
    /// preferred if there are overloaded functions in your smart contract
    pub fn method_hash<T: Tokenize, D: Detokenize>(
        &self,
        signature: Selector,
        args: T,
    ) -> Result<FunctionCall<B, M, D>, AbiError> {
        let function = self
            .base_contract
            .methods
            .get(&signature)
            .map(|(name, index)| &self.base_contract.abi.functions[name][*index])
            .ok_or_else(|| Error::InvalidName(hex::encode(signature)))?;
        self.method_func(function, args)
    }

    /// Returns a transaction builder for the provided function name. If there are
    /// multiple functions with the same name due to overloading, consider using
    /// the `method_hash` method instead, since this will use the first match.
    pub fn method<T: Tokenize, D: Detokenize>(
        &self,
        name: &str,
        args: T,
    ) -> Result<FunctionCall<B, M, D>, AbiError> {
        // get the function
        let function = self.base_contract.abi.function(name)?;
        self.method_func(function, args)
    }

    /// Returns a new contract instance at `address`.
    ///
    /// Clones `self` internally
    #[must_use]
    pub fn at<T: Into<Address>>(&self, address: T) -> Self {
        let mut this = self.clone();
        this.address = address.into();
        this
    }
}
