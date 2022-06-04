use crate::{JsonRpcClient, PinBoxFut, Provider, ProviderError};
use ethers_core::{
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockId, BlockNumber, Bytes, H256, U256,
        U64,
    },
    utils,
};
use serde::{ser::SerializeTuple, Deserialize, Serialize};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub use spoof::{balance, code, nonce, state, storage};

pub enum Call<'a, P> {
    Build(Caller<'a, P>),
    Wait(PinBoxFut<'a, Bytes>),
}

impl<'a, P> Call<'a, P> {
    pub fn new(provider: &'a Provider<P>, tx: &'a TypedTransaction) -> Self {
        Self::Build(Caller::new(provider, tx))
    }

    pub fn map<F>(self, f: F) -> Self
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

    pub fn block(self, id: BlockId) -> Self {
        self.map(|mut call| call.input.block = Some(id))
    }
    pub fn state(self, state: &'a spoof::State) -> Self {
        self.map(|mut call| call.input.state = Some(state))
    }
    pub fn unwrap(self) -> Caller<'a, P> {
        match self {
            Self::Build(b) => b,
            _ => panic!("Call::unwrap on a Wait value"),
        }
    }
}

impl<'a, P: JsonRpcClient> Future for Call<'a, P> {
    type Output = Result<Bytes, ProviderError>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let pin = self.get_mut();
        loop {
            match pin {
                Call::Build(ref call) => {
                    let fut = Box::pin(call.execute());
                    *pin = Call::Wait(fut);
                }
                Call::Wait(ref mut fut) => return fut.as_mut().poll(ctx),
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Caller<'a, P> {
    provider: &'a Provider<P>,
    input: CallInput<'a>,
}

impl<'a, P> Caller<'a, P> {
    pub fn new(provider: &'a Provider<P>, tx: &'a TypedTransaction) -> Self {
        Self { provider, input: CallInput::new(tx) }
    }
}
impl<'a, P: JsonRpcClient> Caller<'a, P> {
    fn execute(&self) -> impl Future<Output = Result<Bytes, ProviderError>> + 'a {
        self.provider.request("eth_call", utils::serialize(&self.input))
    }
}

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

/// Provides types and methods for "spoofing" state overrides for eth_call
pub mod spoof {
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Account {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub nonce: Option<U64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub balance: Option<U256>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code: Option<Bytes>,
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        pub storage: Option<Storage>,
    }

    impl Account {
        pub fn nonce(&mut self, nonce: U64) -> &mut Self {
            self.nonce = Some(nonce);
            self
        }
        pub fn balance(&mut self, bal: U256) -> &mut Self {
            self.balance = Some(bal);
            self
        }
        pub fn code(&mut self, code: Bytes) -> &mut Self {
            self.code = Some(code);
            self
        }
        pub fn store(&mut self, key: H256, val: H256) -> &mut Self {
            self.storage.get_or_insert_with(Default::default).insert(key, val);
            self
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Storage {
        #[serde(rename = "stateDiff")]
        Diff(HashMap<H256, H256>),
        #[serde(rename = "state")]
        Replace(HashMap<H256, H256>),
    }

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

    #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct State(#[serde(skip_serializing_if = "HashMap::is_empty")] HashMap<Address, Account>);

    impl State {
        pub fn account(&mut self, adr: Address) -> &mut Account {
            self.0.entry(adr).or_default()
        }
    }

    /// # Example
    /// ```no_run
    /// # use ethers_core::{
    /// #     types::{Address, TransactionRequest, H256},
    /// #     utils::{parse_ether},
    /// # };
    /// # use ethers_providers::{Provider, Http, Middleware, spoof};
    /// # use std::convert::TryFrom;
    /// #
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let provider = Provider::<Http>::try_from("http://127.0.0.1:8545").unwrap();
    ///
    /// let adr1: Address = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse().unwrap();
    /// let adr2: Address = "0x295a70b2de5e3953354a6a8344e616ed314d7251".parse().unwrap();
    ///
    /// let tx = Default::default();
    ///
    /// let mut state = spoof::state();
    /// state.account(adr1).store(H256::default(), 1.into()).nonce(2.into());
    /// provider.call_builder(&tx).spoof(&state).await.unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn state() -> State {
        Default::default()
    }

    /// # Example
    /// ```no_run
    /// # use ethers_core::{
    /// #     types::{Address, TransactionRequest, H256},
    /// #     utils::{parse_ether},
    /// # };
    /// # use ethers_providers::{Provider, Http, Middleware, spoof};
    /// # use std::convert::TryFrom;
    /// # #[tokio::main(flavor = "current_thread")]
    /// #
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let provider = Provider::<Http>::try_from("http://127.0.0.1:8545").unwrap();
    ///
    /// let adr1: Address = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse()?;
    /// let adr2: Address = "0x295a70b2de5e3953354a6a8344e616ed314d7251".parse()?;
    /// let pay_amt = parse_ether(1u64)?;
    ///
    /// // Not enough ether to pay for the transaction
    /// let tx = TransactionRequest::pay(adr2, pay_amt).from(adr1);
    /// let tx = tx.into();
    ///
    /// // override the sender's balance for the call
    /// let mut state = spoof::balance(adr1, pay_amt * 2);
    /// provider.call_builder(&tx).spoof(&state).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn balance(adr: Address, bal: U256) -> State {
        let mut state = State::default();
        state.account(adr).balance(bal);
        state
    }

    pub fn nonce(adr: Address, nonce: U64) -> State {
        let mut state = State::default();
        state.account(adr).nonce(nonce);
        state
    }

    pub fn code(adr: Address, code: Bytes) -> State {
        let mut state = State::default();
        state.account(adr).code(code);
        state
    }

    pub fn storage(adr: Address, key: H256, val: H256) -> State {
        let mut state = State::default();
        state.account(adr).store(key, val);
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Http, Middleware, Provider};
    use ethers_core::{
        types::TransactionRequest,
        utils::{parse_ether, Anvil},
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
    fn test_encode<'a, P>(call: Call<'a, P>) {
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
    async fn test_future() {
        let anvil = Anvil::new().spawn();
        let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();

        let accounts = provider.get_accounts().await.unwrap();
        let tx = TransactionRequest::pay(accounts[1], parse_ether(1u64).unwrap()).from(accounts[0]);

        provider.call_raw(&tx.into()).await.expect("eth_call success");
    }
}
