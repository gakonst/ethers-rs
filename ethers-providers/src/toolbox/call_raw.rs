//! Overrides for the `eth_call` rpc method

use crate::{utils::PinBoxFut, JsonRpcClient, Provider, ProviderError};
use ethers_core::{
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockId, BlockNumber, Bytes, H256, U256,
        U64,
    },
    utils,
};
use pin_project::pin_project;
use serde::{ser::SerializeTuple, Deserialize, Serialize};
use std::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub use spoof::{balance, code, nonce, state, storage};

/// Provides methods for overriding parameters to the `eth_call` rpc method
pub trait RawCall<'a> {
    /// Sets the block number to execute against
    fn block(self, id: BlockId) -> Self;
    /// Sets the [state override set](https://geth.ethereum.org/docs/rpc/ns-eth#3-object---state-override-set).
    /// Note that not all client implementations will support this as a parameter.
    fn state(self, state: &'a spoof::State) -> Self;

    /// Maps a closure `f` over the result of `.await`ing this call
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map::new(self, f)
    }
}

/// A builder which implements [`RawCall`] methods for overriding `eth_call` parameters.
///
/// `CallBuilder` also implements [`std::future::Future`], so `.await`ing a `CallBuilder` will
/// resolve to the result of executing the `eth_call`.
#[must_use = "call_raw::CallBuilder does nothing unless you `.await` or poll it"]
pub enum CallBuilder<'a, P> {
    /// The primary builder which exposes [`RawCall`] methods.
    Build(Caller<'a, P>),
    /// Used by the [`std::future::Future`] implementation. You are unlikely to encounter this
    /// variant unless you are constructing your own [`RawCall`] wrapper type.
    Wait(PinBoxFut<'a, Bytes>),
}

impl<P: fmt::Debug> fmt::Debug for CallBuilder<'_, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(call) => f.debug_tuple("Build").field(call).finish(),
            Self::Wait(_) => f.debug_tuple("Wait").field(&"< Future >").finish(),
        }
    }
}

impl<'a, P> CallBuilder<'a, P> {
    /// Instantiate a new call builder based on `tx`
    pub fn new(provider: &'a Provider<P>, tx: &'a TypedTransaction) -> Self {
        Self::Build(Caller::new(provider, tx))
    }

    /// Applies a closure `f` to a `CallBuilder::Build`. Does nothing for `CallBuilder::Wait`.
    pub fn map_input<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut Caller<'a, P>),
    {
        match self {
            Self::Build(mut call) => {
                f(&mut call);
                Self::Build(call)
            }
            wait => wait,
        }
    }

    /// Returns the inner `Caller` from a `CallBuilder::Build`. Panics if the `CallBuilder` future
    /// has already been polled.
    pub fn unwrap(self) -> Caller<'a, P> {
        match self {
            Self::Build(b) => b,
            _ => panic!("CallBuilder::unwrap on a Wait value"),
        }
    }
}

impl<'a, P> RawCall<'a> for CallBuilder<'a, P> {
    /// Sets the block number to execute against
    fn block(self, id: BlockId) -> Self {
        self.map_input(|mut call| call.input.block = Some(id))
    }
    /// Sets the [state override set](https://geth.ethereum.org/docs/rpc/ns-eth#3-object---state-override-set).
    /// Note that not all client implementations will support this as a parameter.
    fn state(self, state: &'a spoof::State) -> Self {
        self.map_input(|mut call| call.input.state = Some(state))
    }
}

impl<'a, P: JsonRpcClient> Future for CallBuilder<'a, P> {
    type Output = Result<Bytes, ProviderError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let pin = self.get_mut();
        loop {
            match pin {
                CallBuilder::Build(ref call) => {
                    let fut = Box::pin(call.execute());
                    *pin = CallBuilder::Wait(fut);
                }
                CallBuilder::Wait(ref mut fut) => return fut.as_mut().poll(cx),
            }
        }
    }
}

/// Holds the inputs to the `eth_call` rpc method along with the rpc provider.
/// This type is constructed by [`CallBuilder::new`].
#[derive(Clone, Debug)]
pub struct Caller<'a, P> {
    provider: &'a Provider<P>,
    input: CallInput<'a>,
}

impl<'a, P> Caller<'a, P> {
    /// Instantiate a new `Caller` based on `tx`
    pub fn new(provider: &'a Provider<P>, tx: &'a TypedTransaction) -> Self {
        Self { provider, input: CallInput::new(tx) }
    }
}
impl<'a, P: JsonRpcClient> Caller<'a, P> {
    /// Executes an `eth_call` rpc request with the overriden parameters. Returns a future that
    /// resolves to the result of the request.
    fn execute(&self) -> impl Future<Output = Result<Bytes, ProviderError>> + 'a {
        self.provider.request("eth_call", utils::serialize(&self.input))
    }
}

/// The input parameters to the `eth_call` rpc method
#[derive(Clone, Debug, PartialEq, Eq)]
struct CallInput<'a> {
    tx: &'a TypedTransaction,
    block: Option<BlockId>,
    state: Option<&'a spoof::State>,
}

impl<'a> CallInput<'a> {
    fn new(tx: &'a TypedTransaction) -> Self {
        Self { tx, block: None, state: None }
    }
}

impl<'a> Serialize for CallInput<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let len = 2 + self.state.is_some() as usize;

        let mut tup = serializer.serialize_tuple(len)?;
        tup.serialize_element(self.tx)?;

        let block = self.block.unwrap_or_else(|| BlockNumber::Latest.into());
        tup.serialize_element(&block)?;

        if let Some(state) = self.state {
            tup.serialize_element(state)?;
        }
        tup.end()
    }
}

/// An implementer of [`RawCall`] that maps a function `f` over the output of the inner future.
///
/// This struct is created by the [`map`] method on [`RawCall`].
///
/// [`map`]: RawCall::map
#[must_use = "call_raw::Map does nothing unless you `.await` or poll it"]
#[derive(Clone)]
#[pin_project]
pub struct Map<T, F> {
    #[pin]
    inner: T,
    f: F,
}

impl<T: fmt::Debug, F> fmt::Debug for Map<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map").field("inner", &self.inner).finish()
    }
}

impl<T, F> Map<T, F> {
    /// Instantiate a new map
    pub fn new(inner: T, f: F) -> Self {
        Self { inner, f }
    }
}

impl<'a, T, F> RawCall<'a> for Map<T, F>
where
    T: RawCall<'a>,
{
    /// Sets the block number to execute against
    fn block(self, id: BlockId) -> Self {
        Self { inner: self.inner.block(id), f: self.f }
    }

    /// Sets the [state override set](https://geth.ethereum.org/docs/rpc/ns-eth#3-object---state-override-set).
    /// Note that not all client implementations will support this as a parameter.
    fn state(self, state: &'a spoof::State) -> Self {
        Self { inner: self.inner.state(state), f: self.f }
    }
}

impl<T, F, Y> Future for Map<T, F>
where
    T: Future,
    F: FnMut(T::Output) -> Y,
{
    type Output = Y;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pin = self.project();
        let x = futures_util::ready!(pin.inner.poll(cx));
        Poll::Ready((pin.f)(x))
    }
}

/// Provides types and methods for constructing an `eth_call`
/// [state override set](https://geth.ethereum.org/docs/rpc/ns-eth#3-object---state-override-set)
pub mod spoof {
    use super::*;
    use std::collections::HashMap;

    /// The state elements to override for a particular account.
    #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Account {
        /// Account nonce
        #[serde(skip_serializing_if = "Option::is_none")]
        pub nonce: Option<U64>,
        /// Account balance
        #[serde(skip_serializing_if = "Option::is_none")]
        pub balance: Option<U256>,
        /// Account code
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code: Option<Bytes>,
        /// Account storage
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        pub storage: Option<Storage>,
    }

    impl Account {
        /// Override the account nonce
        pub fn nonce(&mut self, nonce: U64) -> &mut Self {
            self.nonce = Some(nonce);
            self
        }
        /// Override the account balance
        pub fn balance(&mut self, bal: U256) -> &mut Self {
            self.balance = Some(bal);
            self
        }
        /// Override the code at the account
        pub fn code(&mut self, code: Bytes) -> &mut Self {
            self.code = Some(code);
            self
        }
        /// Override the value of the account storage at the given storage `key`
        pub fn store(&mut self, key: H256, val: H256) -> &mut Self {
            self.storage.get_or_insert_with(Default::default).insert(key, val);
            self
        }
    }

    /// Wraps a map from storage slot to the overriden value.
    ///
    /// Storage overrides can either replace the existing state of an account or they can be treated
    /// as a diff on the existing state.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Storage {
        /// State Diff
        #[serde(rename = "stateDiff")]
        Diff(HashMap<H256, H256>),
        /// State override
        #[serde(rename = "state")]
        Replace(HashMap<H256, H256>),
    }

    /// The default storage override is a diff on the existing state of the account.
    impl Default for Storage {
        fn default() -> Self {
            Self::Diff(Default::default())
        }
    }
    impl std::ops::Deref for Storage {
        type Target = HashMap<H256, H256>;
        fn deref(&self) -> &Self::Target {
            match self {
                Self::Diff(map) => map,
                Self::Replace(map) => map,
            }
        }
    }
    impl std::ops::DerefMut for Storage {
        fn deref_mut(&mut self) -> &mut Self::Target {
            match self {
                Self::Diff(map) => map,
                Self::Replace(map) => map,
            }
        }
    }

    /// A wrapper type that holds a complete state override set.
    #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct State(#[serde(skip_serializing_if = "HashMap::is_empty")] HashMap<Address, Account>);

    impl State {
        /// Returns a mutable reference to the [`Account`] in the map.
        pub fn account(&mut self, adr: Address) -> &mut Account {
            self.0.entry(adr).or_default()
        }
    }

    /// Returns an empty state override set.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ethers_core::{
    /// #     types::{Address, TransactionRequest, H256},
    /// #     utils::{parse_ether, Geth},
    /// # };
    /// # use ethers_providers::{Provider, Http, Middleware, call_raw::{spoof, RawCall}};
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// let geth = Geth::new().spawn();
    /// let provider = Provider::<Http>::try_from(geth.endpoint()).unwrap();
    ///
    /// let adr1: Address = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse()?;
    /// let adr2: Address = "0x295a70b2de5e3953354a6a8344e616ed314d7251".parse()?;
    /// let key = H256::from_low_u64_be(1);
    /// let val = H256::from_low_u64_be(17);
    ///
    /// let tx = TransactionRequest::default().to(adr2).from(adr1).into();
    ///
    /// // override the storage at `adr2`
    /// let mut state = spoof::state();
    /// state.account(adr2).store(key, val);
    ///
    /// // override the nonce at `adr1`
    /// state.account(adr1).nonce(2.into());
    ///
    /// provider.call_raw(&tx).state(&state).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn state() -> State {
        Default::default()
    }

    /// Returns a state override set with a single element setting the balance of the address.
    ///
    /// # Example
    /// ```no_run
    /// # use ethers_core::{
    /// #     types::{Address, TransactionRequest, H256},
    /// #     utils::{parse_ether, Geth},
    /// # };
    /// # use ethers_providers::{Provider, Http, Middleware, call_raw::{RawCall, spoof}};
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// let geth = Geth::new().spawn();
    /// let provider = Provider::<Http>::try_from(geth.endpoint()).unwrap();
    ///
    /// let adr1: Address = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse()?;
    /// let adr2: Address = "0x295a70b2de5e3953354a6a8344e616ed314d7251".parse()?;
    /// let pay_amt = parse_ether(1u64)?;
    ///
    /// // Not enough ether to pay for the transaction
    /// let tx = TransactionRequest::pay(adr2, pay_amt).from(adr1).into();
    ///
    /// // override the sender's balance for the call
    /// let state = spoof::balance(adr1, pay_amt * 2);
    /// provider.call_raw(&tx).state(&state).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn balance(adr: Address, bal: U256) -> State {
        let mut state = State::default();
        state.account(adr).balance(bal);
        state
    }

    /// Returns a state override set with a single element setting the nonce of the address.
    ///
    /// # Example
    /// ```no_run
    /// # use ethers_core::{
    /// #     types::{Address, TransactionRequest, H256},
    /// #     utils::{parse_ether, Geth},
    /// # };
    /// # use ethers_providers::{Provider, Http, Middleware, call_raw::{RawCall, spoof}};
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// let geth = Geth::new().spawn();
    /// let provider = Provider::<Http>::try_from(geth.endpoint()).unwrap();
    ///
    /// let adr: Address = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse()?;
    /// let pay_amt = parse_ether(1u64)?;
    ///
    /// let tx = TransactionRequest::default().from(adr).into();
    ///
    /// // override the sender's nonce for the call
    /// let state = spoof::nonce(adr, 72.into());
    /// provider.call_raw(&tx).state(&state).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn nonce(adr: Address, nonce: U64) -> State {
        let mut state = State::default();
        state.account(adr).nonce(nonce);
        state
    }

    /// Returns a state override set with a single element setting the code at the address.
    ///
    /// # Example
    /// ```no_run
    /// # use ethers_core::{
    /// #     types::{Address, TransactionRequest, H256},
    /// #     utils::{parse_ether, Geth},
    /// # };
    /// # use ethers_providers::{Provider, Http, Middleware, call_raw::{RawCall, spoof}};
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// let geth = Geth::new().spawn();
    /// let provider = Provider::<Http>::try_from(geth.endpoint()).unwrap();
    ///
    /// let adr: Address = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse()?;
    /// let pay_amt = parse_ether(1u64)?;
    ///
    /// let tx = TransactionRequest::default().to(adr).into();
    ///
    /// // override the code at the target address
    /// let state = spoof::code(adr, "0x00".parse()?);
    /// provider.call_raw(&tx).state(&state).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn code(adr: Address, code: Bytes) -> State {
        let mut state = State::default();
        state.account(adr).code(code);
        state
    }

    /// Returns a state override set with a single element setting the storage at the given address
    /// and key.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ethers_core::{
    /// #     types::{Address, TransactionRequest, H256},
    /// #     utils::{parse_ether, Geth},
    /// # };
    /// # use ethers_providers::{Provider, Http, Middleware, call_raw::{RawCall, spoof}};
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// let geth = Geth::new().spawn();
    /// let provider = Provider::<Http>::try_from(geth.endpoint()).unwrap();
    ///
    /// let adr: Address = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse()?;
    /// let key = H256::from_low_u64_be(1);
    /// let val = H256::from_low_u64_be(17);
    ///
    /// let tx = TransactionRequest::default().to(adr).into();
    ///
    /// // override the storage slot `key` at `adr`
    /// let state = spoof::storage(adr, key, val);
    /// provider.call_raw(&tx).state(&state).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn storage(adr: Address, key: H256, val: H256) -> State {
        let mut state = State::default();
        state.account(adr).store(key, val);
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Http, Provider};
    use ethers_core::{
        types::TransactionRequest,
        utils::{get_contract_address, keccak256, parse_ether, Geth},
    };
    use std::convert::TryFrom;

    // Deserializes eth_call parameters as owned data for testing serialization
    #[derive(Debug, Deserialize)]
    struct CallInputOwned(
        TypedTransaction,
        Option<BlockId>,
        #[serde(default)] Option<spoof::State>,
    );
    impl<'a> From<&'a CallInputOwned> for CallInput<'a> {
        fn from(src: &'a CallInputOwned) -> Self {
            Self { tx: &src.0, block: src.1, state: src.2.as_ref() }
        }
    }

    // Tests "roundtrip" serialization of calls: deserialize(serialize(call)) == call
    fn test_encode<P>(call: CallBuilder<P>) {
        let input = call.unwrap().input;
        let ser = utils::serialize(&input).to_string();
        let de: CallInputOwned = serde_json::from_str(&ser).unwrap();
        let de = CallInput::from(&de);

        assert_eq!(input.tx, de.tx);
        assert_eq!(input.state, de.state);

        let block = input.block.or_else(|| Some(BlockNumber::Latest.into()));
        assert_eq!(block, de.block);
    }

    #[test]
    fn test_serialize() {
        let adr1: Address = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse().unwrap();
        let adr2: Address = "0x295a70b2de5e3953354a6a8344e616ed314d7251".parse().unwrap();
        let k1 = utils::keccak256("foo").into();
        let v1 = H256::from_low_u64_be(534);
        let k2 = utils::keccak256("bar").into();
        let v2 = H256::from_low_u64_be(8675309);

        let tx = TypedTransaction::default();
        let (provider, _) = Provider::mocked();

        let call = provider.call_raw(&tx);
        test_encode(call);

        let mut state = spoof::state();
        state.account(adr1).nonce(1.into()).balance(2.into()).store(k1, v1).store(k2, v2);
        let call = provider.call_raw(&tx).block(100.into()).state(&state);
        test_encode(call);

        let mut state = spoof::state();
        state.account(adr1).nonce(1.into());
        state.account(adr2).nonce(7.into());
        let call = provider.call_raw(&tx).state(&state);
        test_encode(call);

        // State override with an empty acccount should be encoded as "0xab..": {}
        let mut state = spoof::state();
        state.account(adr1);
        let call = provider.call_raw(&tx).state(&state);
        test_encode(call);
    }

    #[tokio::test]
    async fn test_state_overrides() {
        let geth = Geth::new().spawn();
        let provider = Provider::<Http>::try_from(geth.endpoint()).unwrap();

        let adr1: Address = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse().unwrap();
        let adr2: Address = "0x295a70b2de5e3953354a6a8344e616ed314d7251".parse().unwrap();
        let pay_amt = parse_ether(1u64).unwrap();

        // Not enough ether to pay for the transaction
        let tx = TransactionRequest::pay(adr2, pay_amt).from(adr1).into();

        // assert that overriding the sender's balance works
        let state = spoof::balance(adr1, pay_amt * 2);
        provider.call_raw(&tx).state(&state).await.expect("eth_call success");

        // bytecode that returns the result of the SELFBALANCE opcode
        const RETURN_BALANCE: &str = "0x4760005260206000f3";
        let bytecode = RETURN_BALANCE.parse().unwrap();
        let balance = 100.into();

        let tx = TransactionRequest::default().to(adr2).into();
        let mut state = spoof::state();
        state.account(adr2).code(bytecode).balance(balance);

        // assert that overriding the code and balance at adr2 works
        let bytes = provider.call_raw(&tx).state(&state).await.unwrap();
        assert_eq!(U256::from_big_endian(bytes.as_ref()), balance);

        // bytecode that deploys a contract and returns the deployed address
        const DEPLOY_CONTRACT: &str = "0x6000600052602060006000f060005260206000f3";
        let bytecode = DEPLOY_CONTRACT.parse().unwrap();
        let nonce = 17.into();

        let mut state = spoof::state();
        state.account(adr2).code(bytecode).nonce(nonce);

        // assert that overriding nonce works (contract is deployed to expected address)
        let bytes = provider.call_raw(&tx).state(&state).await.unwrap();
        let deployed = Address::from_slice(&bytes.as_ref()[12..]);
        assert_eq!(deployed, get_contract_address(adr2, nonce.as_u64()));

        // bytecode that returns the value of storage slot 1
        const RETURN_STORAGE: &str = "0x60015460005260206000f3";
        let bytecode = RETURN_STORAGE.parse().unwrap();
        let slot = H256::from_low_u64_be(1);
        let val = keccak256("foo").into();

        let mut state = spoof::state();
        state.account(adr2).code(bytecode).store(slot, val);

        // assert that overriding storage works
        let bytes = provider.call_raw(&tx).state(&state).await.unwrap();
        assert_eq!(H256::from_slice(bytes.as_ref()), val);
    }
}
