//! Solc artifact types

use colored::Colorize;
use md5::Digest;
use semver::Version;
use std::{
    collections::BTreeMap,
    fmt, fs, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{compile::*, utils};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

/// An ordered list of files and their source
pub type Sources = BTreeMap<PathBuf, Source>;

/// Input type `solc` expects
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompilerInput {
    pub language: String,
    pub sources: Sources,
    pub settings: Settings,
}

impl CompilerInput {
    /// Reads all contracts found under the path
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        Source::read_all_from(path.as_ref()).map(Self::with_sources)
    }

    /// Creates a new Compiler input with default settings and the given sources
    pub fn with_sources(sources: Sources) -> Self {
        Self { language: "Solidity".to_string(), sources, settings: Default::default() }
    }

    /// Sets the EVM version for compilation
    pub fn evm_version(mut self, version: EvmVersion) -> Self {
        self.settings.evm_version = Some(version);
        self
    }

    /// Sets the optimizer runs (default = 200)
    pub fn optimizer(mut self, runs: usize) -> Self {
        self.settings.optimizer.runs(runs);
        self
    }
}

impl Default for CompilerInput {
    fn default() -> Self {
        Self::with_sources(Default::default())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub optimizer: Optimizer,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// This field can be used to select desired outputs based
    /// on file and contract names.
    /// If this field is omitted, then the compiler loads and does type
    /// checking, but will not generate any outputs apart from errors.
    /// The first level key is the file name and the second level key is the
    /// contract name. An empty contract name is used for outputs that are
    /// not tied to a contract but to the whole source file like the AST.
    /// A star as contract name refers to all contracts in the file.
    /// Similarly, a star as a file name matches all files.
    /// To select all outputs the compiler can possibly generate, use
    /// "outputSelection: { "*": { "*": [ "*" ], "": [ "*" ] } }"
    /// but note that this might slow down the compilation process needlessly.
    ///
    /// The available output types are as follows:
    ///
    /// File level (needs empty string as contract name):
    ///   ast - AST of all source files
    ///
    /// Contract level (needs the contract name or "*"):
    ///   abi - ABI
    ///   devdoc - Developer documentation (natspec)
    ///   userdoc - User documentation (natspec)
    ///   metadata - Metadata
    ///   ir - Yul intermediate representation of the code before optimization
    ///   irOptimized - Intermediate representation after optimization
    ///   storageLayout - Slots, offsets and types of the contract's state
    ///     variables.
    ///   evm.assembly - New assembly format
    ///   evm.legacyAssembly - Old-style assembly format in JSON
    ///   evm.bytecode.functionDebugData - Debugging information at function level
    ///   evm.bytecode.object - Bytecode object
    ///   evm.bytecode.opcodes - Opcodes list
    ///   evm.bytecode.sourceMap - Source mapping (useful for debugging)
    ///   evm.bytecode.linkReferences - Link references (if unlinked object)
    ///   evm.bytecode.generatedSources - Sources generated by the compiler
    ///   evm.deployedBytecode* - Deployed bytecode (has all the options that
    ///     evm.bytecode has)
    ///   evm.deployedBytecode.immutableReferences - Map from AST ids to
    ///     bytecode ranges that reference immutables
    ///   evm.methodIdentifiers - The list of function hashes
    ///   evm.gasEstimates - Function gas estimates
    ///   ewasm.wast - Ewasm in WebAssembly S-expressions format
    ///   ewasm.wasm - Ewasm in WebAssembly binary format
    ///
    /// Note that using a using `evm`, `evm.bytecode`, `ewasm`, etc. will select
    /// every target part of that output. Additionally, `*` can be used as a
    /// wildcard to request everything.
    ///
    /// The default output selection is
    ///
    /// ```json
    ///   {
    ///    "*": {
    ///      "*": [
    ///        "abi",
    ///        "evm.bytecode",
    ///        "evm.deployedBytecode",
    ///        "evm.methodIdentifiers"
    ///      ],
    ///      "": [
    ///        "ast"
    ///      ]
    ///    }
    ///  }
    /// ```
    #[serde(default)]
    pub output_selection: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    #[serde(default, with = "display_from_str_opt", skip_serializing_if = "Option::is_none")]
    pub evm_version: Option<EvmVersion>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub libraries: BTreeMap<String, BTreeMap<String, String>>,
}

impl Settings {
    /// Default output selection for compiler output
    pub fn default_output_selection() -> BTreeMap<String, BTreeMap<String, Vec<String>>> {
        let mut output_selection = BTreeMap::default();
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
        output_selection.insert("*".to_string(), output);
        output_selection
    }

    /// Adds `ast` to output
    pub fn with_ast(mut self) -> Self {
        let output = self.output_selection.entry("*".to_string()).or_insert_with(BTreeMap::default);
        output.insert("".to_string(), vec!["ast".to_string()]);
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            optimizer: Default::default(),
            metadata: None,
            output_selection: Self::default_output_selection(),
            evm_version: Some(EvmVersion::Istanbul),
            libraries: Default::default(),
        }
        .with_ast()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Optimizer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runs: Option<usize>,
}

impl Optimizer {
    pub fn runs(&mut self, runs: usize) {
        self.runs = Some(runs);
    }

    pub fn disable(&mut self) {
        self.enabled.take();
    }

    pub fn enable(&mut self) {
        self.enabled = Some(true)
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self { enabled: Some(false), runs: Some(200) }
    }
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

impl EvmVersion {
    /// Checks against the given solidity `semver::Version`
    pub fn normalize_version(self, version: &Version) -> Option<EvmVersion> {
        // the EVM version flag was only added at 0.4.21
        // we work our way backwards
        if version >= &CONSTANTINOPLE_SOLC {
            // If the Solc is at least at london, it supports all EVM versions
            Some(if version >= &LONDON_SOLC {
                self
                // For all other cases, cap at the at-the-time highest possible
                // fork
            } else if version >= &BERLIN_SOLC && self >= EvmVersion::Berlin {
                EvmVersion::Berlin
            } else if version >= &ISTANBUL_SOLC && self >= EvmVersion::Istanbul {
                EvmVersion::Istanbul
            } else if version >= &PETERSBURG_SOLC && self >= EvmVersion::Petersburg {
                EvmVersion::Petersburg
            } else if self >= EvmVersion::Constantinople {
                EvmVersion::Constantinople
            } else {
                self
            })
        } else {
            None
        }
    }
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "useLiteralContent")]
    pub use_literal_content: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Source {
    pub content: String,
}

impl Source {
    /// Reads the file content
    pub fn read(file: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self { content: fs::read_to_string(file.as_ref())? })
    }

    /// Finds all source files under the given dir path and reads them all
    pub fn read_all_from(dir: impl AsRef<Path>) -> io::Result<Sources> {
        Self::read_all(utils::source_files(dir)?)
    }

    /// Reads all files
    pub fn read_all<T, I>(files: I) -> io::Result<Sources>
    where
        I: IntoIterator<Item = T>,
        T: Into<PathBuf>,
    {
        files
            .into_iter()
            .map(Into::into)
            .map(|file| Self::read(&file).map(|source| (file, source)))
            .collect()
    }

    /// Generate a non-cryptographically secure checksum of the file's content
    pub fn content_hash(&self) -> String {
        let mut hasher = md5::Md5::new();
        hasher.update(&self.content);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Returns all import statements of the file
    pub fn parse_imports(&self) -> Vec<&str> {
        utils::find_import_paths(self.as_ref())
    }
}

#[cfg(feature = "async")]
impl Source {
    /// async version of `Self::read`
    pub async fn async_read(file: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self { content: tokio::fs::read_to_string(file.as_ref()).await? })
    }

    /// Finds all source files under the given dir path and reads them all
    pub async fn async_read_all_from(dir: impl AsRef<Path>) -> io::Result<Sources> {
        Self::async_read_all(utils::source_files(dir.as_ref())?).await
    }

    /// async version of `Self::read_all`
    pub async fn async_read_all<T, I>(files: I) -> io::Result<Sources>
    where
        I: IntoIterator<Item = T>,
        T: Into<PathBuf>,
    {
        futures_util::future::join_all(
            files
                .into_iter()
                .map(Into::into)
                .map(|file| async { Self::async_read(&file).await.map(|source| (file, source)) }),
        )
        .await
        .into_iter()
        .collect()
    }
}

impl AsRef<str> for Source {
    fn as_ref(&self) -> &str {
        &self.content
    }
}

/// Output type `solc` produces
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct CompilerOutput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Error>,
    #[serde(default)]
    pub sources: BTreeMap<String, SourceFile>,
    #[serde(default)]
    pub contracts: BTreeMap<String, BTreeMap<String, Contract>>,
}

impl CompilerOutput {
    /// Whether the output contains an compiler error
    pub fn has_error(&self) -> bool {
        self.errors.iter().any(|err| err.severity.is_error())
    }

    pub fn diagnostics(&self) -> OutputDiagnostics {
        OutputDiagnostics(&self.errors)
    }
}

/// Helper type to implement display for solc errors
#[derive(Clone, Debug)]
pub struct OutputDiagnostics<'a>(&'a [Error]);

impl<'a> OutputDiagnostics<'a> {
    pub fn has_error(&self) -> bool {
        self.0.iter().any(|err| err.severity.is_error())
    }
}

impl<'a> fmt::Display for OutputDiagnostics<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.has_error() {
            f.write_str("Compiler run successful")?;
        }
        for err in self.0 {
            writeln!(f, "{}", err)?;
        }
        Ok(())
    }
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
    #[serde(default, rename = "storageLayout", skip_serializing_if = "StorageLayout::is_empty")]
    pub storage_layout: StorageLayout,
    /// EVM-related outputs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evm: Option<Evm>,
    /// Ewasm related outputs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ewasm: Option<Ewasm>,
}

/// Minimal representation of a contract's abi with bytecode
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct CompactContract {
    /// The Ethereum Contract ABI. If empty, it is represented as an empty
    /// array. See https://docs.soliditylang.org/en/develop/abi-spec.html
    pub abi: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bin: Option<String>,
    #[serde(default, rename = "bin-runtime", skip_serializing_if = "Option::is_none")]
    pub bin_runtime: Option<String>,
}

impl From<Contract> for CompactContract {
    fn from(c: Contract) -> Self {
        let (bin, bin_runtime) = if let Some(evm) = c.evm {
            (Some(evm.bytecode.object), evm.deployed_bytecode.bytecode.map(|evm| evm.object))
        } else {
            (None, None)
        };

        Self { abi: c.abi, bin, bin_runtime }
    }
}

/// Helper type to serialize while borrowing from `Contract`
#[derive(Clone, Debug, Serialize)]
pub struct CompactContractRef<'a> {
    pub abi: &'a [serde_json::Value],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bin: Option<&'a str>,
    #[serde(default, rename = "bin-runtime", skip_serializing_if = "Option::is_none")]
    pub bin_runtime: Option<&'a str>,
}

impl<'a> From<&'a Contract> for CompactContractRef<'a> {
    fn from(c: &'a Contract) -> Self {
        let (bin, bin_runtime) = if let Some(ref evm) = c.evm {
            (
                Some(evm.bytecode.object.as_str()),
                evm.deployed_bytecode.bytecode.as_ref().map(|evm| evm.object.as_str()),
            )
        } else {
            (None, None)
        };

        Self { abi: &c.abi, bin, bin_runtime }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct UserDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
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
    #[serde(default, rename = "custom:experimental", skip_serializing_if = "Option::is_none")]
    pub custom_experimental: Option<String>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
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
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub method_identifiers: BTreeMap<String, String>,
    /// Function gas estimates
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_estimates: Option<GasEstimates>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Bytecode {
    /// Debugging information at function level
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
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
    pub entry_point: Option<u32>,
    pub id: Option<u32>,
    pub parameter_slots: Option<u32>,
    pub return_slots: Option<u32>,
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
    pub severity: Severity,
    pub error_code: Option<String>,
    pub message: String,
    pub formatted_message: Option<String>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.severity.fmt(f)?;
        writeln!(f, ": {}", self.message)?;
        if let Some(msg) = &self.formatted_message {
            msg.as_str().yellow().fmt(f)?;
        }
        Ok(())
    }
}

// Error: No visibility specified. Did you intend to add "public"?
// --> /Users/Matthias/git/rust/ethers-rs/hh/contracts/Greeter2.sol:15:5:
// |
// 15 |     function greet()  view returns (string memory) {
// |     ^ (Relevant source part starts here and spans across multiple lines).

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => f.write_str(&"Error".red()),
            Severity::Warning => f.write_str(&"Warning".yellow()),
            Severity::Info => f.write_str("Info"),
        }
    }
}

impl Severity {
    pub fn is_error(&self) -> bool {
        matches!(self, Severity::Error)
    }

    pub fn is_warning(&self) -> bool {
        matches!(self, Severity::Warning)
    }

    pub fn is_info(&self) -> bool {
        matches!(self, Severity::Info)
    }
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "error" => Ok(Severity::Error),
            "warning" => Ok(Severity::Warning),
            "info" => Ok(Severity::Info),
            s => Err(format!("Invalid severity: {}", s)),
        }
    }
}

impl Serialize for Severity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Severity::Error => serializer.serialize_str("error"),
            Severity::Warning => serializer.serialize_str("warning"),
            Severity::Info => serializer.serialize_str("info"),
        }
    }
}

impl<'de> Deserialize<'de> for Severity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SeverityVisitor;

        impl<'de> Visitor<'de> for SeverityVisitor {
            type Value = Severity;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "severity string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value.parse().map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(SeverityVisitor)
    }
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
                panic!("Failed to read compiler output of {} {}", path.display(), err)
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
                panic!("Failed to read compiler output of {} {}", path.display(), err)
            });
        }
    }
}
