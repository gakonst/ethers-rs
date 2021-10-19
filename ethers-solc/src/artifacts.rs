//! Solc artifact types

use std::{collections::BTreeMap, fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// Input type `solc` expects
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompilerInput {
    pub language: String,
    pub sources: BTreeMap<String, Source>,
    pub settings: Settings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub optimizer: Optimizer,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default)]
    pub output_selection: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    #[serde(
        default,
        with = "display_from_str_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub evm_version: Option<EvmVersion>,
    #[serde(
        default,
        skip_serializing_if = "::std::collections::BTreeMap::is_empty"
    )]
    pub libraries: BTreeMap<String, BTreeMap<String, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Optimizer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runs: Option<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvmVersion {
    Homestead,
    TangerineWhistle,
    SpuriusDragon,
    Constantinople,
    Petersburg,
    Istanbul,
    Berlin,
    London,
    Byzantium,
}

impl fmt::Display for EvmVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            EvmVersion::Homestead => "homestead",
            EvmVersion::TangerineWhistle => "tangerineWhistle",
            EvmVersion::SpuriusDragon => "spuriusDragon",
            EvmVersion::Constantinople => "constantinople",
            EvmVersion::Petersburg => "petersburg",
            EvmVersion::Istanbul => "istanbul",
            EvmVersion::Berlin => "berlin",
            EvmVersion::London => "london",
            EvmVersion::Byzantium => "byzantium",
        };
        write!(f, "{}", string)
    }
}

impl FromStr for EvmVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "homestead" => Ok(EvmVersion::Homestead),
            "tangerineWhistle" => Ok(EvmVersion::TangerineWhistle),
            "spuriusDragon" => Ok(EvmVersion::SpuriusDragon),
            "constantinople" => Ok(EvmVersion::Constantinople),
            "petersburg" => Ok(EvmVersion::Petersburg),
            "istanbul" => Ok(EvmVersion::Istanbul),
            "berlin" => Ok(EvmVersion::Berlin),
            "london" => Ok(EvmVersion::London),
            "byzantium" => Ok(EvmVersion::Byzantium),
            s => Err(format!("Unknown evm version: {}", s)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "useLiteralContent")]
    pub use_literal_content: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Source {
    pub content: String,
}

/// Output type `solc` produces
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct CompilerOutput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Error>,
    pub sources: BTreeMap<String, SourceFile>,
    pub contracts: BTreeMap<String, BTreeMap<String, Contract>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Contract {
    /// The Ethereum Contract ABI. If empty, it is represented as an empty
    /// array. See https://docs.soliditylang.org/en/develop/abi-spec.html
    pub abi: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    #[serde(default)]
    pub userdoc: UserDoc,
    #[serde(default)]
    pub devdoc: DevDoc,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ir: Option<String>,
    #[serde(
        default,
        rename = "storageLayout",
        skip_serializing_if = "StorageLayout::is_empty"
    )]
    pub storage_layout: StorageLayout,
    /// EVM-related outputs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evm: Option<Evm>,
    /// Ewasm related outputs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ewasm: Option<Ewasm>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct UserDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "::std::collections::BTreeMap::is_empty"
    )]
    pub methods: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notice: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct DevDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(
        default,
        rename = "custom:experimental",
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_experimental: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "::std::collections::BTreeMap::is_empty"
    )]
    pub methods: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Evm {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembly: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_assembly: Option<serde_json::Value>,
    pub bytecode: Bytecode,
    pub deployed_bytecode: DeployedBytecode,
    /// The list of function hashes
    #[serde(
        default,
        skip_serializing_if = "::std::collections::BTreeMap::is_empty"
    )]
    pub method_identifiers: BTreeMap<String, String>,
    /// Function gas estimates
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_estimates: Option<GasEstimates>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Bytecode {
    /// Debugging information at function level
    #[serde(
        default,
        skip_serializing_if = "::std::collections::BTreeMap::is_empty"
    )]
    pub function_debug_data: BTreeMap<String, FunctionDebugData>,
    /// The bytecode as a hex string.
    pub object: String,
    /// Opcodes list (string)
    pub opcodes: String,
    /// The source mapping as a string. See the source mapping definition.
    pub source_map: String,
    /// Array of sources generated by the compiler. Currently only contains a
    /// single Yul file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generated_sources: Vec<GeneratedSource>,
    /// If given, this is an unlinked object.
    #[serde(default)]
    pub link_references: BTreeMap<String, BTreeMap<String, Vec<Offsets>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDebugData {
    pub entry_point: u32,
    pub id: u32,
    pub parameter_slots: u32,
    pub return_slots: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct GeneratedSource {
    pub ast: serde_json::Value,
    pub contents: String,
    pub id: u32,
    pub language: String,
    pub name: String,
}

/// Byte offsets into the bytecode.
/// Linking replaces the 20 bytes located there.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Offsets {
    pub start: u32,
    pub length: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DeployedBytecode {
    #[serde(flatten)]
    pub bytecode: Option<Bytecode>,
    #[serde(
        default,
        rename = "immutableReferences",
        skip_serializing_if = "::std::collections::BTreeMap::is_empty"
    )]
    pub immutable_references: BTreeMap<String, Vec<Offsets>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct GasEstimates {
    pub creation: Creation,
    pub external: BTreeMap<String, String>,
    pub internal: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Creation {
    pub code_deposit_cost: String,
    pub execution_cost: String,
    pub total_cost: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Ewasm {
    pub wast: String,
    pub wasm: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct StorageLayout {
    pub storage: Vec<Storage>,
    pub types: BTreeMap<String, StorageType>,
}

impl StorageLayout {
    fn is_empty(&self) -> bool {
        self.storage.is_empty() && self.types.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Storage {
    #[serde(rename = "astId")]
    pub ast_id: u64,
    pub contract: String,
    pub label: String,
    pub offset: i64,
    pub slot: String,
    #[serde(rename = "type")]
    pub storage_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct StorageType {
    pub encoding: String,
    pub label: String,
    #[serde(rename = "numberOfBytes")]
    pub number_of_bytes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_location: Option<SourceLocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary_source_locations: Vec<SourceLocation>,
    pub r#type: String,
    pub component: String,
    pub severity: String,
    pub error_code: Option<String>,
    pub message: String,
    pub formatted_message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SourceLocation {
    pub file: String,
    pub start: i32,
    pub end: i32,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SourceFile {
    pub id: u32,
    pub ast: serde_json::Value,
}

mod display_from_str_opt {
    use serde::{de, Deserialize, Deserializer, Serializer};
    use std::{fmt, str::FromStr};

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        if let Some(value) = value {
            serializer.collect_str(value)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        T::Err: fmt::Display,
    {
        if let Some(s) = Option::<String>::deserialize(deserializer)? {
            s.parse().map_err(de::Error::custom).map(Some)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn can_parse_compiler_output() {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("test-data/out");

        for path in fs::read_dir(dir).unwrap() {
            let path = path.unwrap().path();
            let compiler_output = fs::read_to_string(&path).unwrap();
            serde_json::from_str::<CompilerOutput>(&compiler_output).unwrap_or_else(|err| {
                panic!(
                    "Failed to read compiler output of {} {}",
                    path.display(),
                    err
                )
            });
        }
    }

    #[test]
    fn can_parse_compiler_input() {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("test-data/in");

        for path in fs::read_dir(dir).unwrap() {
            let path = path.unwrap().path();
            let compiler_output = fs::read_to_string(&path).unwrap();
            serde_json::from_str::<CompilerInput>(&compiler_output).unwrap_or_else(|err| {
                panic!(
                    "Failed to read compiler output of {} {}",
                    path.display(),
                    err
                )
            });
        }
    }
}
