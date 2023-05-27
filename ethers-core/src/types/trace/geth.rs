mod call;
mod four_byte;
mod noop;
mod pre_state;

pub use self::{
    call::{CallConfig, CallFrame, CallLogFrame},
    four_byte::FourByteFrame,
    noop::NoopFrame,
    pre_state::{AccountState, DiffMode, PreStateConfig, PreStateFrame, PreStateMode},
};
use crate::types::{
    serde_helpers::deserialize_stringified_numeric, Address, Bytes, H256, U256, U64,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

// https://github.com/ethereum/go-ethereum/blob/a9ef135e2dd53682d106c6a2aede9187026cc1de/eth/tracers/logger/logger.go#L406-L411
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultFrame {
    pub failed: bool,
    #[serde(deserialize_with = "deserialize_stringified_numeric")]
    pub gas: U256,
    #[serde(rename = "returnValue")]
    pub return_value: Bytes,
    #[serde(rename = "structLogs")]
    pub struct_logs: Vec<StructLog>,
}

// https://github.com/ethereum/go-ethereum/blob/366d2169fbc0e0f803b68c042b77b6b480836dbc/eth/tracers/logger/logger.go#L413-L426
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructLog {
    pub depth: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub gas: u64,
    #[serde(rename = "gasCost")]
    pub gas_cost: u64,
    /// ref <https://github.com/ethereum/go-ethereum/blob/366d2169fbc0e0f803b68c042b77b6b480836dbc/eth/tracers/logger/logger.go#L450-L452>
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<Vec<String>>,
    pub op: String,
    pub pc: u64,
    #[serde(default, rename = "refund", skip_serializing_if = "Option::is_none")]
    pub refund_counter: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack: Option<Vec<U256>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<BTreeMap<H256, H256>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GethTraceFrame {
    Default(DefaultFrame),
    NoopTracer(NoopFrame),
    FourByteTracer(FourByteFrame),
    CallTracer(CallFrame),
    PreStateTracer(PreStateFrame),
}

impl From<DefaultFrame> for GethTraceFrame {
    fn from(value: DefaultFrame) -> Self {
        GethTraceFrame::Default(value)
    }
}

impl From<FourByteFrame> for GethTraceFrame {
    fn from(value: FourByteFrame) -> Self {
        GethTraceFrame::FourByteTracer(value)
    }
}

impl From<CallFrame> for GethTraceFrame {
    fn from(value: CallFrame) -> Self {
        GethTraceFrame::CallTracer(value)
    }
}

impl From<PreStateFrame> for GethTraceFrame {
    fn from(value: PreStateFrame) -> Self {
        GethTraceFrame::PreStateTracer(value)
    }
}

impl From<NoopFrame> for GethTraceFrame {
    fn from(value: NoopFrame) -> Self {
        GethTraceFrame::NoopTracer(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum GethTraceResult {
    ResultKnown { result: GethTraceFrame },
    ResultUnknown { result: Value },
    DefaultKnown(GethTraceFrame),
    DefaultUnknown(Value),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(from = "GethTraceResult")]
#[serde(untagged)]
pub enum GethTrace {
    Known(GethTraceFrame),
    Unknown(Value),
}

impl From<GethTraceResult> for GethTrace {
    fn from(value: GethTraceResult) -> Self {
        match value {
            GethTraceResult::DefaultKnown(t) => GethTrace::Known(t),
            GethTraceResult::DefaultUnknown(v) => GethTrace::Unknown(v),
            GethTraceResult::ResultKnown { result } => GethTrace::Known(result),
            GethTraceResult::ResultUnknown { result } => GethTrace::Unknown(result),
        }
    }
}

impl From<GethTraceFrame> for GethTrace {
    fn from(value: GethTraceFrame) -> Self {
        GethTrace::Known(value)
    }
}

impl From<Value> for GethTrace {
    fn from(value: Value) -> Self {
        GethTrace::Unknown(value)
    }
}

/// Available built-in tracers
///
/// See <https://geth.ethereum.org/docs/developers/evm-tracing/built-in-tracers>
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum GethDebugBuiltInTracerType {
    #[serde(rename = "4byteTracer")]
    FourByteTracer,
    #[serde(rename = "callTracer")]
    CallTracer,
    #[serde(rename = "prestateTracer")]
    PreStateTracer,
    #[serde(rename = "noopTracer")]
    NoopTracer,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GethDebugBuiltInTracerConfig {
    CallTracer(CallConfig),
    PreStateTracer(PreStateConfig),
}

/// Available tracers
///
/// See <https://geth.ethereum.org/docs/developers/evm-tracing/built-in-tracers> and <https://geth.ethereum.org/docs/developers/evm-tracing/custom-tracer>
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GethDebugTracerType {
    /// built-in tracer
    BuiltInTracer(GethDebugBuiltInTracerType),

    /// custom JS tracer
    JsTracer(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GethDebugTracerConfig {
    /// built-in tracer
    BuiltInTracer(GethDebugBuiltInTracerConfig),

    /// custom JS tracer
    JsTracer(Value),
}

/// Bindings for additional `debug_traceTransaction` options
///
/// See <https://geth.ethereum.org/docs/rpc/ns-debug#debug_tracetransaction>
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GethDebugTracingOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_storage: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_stack: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_memory: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_return_data: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracer: Option<GethDebugTracerType>,
    /// tracerConfig is slated for Geth v1.11.0
    /// See <https://github.com/ethereum/go-ethereum/issues/26513>
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracer_config: Option<GethDebugTracerConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
}

/// Bindings for additional `debug_traceCall` options
///
/// See <https://geth.ethereum.org/docs/rpc/ns-debug#debug_tracecall>
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GethDebugTracingCallOptions {
    #[serde(flatten)]
    pub tracing_options: GethDebugTracingOptions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_overrides: Option<spoof::State>,
    // TODO: Add blockoverrides options
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
    /// # Examples
    ///
    /// ```ignore
    /// # use ethers_core::{
    /// #     types::{Address, TransactionRequest, H256, spoof},
    /// #     utils::{parse_ether, Geth},
    /// # };
    /// # use ethers_providers::{Provider, Http, Middleware, call_raw::RawCall};
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
    /// # Examples
    ///
    /// ```ignore
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
    /// # Examples
    ///
    /// ```ignore
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
    /// # Examples
    ///
    /// ```ignore
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
    /// # Examples
    ///
    /// ```ignore
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
