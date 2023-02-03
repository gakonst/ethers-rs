use crate::{
    types::{Address, Bytes, NameOrAddress, H256, U256},
    utils::from_int_or_hex,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

// https://github.com/ethereum/go-ethereum/blob/a9ef135e2dd53682d106c6a2aede9187026cc1de/eth/tracers/logger/logger.go#L406-L411
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultFrame {
    pub failed: bool,
    #[serde(deserialize_with = "from_int_or_hex")]
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

// https://github.com/ethereum/go-ethereum/blob/a9ef135e2dd53682d106c6a2aede9187026cc1de/eth/tracers/native/call.go#L37
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallFrame {
    #[serde(rename = "type")]
    pub typ: String,
    pub from: Address,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<NameOrAddress>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
    #[serde(deserialize_with = "from_int_or_hex")]
    pub gas: U256,
    #[serde(deserialize_with = "from_int_or_hex", rename = "gasUsed")]
    pub gas_used: U256,
    pub input: Bytes,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Bytes>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calls: Option<Vec<CallFrame>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GethTraceFrame {
    Default(DefaultFrame),
    CallTracer(CallFrame),
}

impl From<DefaultFrame> for GethTraceFrame {
    fn from(value: DefaultFrame) -> Self {
        GethTraceFrame::Default(value)
    }
}

impl From<CallFrame> for GethTraceFrame {
    fn from(value: CallFrame) -> Self {
        GethTraceFrame::CallTracer(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GethTrace {
    Known(GethTraceFrame),
    Unknown(Value),
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
    #[serde(rename = "callTracer")]
    CallTracer,
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
    // TODO: Add stateoverrides and blockoverrides options
}
