//! bindings for standard json output selection

use std::{fmt, str::FromStr};

/// Contract level output selection
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContractOutputSelection {
    Abi,
    DevDoc,
    UserDoc,
    Metadata,
    Ir,
    IrOptimized,
    StorageLayout,
}

/// Contract level output selection for `evm`
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EvmOutputSelection {
    All,
    Assembly,
    LegacyAssembly,
    MethodIdentifiers,
    GasEstimates,
    ByteCode(BytecodeOutputSelection),
    DeployedByteCode(BytecodeOutputSelection),
    Ewasm(EwasmOutputSelection),
}

/// Contract level output selection for `evm.bytecode`
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BytecodeOutputSelection {
    All,
    FunctionDebugData,
    Object,
    Opcodes,
    SourceMap,
    LinkReferences,
    GeneratedSources,
}

/// Contract level output selection for `evm.ewasm`
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EwasmOutputSelection {
    All,
    Wast,
    Wasm,
}

impl fmt::Display for EwasmOutputSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EwasmOutputSelection::All => f.write_str("ewasm"),
            EwasmOutputSelection::Wast => f.write_str("ewasm.wast"),
            EwasmOutputSelection::Wasm => f.write_str("ewasm.wasm"),
        }
    }
}

impl FromStr for EwasmOutputSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ewasm" => Ok(EwasmOutputSelection::All),
            "ewasm.wast" => Ok(EwasmOutputSelection::Wast),
            "ewasm.wasm" => Ok(EwasmOutputSelection::Wasm),
            s => Err(format!("Invalid ewasm: {}", s)),
        }
    }
}
