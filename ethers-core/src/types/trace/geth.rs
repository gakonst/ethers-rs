use crate::types::{Bytes, H256};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct GethTrace {
    failed: bool,
    gas: u64,
    #[serde(rename = "returnValue")]
    return_value: Bytes,
    #[serde(rename = "structLogs")]
    struct_logs: Vec<StructLog>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StructLog {
    depth: u64,
    error: Option<String>,
    gas: u64,
    #[serde(rename = "gasCost")]
    gas_cost: u64,
    memory: Option<Vec<String>>,
    op: String,
    pc: u64,
    stack: Vec<String>,
    storage: BTreeMap<H256, H256>,
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
