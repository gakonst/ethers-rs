use crate::types::{Bytes, H160, H256, U256};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct GethTrace {
    pub failed: bool,
    pub gas: u64,
    #[serde(rename = "returnValue")]
    pub return_value: Bytes,
    #[serde(rename = "structLogs")]
    pub struct_logs: Vec<StructLog>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StructLog {
    pub depth: u64,
    pub error: Option<String>,
    pub gas: u64,
    #[serde(rename = "gasCost")]
    pub gas_cost: u64,
    pub memory: Option<Vec<u8>>,
    pub op: String,
    pub pc: U256,
    pub stack: Vec<U256>,
    pub storage: BTreeMap<H160, BTreeMap<H256, H256>>,
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
    pub tracer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
}
