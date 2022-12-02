//! Types for the Parity Ad-Hoc Trace API
//!
//! <https://openethereum.github.io/wiki/JSONRPC-trace-module>
use crate::types::{Bytes, H160, H256, U256};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod filter;
pub use filter::*;

mod geth;
pub use geth::*;

mod opcodes;
pub use opcodes::*;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
/// Description of the type of trace to make
pub enum TraceType {
    /// Transaction Trace
    #[serde(rename = "trace")]
    Trace,
    /// Virtual Machine Execution Trace
    #[serde(rename = "vmTrace")]
    VmTrace,
    /// State Difference
    #[serde(rename = "stateDiff")]
    StateDiff,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
/// Ad-Hoc trace API type
pub struct BlockTrace {
    /// Output Bytes
    pub output: Bytes,
    /// Transaction Trace
    pub trace: Option<Vec<TransactionTrace>>,
    /// Virtual Machine Execution Trace
    #[serde(rename = "vmTrace")]
    pub vm_trace: Option<VMTrace>,
    /// State Difference
    #[serde(rename = "stateDiff")]
    pub state_diff: Option<StateDiff>,
    /// Transaction Hash
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<H256>,
}

//---------------- State Diff ----------------
/// Aux type for Diff::Changed.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct ChangedType<T> {
    /// Previous value.
    pub from: T,
    /// Current value.
    pub to: T,
}

/// Serde-friendly `Diff` shadow.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum Diff<T> {
    /// No change.
    #[serde(rename = "=")]
    Same,
    /// A new value has been set.
    #[serde(rename = "+")]
    Born(T),
    /// A value has been removed.
    #[serde(rename = "-")]
    Died(T),
    /// Value changed.
    #[serde(rename = "*")]
    Changed(ChangedType<T>),
}

/// Serde-friendly `AccountDiff` shadow.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct AccountDiff {
    /// Account balance.
    pub balance: Diff<U256>,
    /// Account nonce.
    pub nonce: Diff<U256>,
    /// Account code.
    pub code: Diff<Bytes>,
    /// Account storage.
    pub storage: BTreeMap<H256, Diff<H256>>,
}

/// Serde-friendly `StateDiff` shadow.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct StateDiff(pub BTreeMap<H160, AccountDiff>);

// ------------------ Trace -------------
/// Trace
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct TransactionTrace {
    /// Trace address
    #[serde(rename = "traceAddress")]
    pub trace_address: Vec<usize>,
    /// Subtraces
    pub subtraces: usize,
    /// Action
    pub action: Action,
    /// Action Type
    #[serde(rename = "type")]
    pub action_type: ActionType,
    /// Result
    pub result: Option<Res>,
    /// Error
    pub error: Option<String>,
}

// ---------------- VmTrace ------------------------------
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[allow(clippy::upper_case_acronyms)]
/// A record of a full VM trace for a CALL/CREATE.
pub struct VMTrace {
    /// The code to be executed.
    pub code: Bytes,
    /// The operations executed.
    pub ops: Vec<VMOperation>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[allow(clippy::upper_case_acronyms)]
/// A record of the execution of a single VM operation.
pub struct VMOperation {
    /// The program counter.
    pub pc: usize,
    /// The gas cost for this instruction.
    pub cost: u64,
    /// Information concerning the execution of the operation.
    pub ex: Option<VMExecutedOperation>,
    /// Subordinate trace of the CALL/CREATE if applicable.
    // #[serde(bound="VMTrace: Deserialize")]
    pub sub: Option<VMTrace>,
    /// The opcode of the executed instruction
    #[serde(rename = "op")]
    pub op: ExecutedInstruction,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[allow(clippy::upper_case_acronyms)]
/// A record of an executed VM operation.
pub struct VMExecutedOperation {
    /// The total gas used.
    #[serde(rename = "used")]
    pub used: u64,
    /// The stack item placed, if any.
    pub push: Vec<U256>,
    /// If altered, the memory delta.
    #[serde(rename = "mem")]
    pub mem: Option<MemoryDiff>,
    /// The altered storage value, if any.
    #[serde(rename = "store")]
    pub store: Option<StorageDiff>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[allow(clippy::upper_case_acronyms)]
/// A diff of some chunk of memory.
pub struct MemoryDiff {
    /// Offset into memory the change begins.
    pub off: usize,
    /// The changed data.
    pub data: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[allow(clippy::upper_case_acronyms)]
/// A diff of some storage value.
pub struct StorageDiff {
    /// Which key in storage is changed.
    pub key: U256,
    /// What the value has been changed to.
    pub val: U256,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::upper_case_acronyms)]
/// Helper to classify the executed instruction
pub enum ExecutedInstruction {
    /// The instruction is recognized
    Known(Opcode),
    /// The instruction is not recognized
    Unknown(String),
}

impl Default for ExecutedInstruction {
    fn default() -> Self {
        Self::Known(Opcode::INVALID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // tx: https://etherscan.io/tx/0x4a91b11dbd2b11c308cfe7775eac2036f20c501691e3f8005d83b2dcce62d6b5
    // using the 'trace_replayTransaction' API function
    // with 'trace', 'vmTrace', 'stateDiff'
    const EXAMPLE_TRACE: &str = include!("./example-trace-str.rs");

    // block: https://etherscan.io/block/46147
    // using the 'trace_replayBlockTransactions' API function
    // with 'trace', 'vmTrace', 'stateDiff'
    const EXAMPLE_TRACES: &str = include!("./example-traces-str.rs");

    #[test]
    fn test_serialize_trace_type() {
        let trace_type_str = r#"["trace","vmTrace","stateDiff"]"#;
        let trace_type = vec![TraceType::Trace, TraceType::VmTrace, TraceType::StateDiff];

        let se_trace_str: String = serde_json::to_string(&trace_type).unwrap();
        assert_eq!(trace_type_str, se_trace_str);
    }

    #[test]
    fn test_deserialize_blocktrace() {
        let _trace: BlockTrace = serde_json::from_str(EXAMPLE_TRACE).unwrap();
    }

    #[test]
    fn test_deserialize_blocktraces() {
        let _traces: Vec<BlockTrace> = serde_json::from_str(EXAMPLE_TRACES).unwrap();
    }

    #[test]
    fn test_deserialize_unknown_opcode() {
        let example_opcodes = r#"["GAS", "CREATE2", "CUSTOMOP"]"#;
        let parsed_opcodes: Vec<ExecutedInstruction> =
            serde_json::from_str(example_opcodes).unwrap();
        assert_eq!(
            vec![
                ExecutedInstruction::Known(Opcode::GAS),
                ExecutedInstruction::Known(Opcode::CREATE2),
                ExecutedInstruction::Unknown("CUSTOMOP".to_string())
            ],
            parsed_opcodes
        )
    }
}
