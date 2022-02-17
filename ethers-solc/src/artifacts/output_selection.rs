//! bindings for standard json output selection

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

/// Contract level output selection
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContractOutputSelection {
    Abi,
    DevDoc,
    UserDoc,
    Metadata,
    Ir,
    IrOptimized,
    StorageLayout,
    Evm(EvmOutputSelection),
    Ewasm(EwasmOutputSelection),
}

impl ContractOutputSelection {
    /// Returns the basic set of contract level settings that should be included in the `Contract`
    /// that solc emits:
    ///    - "abi"
    ///    - "evm.bytecode"
    ///    - "evm.deployedBytecode"
    ///    - "evm.methodIdentifiers"
    pub fn basic() -> Vec<ContractOutputSelection> {
        vec![
            ContractOutputSelection::Abi,
            BytecodeOutputSelection::All.into(),
            DeployedBytecodeOutputSelection::All.into(),
            EvmOutputSelection::MethodIdentifiers.into(),
        ]
    }
}

impl Serialize for ContractOutputSelection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ContractOutputSelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for ContractOutputSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractOutputSelection::Abi => f.write_str("abi"),
            ContractOutputSelection::DevDoc => f.write_str("devdoc"),
            ContractOutputSelection::UserDoc => f.write_str("userdoc"),
            ContractOutputSelection::Metadata => f.write_str("metadata"),
            ContractOutputSelection::Ir => f.write_str("ir"),
            ContractOutputSelection::IrOptimized => f.write_str("irOptimized"),
            ContractOutputSelection::StorageLayout => f.write_str("storageLayout"),
            ContractOutputSelection::Evm(e) => e.fmt(f),
            ContractOutputSelection::Ewasm(e) => e.fmt(f),
        }
    }
}

impl FromStr for ContractOutputSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "abi" => Ok(ContractOutputSelection::Abi),
            "devdoc" => Ok(ContractOutputSelection::DevDoc),
            "userdoc" => Ok(ContractOutputSelection::UserDoc),
            "metadata" => Ok(ContractOutputSelection::Metadata),
            "ir" => Ok(ContractOutputSelection::Ir),
            "ir-optimized" | "irOptimized" | "iroptimized" => {
                Ok(ContractOutputSelection::IrOptimized)
            }
            "storage-layout" | "storagelayout" | "storageLayout" => {
                Ok(ContractOutputSelection::StorageLayout)
            }
            s => EvmOutputSelection::from_str(s)
                .map(ContractOutputSelection::Evm)
                .or_else(|_| EwasmOutputSelection::from_str(s).map(ContractOutputSelection::Ewasm))
                .map_err(|_| format!("Invalid contract output selection: {}", s)),
        }
    }
}

impl<T: Into<EvmOutputSelection>> From<T> for ContractOutputSelection {
    fn from(evm: T) -> Self {
        ContractOutputSelection::Evm(evm.into())
    }
}

impl From<EwasmOutputSelection> for ContractOutputSelection {
    fn from(ewasm: EwasmOutputSelection) -> Self {
        ContractOutputSelection::Ewasm(ewasm)
    }
}

/// Contract level output selection for `evm`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EvmOutputSelection {
    All,
    Assembly,
    LegacyAssembly,
    MethodIdentifiers,
    GasEstimates,
    ByteCode(BytecodeOutputSelection),
    DeployedByteCode(DeployedBytecodeOutputSelection),
}

impl From<BytecodeOutputSelection> for EvmOutputSelection {
    fn from(b: BytecodeOutputSelection) -> Self {
        EvmOutputSelection::ByteCode(b)
    }
}

impl From<DeployedBytecodeOutputSelection> for EvmOutputSelection {
    fn from(b: DeployedBytecodeOutputSelection) -> Self {
        EvmOutputSelection::DeployedByteCode(b)
    }
}

impl Serialize for EvmOutputSelection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for EvmOutputSelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for EvmOutputSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvmOutputSelection::All => f.write_str("evm"),
            EvmOutputSelection::Assembly => f.write_str("evm.assembly"),
            EvmOutputSelection::LegacyAssembly => f.write_str("evm.legacyAssembly"),
            EvmOutputSelection::MethodIdentifiers => f.write_str("evm.methodIdentifiers"),
            EvmOutputSelection::GasEstimates => f.write_str("evm.gasEstimates"),
            EvmOutputSelection::ByteCode(b) => b.fmt(f),
            EvmOutputSelection::DeployedByteCode(b) => b.fmt(f),
        }
    }
}

impl FromStr for EvmOutputSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "evm" => Ok(EvmOutputSelection::All),
            "asm" | "evm.assembly" => Ok(EvmOutputSelection::Assembly),
            "evm.legacyAssembly" => Ok(EvmOutputSelection::LegacyAssembly),
            "methodidentifiers" | "evm.methodIdentifiers" | "evm.methodidentifiers" => {
                Ok(EvmOutputSelection::MethodIdentifiers)
            }
            "gas" | "evm.gasEstimates" | "evm.gasestimates" => Ok(EvmOutputSelection::GasEstimates),
            s => BytecodeOutputSelection::from_str(s)
                .map(EvmOutputSelection::ByteCode)
                .or_else(|_| {
                    DeployedBytecodeOutputSelection::from_str(s)
                        .map(EvmOutputSelection::DeployedByteCode)
                })
                .map_err(|_| format!("Invalid evm selection: {}", s)),
        }
    }
}

/// Contract level output selection for `evm.bytecode`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BytecodeOutputSelection {
    All,
    FunctionDebugData,
    Object,
    Opcodes,
    SourceMap,
    LinkReferences,
    GeneratedSources,
}

impl Serialize for BytecodeOutputSelection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for BytecodeOutputSelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for BytecodeOutputSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BytecodeOutputSelection::All => f.write_str("evm.bytecode"),
            BytecodeOutputSelection::FunctionDebugData => {
                f.write_str("evm.bytecode.functionDebugData")
            }
            BytecodeOutputSelection::Object => f.write_str("evm.bytecode.object"),
            BytecodeOutputSelection::Opcodes => f.write_str("evm.bytecode.opcodes"),
            BytecodeOutputSelection::SourceMap => f.write_str("evm.bytecode.sourceMap"),
            BytecodeOutputSelection::LinkReferences => f.write_str("evm.bytecode.linkReferences"),
            BytecodeOutputSelection::GeneratedSources => {
                f.write_str("evm.bytecode.generatedSources")
            }
        }
    }
}

impl FromStr for BytecodeOutputSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "evm.bytecode" => Ok(BytecodeOutputSelection::All),
            "evm.bytecode.functionDebugData" => Ok(BytecodeOutputSelection::FunctionDebugData),
            "evm.bytecode.object" => Ok(BytecodeOutputSelection::Object),
            "evm.bytecode.opcodes" => Ok(BytecodeOutputSelection::Opcodes),
            "evm.bytecode.sourceMap" => Ok(BytecodeOutputSelection::SourceMap),
            "evm.bytecode.linkReferences" => Ok(BytecodeOutputSelection::LinkReferences),
            "evm.bytecode.generatedSources" => Ok(BytecodeOutputSelection::GeneratedSources),
            s => Err(format!("Invalid bytecode selection: {}", s)),
        }
    }
}

/// Contract level output selection for `evm.deployedBytecode`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeployedBytecodeOutputSelection {
    All,
    FunctionDebugData,
    Object,
    Opcodes,
    SourceMap,
    LinkReferences,
    GeneratedSources,
}

impl Serialize for DeployedBytecodeOutputSelection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for DeployedBytecodeOutputSelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for DeployedBytecodeOutputSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeployedBytecodeOutputSelection::All => f.write_str("evm.deployedBytecode"),
            DeployedBytecodeOutputSelection::FunctionDebugData => {
                f.write_str("evm.deployedBytecode.functionDebugData")
            }
            DeployedBytecodeOutputSelection::Object => f.write_str("evm.deployedBytecode.object"),
            DeployedBytecodeOutputSelection::Opcodes => f.write_str("evm.deployedBytecode.opcodes"),
            DeployedBytecodeOutputSelection::SourceMap => {
                f.write_str("evm.deployedBytecode.sourceMap")
            }
            DeployedBytecodeOutputSelection::LinkReferences => {
                f.write_str("evm.deployedBytecode.linkReferences")
            }
            DeployedBytecodeOutputSelection::GeneratedSources => {
                f.write_str("evm.deployedBytecode.generatedSources")
            }
        }
    }
}

impl FromStr for DeployedBytecodeOutputSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "evm.deployedBytecode" => Ok(DeployedBytecodeOutputSelection::All),
            "evm.deployedBytecode.functionDebugData" => {
                Ok(DeployedBytecodeOutputSelection::FunctionDebugData)
            }
            "evm.deployedBytecode.object" => Ok(DeployedBytecodeOutputSelection::Object),
            "evm.deployedBytecode.opcodes" => Ok(DeployedBytecodeOutputSelection::Opcodes),
            "evm.deployedBytecode.sourceMap" => Ok(DeployedBytecodeOutputSelection::SourceMap),
            "evm.deployedBytecode.linkReferences" => {
                Ok(DeployedBytecodeOutputSelection::LinkReferences)
            }
            "evm.deployedBytecode.generatedSources" => {
                Ok(DeployedBytecodeOutputSelection::GeneratedSources)
            }
            s => Err(format!("Invalid deployedBytecode selection: {}", s)),
        }
    }
}

/// Contract level output selection for `evm.ewasm`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EwasmOutputSelection {
    All,
    Wast,
    Wasm,
}

impl Serialize for EwasmOutputSelection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for EwasmOutputSelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
    }
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
            s => Err(format!("Invalid ewasm selection: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn outputselection_serde_works() {
        let mut output = BTreeMap::default();
        output.insert(
            "*".to_string(),
            vec![
                "abi".to_string(),
                "evm.bytecode".to_string(),
                "evm.deployedBytecode".to_string(),
                "evm.methodIdentifiers".to_string(),
            ],
        );

        let json = serde_json::to_string(&output).unwrap();
        let deserde_selection: BTreeMap<String, Vec<ContractOutputSelection>> =
            serde_json::from_str(&json).unwrap();

        assert_eq!(json, serde_json::to_string(&deserde_selection).unwrap());
    }
}
