//! Solc artifact types
use ethers_core::{abi::Abi, types::Bytes};

use colored::Colorize;
use md5::Digest;
use semver::Version;
use std::{
    collections::{BTreeMap, HashSet},
    convert::TryFrom,
    fmt, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    compile::*,
    error::SolcIoError,
    remappings::Remapping,
    sourcemap::{self, SourceMap, SyntaxError},
    utils,
};
use ethers_core::abi::Address;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

pub mod output_selection;
pub mod serde_helpers;
use crate::{
    artifacts::output_selection::{ContractOutputSelection, OutputSelection},
    cache::FilteredSources,
};
pub use serde_helpers::{deserialize_bytes, deserialize_opt_bytes};

/// Solidity files are made up of multiple `source units`, a solidity contract is such a `source
/// unit`, therefore a solidity file can contain multiple contracts: (1-N*) relationship.
///
/// This types represents this mapping as `file name -> (contract name -> T)`, where the generic is
/// intended to represent contract specific information, like [`Contract`] itself, See [`Contracts`]
pub type FileToContractsMap<T> = BTreeMap<String, BTreeMap<String, T>>;

/// file -> (contract name -> Contract)
pub type Contracts = FileToContractsMap<Contract>;

/// An ordered list of files and their source
pub type Sources = BTreeMap<PathBuf, Source>;

/// A set of different Solc installations with their version and the sources to be compiled
pub type VersionedSources = BTreeMap<Solc, (Version, Sources)>;

/// A set of different Solc installations with their version and the sources to be compiled
pub type VersionedFilteredSources = BTreeMap<Solc, (Version, FilteredSources)>;

/// Input type `solc` expects
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompilerInput {
    pub language: String,
    pub sources: Sources,
    pub settings: Settings,
}

impl CompilerInput {
    /// Reads all contracts found under the path
    pub fn new(path: impl AsRef<Path>) -> Result<Vec<Self>, SolcIoError> {
        Source::read_all_from(path.as_ref()).map(Self::with_sources)
    }

    /// Creates a new [CompilerInput](s) with default settings and the given sources
    ///
    /// A [CompilerInput] expects a language setting, supported by solc are solidity or yul.
    /// In case the `sources` is a mix of solidity and yul files, 2 CompilerInputs are returned
    pub fn with_sources(sources: Sources) -> Vec<Self> {
        let mut solidity_sources = BTreeMap::new();
        let mut yul_sources = BTreeMap::new();
        for (path, source) in sources {
            if path.extension() == Some(std::ffi::OsStr::new("yul")) {
                yul_sources.insert(path, source);
            } else {
                solidity_sources.insert(path, source);
            }
        }
        let mut res = Vec::new();
        if !solidity_sources.is_empty() {
            res.push(Self {
                language: "Solidity".to_string(),
                sources: solidity_sources,
                settings: Default::default(),
            });
        }
        if !yul_sources.is_empty() {
            res.push(Self {
                language: "Yul".to_string(),
                sources: yul_sources,
                settings: Default::default(),
            });
        }
        res
    }

    /// Sets the settings for compilation
    #[must_use]
    pub fn settings(mut self, settings: Settings) -> Self {
        self.settings = settings;
        self
    }

    /// Sets the EVM version for compilation
    #[must_use]
    pub fn evm_version(mut self, version: EvmVersion) -> Self {
        self.settings.evm_version = Some(version);
        self
    }

    /// Sets the optimizer runs (default = 200)
    #[must_use]
    pub fn optimizer(mut self, runs: usize) -> Self {
        self.settings.optimizer.runs(runs);
        self
    }

    /// Normalizes the EVM version used in the settings to be up to the latest one
    /// supported by the provided compiler version.
    #[must_use]
    pub fn normalize_evm_version(mut self, version: &Version) -> Self {
        if let Some(ref mut evm_version) = self.settings.evm_version {
            self.settings.evm_version = evm_version.normalize_version(version);
        }
        self
    }

    #[must_use]
    pub fn with_remappings(mut self, remappings: Vec<Remapping>) -> Self {
        self.settings.remappings = remappings;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Stop compilation after the given stage.
    /// since 0.8.11: only "parsing" is valid here
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_after: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remappings: Vec<Remapping>,
    pub optimizer: Optimizer,
    /// Metadata settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SettingsMetadata>,
    /// This field can be used to select desired outputs based
    /// on file and contract names.
    /// If this field is omitted, then the compiler loads and does type
    /// checking, but will not generate any outputs apart from errors.
    #[serde(default)]
    pub output_selection: OutputSelection,
    #[serde(
        default,
        with = "serde_helpers::display_from_str_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub evm_version: Option<EvmVersion>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub libraries: BTreeMap<String, BTreeMap<String, String>>,
}

impl Settings {
    /// Creates a new `Settings` instance with the given `output_selection`
    pub fn new(output_selection: impl Into<OutputSelection>) -> Self {
        Self { output_selection: output_selection.into(), ..Default::default() }
    }

    /// Inserts a set of `ContractOutputSelection`
    pub fn push_all(&mut self, settings: impl IntoIterator<Item = ContractOutputSelection>) {
        for value in settings {
            self.push_output_selection(value)
        }
    }

    /// Inserts a set of `ContractOutputSelection`
    #[must_use]
    pub fn with_extra_output(
        mut self,
        settings: impl IntoIterator<Item = ContractOutputSelection>,
    ) -> Self {
        for value in settings {
            self.push_output_selection(value)
        }
        self
    }

    /// Inserts the value for all files and contracts
    ///
    /// ```
    /// use ethers_solc::artifacts::output_selection::ContractOutputSelection;
    /// use ethers_solc::artifacts::Settings;
    /// let mut selection = Settings::default();
    /// selection.push_output_selection(ContractOutputSelection::Metadata);
    /// ```
    pub fn push_output_selection(&mut self, value: impl ToString) {
        self.push_contract_output_selection("*", value)
    }

    /// Inserts the `key` `value` pair to the `output_selection` for all files
    ///
    /// If the `key` already exists, then the value is added to the existing list
    pub fn push_contract_output_selection(
        &mut self,
        contracts: impl Into<String>,
        value: impl ToString,
    ) {
        let value = value.to_string();
        let values = self
            .output_selection
            .as_mut()
            .entry("*".to_string())
            .or_default()
            .entry(contracts.into())
            .or_default();
        if !values.contains(&value) {
            values.push(value)
        }
    }

    /// Sets the value for all files and contracts
    pub fn set_output_selection(&mut self, values: impl IntoIterator<Item = impl ToString>) {
        self.set_contract_output_selection("*", values)
    }

    /// Sets the `key` to the `values` pair to the `output_selection` for all files
    ///
    /// This will replace the existing values for `key` if they're present
    pub fn set_contract_output_selection(
        &mut self,
        key: impl Into<String>,
        values: impl IntoIterator<Item = impl ToString>,
    ) {
        self.output_selection
            .as_mut()
            .entry("*".to_string())
            .or_default()
            .insert(key.into(), values.into_iter().map(|s| s.to_string()).collect());
    }

    /// Adds `ast` to output
    #[must_use]
    pub fn with_ast(mut self) -> Self {
        let output =
            self.output_selection.as_mut().entry("*".to_string()).or_insert_with(BTreeMap::default);
        output.insert("".to_string(), vec!["ast".to_string()]);
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            stop_after: None,
            optimizer: Default::default(),
            metadata: None,
            output_selection: OutputSelection::default_output_selection(),
            evm_version: Some(EvmVersion::default()),
            libraries: Default::default(),
            remappings: Default::default(),
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
    /// Switch optimizer components on or off in detail.
    /// The "enabled" switch above provides two defaults which can be
    /// tweaked here. If "details" is given, "enabled" can be omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<OptimizerDetails>,
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
        Self { enabled: Some(false), runs: Some(200), details: None }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptimizerDetails {
    /// The peephole optimizer is always on if no details are given,
    /// use details to switch it off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peephole: Option<bool>,
    /// The inliner is always on if no details are given,
    /// use details to switch it off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inliner: Option<bool>,
    /// The unused jumpdest remover is always on if no details are given,
    /// use details to switch it off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jumpdest_remover: Option<bool>,
    /// Sometimes re-orders literals in commutative operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_literals: Option<bool>,
    /// Removes duplicate code blocks
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deduplicate: Option<bool>,
    /// Common subexpression elimination, this is the most complicated step but
    /// can also provide the largest gain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cse: Option<bool>,
    /// Optimize representation of literal numbers and strings in code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constant_optimizer: Option<bool>,
    /// The new Yul optimizer. Mostly operates on the code of ABI coder v2
    /// and inline assembly.
    /// It is activated together with the global optimizer setting
    /// and can be deactivated here.
    /// Before Solidity 0.6.0 it had to be activated through this switch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yul: Option<bool>,
    /// Tuning options for the Yul optimizer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yul_details: Option<YulDetails>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct YulDetails {
    /// Improve allocation of stack slots for variables, can free up stack slots early.
    /// Activated by default if the Yul optimizer is activated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_allocation: Option<bool>,
    /// Select optimization steps to be applied.
    /// Optional, the optimizer will use the default sequence if omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optimizer_steps: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvmVersion {
    Homestead,
    TangerineWhistle,
    SpuriousDragon,
    Byzantium,
    Constantinople,
    Petersburg,
    Istanbul,
    Berlin,
    London,
}

impl Default for EvmVersion {
    fn default() -> Self {
        Self::London
    }
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
            EvmVersion::SpuriousDragon => "spuriousDragon",
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
            "spuriousDragon" => Ok(EvmVersion::SpuriousDragon),
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
pub struct SettingsMetadata {
    /// Use only literal content and not URLs (false by default)
    #[serde(default, rename = "useLiteralContent", skip_serializing_if = "Option::is_none")]
    pub use_literal_content: Option<bool>,
    /// Use the given hash method for the metadata hash that is appended to the bytecode.
    /// The metadata hash can be removed from the bytecode via option "none".
    /// The other options are "ipfs" and "bzzr1".
    /// If the option is omitted, "ipfs" is used by default.
    #[serde(default, rename = "bytecodeHash", skip_serializing_if = "Option::is_none")]
    pub bytecode_hash: Option<String>,
}

/// Bindings for [`solc` contract metadata](https://docs.soliditylang.org/en/latest/metadata.html)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    pub compiler: Compiler,
    pub language: String,
    pub output: Output,
    pub settings: MetadataSettings,
    pub sources: MetadataSources,
    pub version: i64,
}

/// Compiler settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataSettings {
    /// Required for Solidity: File and name of the contract or library this metadata is created
    /// for.
    #[serde(default, rename = "compilationTarget")]
    pub compilation_target: BTreeMap<String, String>,
    #[serde(flatten)]
    pub inner: Settings,
}

/// Compilation source files/source units, keys are file names
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataSources {
    #[serde(flatten)]
    pub inner: BTreeMap<String, MetadataSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataSource {
    /// Required: keccak256 hash of the source file
    pub keccak256: String,
    /// Required (unless "content" is used, see below): Sorted URL(s)
    /// to the source file, protocol is more or less arbitrary, but a
    /// Swarm URL is recommended
    #[serde(default)]
    pub urls: Vec<String>,
    /// Required (unless "url" is used): literal contents of the source file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Optional: SPDX license identifier as given in the source file
    pub license: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compiler {
    pub version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub abi: Vec<SolcAbi>,
    pub devdoc: Option<Doc>,
    pub userdoc: Option<Doc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolcAbi {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<Item>,
    #[serde(rename = "stateMutability")]
    pub state_mutability: Option<String>,
    #[serde(rename = "type")]
    pub abi_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<Item>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "internalType")]
    pub internal_type: String,
    pub name: String,
    #[serde(rename = "type")]
    pub put_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Doc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub methods: Option<Libraries>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Libraries {
    #[serde(flatten)]
    pub libs: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Source {
    pub content: String,
}

impl Source {
    /// this is a heuristically measured threshold at which we can generally expect a speedup by
    /// using rayon's `par_iter`, See `Self::read_all_files`
    pub const NUM_READ_PAR: usize = 8;

    /// Reads the file content
    pub fn read(file: impl AsRef<Path>) -> Result<Self, SolcIoError> {
        let file = file.as_ref();
        Ok(Self { content: fs::read_to_string(file).map_err(|err| SolcIoError::new(err, file))? })
    }

    /// Recursively finds all source files under the given dir path and reads them all
    pub fn read_all_from(dir: impl AsRef<Path>) -> Result<Sources, SolcIoError> {
        Self::read_all_files(utils::source_files(dir))
    }

    /// Reads all source files of the given vec
    ///
    /// Depending on the len of the vec it will try to read the files in parallel
    pub fn read_all_files(files: Vec<PathBuf>) -> Result<Sources, SolcIoError> {
        use rayon::prelude::*;

        if files.len() < Self::NUM_READ_PAR {
            Self::read_all(files)
        } else {
            files
                .par_iter()
                .map(Into::into)
                .map(|file| Self::read(&file).map(|source| (file, source)))
                .collect()
        }
    }

    /// Reads all files
    pub fn read_all<T, I>(files: I) -> Result<Sources, SolcIoError>
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

    /// Parallelized version of `Self::read_all` that reads all files using a parallel iterator
    ///
    /// NOTE: this is only expected to be faster than `Self::read_all` if the given iterator
    /// contains at least several paths. see also `Self::read_all_files`.
    pub fn par_read_all<T, I>(files: I) -> Result<Sources, SolcIoError>
    where
        I: IntoIterator<Item = T>,
        <I as IntoIterator>::IntoIter: Send,
        T: Into<PathBuf> + Send,
    {
        use rayon::{iter::ParallelBridge, prelude::ParallelIterator};
        files
            .into_iter()
            .par_bridge()
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
        utils::find_import_paths(self.as_ref()).map(|m| m.as_str()).collect()
    }
}

#[cfg(feature = "async")]
impl Source {
    /// async version of `Self::read`
    pub async fn async_read(file: impl AsRef<Path>) -> Result<Self, SolcIoError> {
        let file = file.as_ref();
        Ok(Self {
            content: tokio::fs::read_to_string(file)
                .await
                .map_err(|err| SolcIoError::new(err, file))?,
        })
    }

    /// Finds all source files under the given dir path and reads them all
    pub async fn async_read_all_from(dir: impl AsRef<Path>) -> Result<Sources, SolcIoError> {
        Self::async_read_all(utils::source_files(dir.as_ref())).await
    }

    /// async version of `Self::read_all`
    pub async fn async_read_all<T, I>(files: I) -> Result<Sources, SolcIoError>
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct CompilerOutput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Error>,
    #[serde(default)]
    pub sources: BTreeMap<String, SourceFile>,
    #[serde(default)]
    pub contracts: Contracts,
}

impl CompilerOutput {
    /// Whether the output contains a compiler error
    pub fn has_error(&self) -> bool {
        self.errors.iter().any(|err| err.severity.is_error())
    }

    /// Whether the output contains a compiler warning
    pub fn has_warning(&self, ignored_error_codes: &[u64]) -> bool {
        self.errors.iter().any(|err| {
            if err.severity.is_warning() {
                err.error_code.as_ref().map_or(false, |code| !ignored_error_codes.contains(code))
            } else {
                false
            }
        })
    }

    /// Finds the _first_ contract with the given name
    pub fn find(&self, contract: impl AsRef<str>) -> Option<CompactContractRef> {
        let contract_name = contract.as_ref();
        self.contracts_iter().find_map(|(name, contract)| {
            (name == contract_name).then(|| CompactContractRef::from(contract))
        })
    }

    /// Finds the first contract with the given name and removes it from the set
    pub fn remove(&mut self, contract: impl AsRef<str>) -> Option<Contract> {
        let contract_name = contract.as_ref();
        self.contracts.values_mut().find_map(|c| c.remove(contract_name))
    }

    /// Iterate over all contracts and their names
    pub fn contracts_iter(&self) -> impl Iterator<Item = (&String, &Contract)> {
        self.contracts.values().flatten()
    }

    /// Iterate over all contracts and their names
    pub fn contracts_into_iter(self) -> impl Iterator<Item = (String, Contract)> {
        self.contracts.into_values().flatten()
    }

    /// Given the contract file's path and the contract's name, tries to return the contract's
    /// bytecode, runtime bytecode, and abi
    pub fn get(&self, path: &str, contract: &str) -> Option<CompactContractRef> {
        self.contracts
            .get(path)
            .and_then(|contracts| contracts.get(contract))
            .map(CompactContractRef::from)
    }

    /// Returns the output's source files and contracts separately, wrapped in helper types that
    /// provide several helper methods
    pub fn split(self) -> (SourceFiles, OutputContracts) {
        (SourceFiles(self.sources), OutputContracts(self.contracts))
    }

    /// Retains only those files the given iterator yields
    ///
    /// In other words, removes all contracts for files not included in the iterator
    pub fn retain_files<'a, I>(&mut self, files: I)
    where
        I: IntoIterator<Item = &'a str>,
    {
        let files: HashSet<_> = files.into_iter().collect();

        self.contracts.retain(|f, _| files.contains(f.as_str()));
        self.sources.retain(|f, _| files.contains(f.as_str()));
        self.errors.retain(|err| {
            err.source_location.as_ref().map(|s| files.contains(s.file.as_str())).unwrap_or(true)
        });
    }

    pub fn merge(&mut self, other: CompilerOutput) {
        self.errors.extend(other.errors);
        self.contracts.extend(other.contracts);
        self.sources.extend(other.sources);
    }
}

/// A wrapper helper type for the `Contracts` type alias
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OutputContracts(pub Contracts);

impl OutputContracts {
    /// Returns an iterator over all contracts and their source names.
    pub fn into_contracts(self) -> impl Iterator<Item = (String, Contract)> {
        self.0.into_values().flatten()
    }

    /// Iterate over all contracts and their names
    pub fn contracts_iter(&self) -> impl Iterator<Item = (&String, &Contract)> {
        self.0.values().flatten()
    }

    /// Finds the _first_ contract with the given name
    pub fn find(&self, contract: impl AsRef<str>) -> Option<CompactContractRef> {
        let contract_name = contract.as_ref();
        self.contracts_iter().find_map(|(name, contract)| {
            (name == contract_name).then(|| CompactContractRef::from(contract))
        })
    }

    /// Finds the first contract with the given name and removes it from the set
    pub fn remove(&mut self, contract: impl AsRef<str>) -> Option<Contract> {
        let contract_name = contract.as_ref();
        self.0.values_mut().find_map(|c| c.remove(contract_name))
    }
}

/// Represents a compiled solidity contract
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contract {
    /// The Ethereum Contract Metadata.
    /// See https://docs.soliditylang.org/en/develop/metadata.html
    pub abi: Option<LosslessAbi>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_helpers::json_string_opt"
    )]
    pub metadata: Option<Metadata>,
    #[serde(default)]
    pub userdoc: UserDoc,
    #[serde(default)]
    pub devdoc: DevDoc,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ir: Option<String>,
    #[serde(default, skip_serializing_if = "StorageLayout::is_empty")]
    pub storage_layout: StorageLayout,
    /// EVM-related outputs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evm: Option<Evm>,
    /// Ewasm related outputs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ewasm: Option<Ewasm>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ir_optimized: Option<String>,
}

/// A helper type that ensures lossless (de)serialisation unlike [`ethabi::Contract`] which omits
/// some information of (nested) components in a serde roundtrip. This is a problem for
/// abienconderv2 structs because `ethabi::Contract`'s representation of those are [`ethabi::Param`]
/// and the `kind` field of type [`ethabi::ParamType`] does not support deeply nested components as
/// it's the case for structs. This is not easily fixable in ethabi as it would require a redesign
/// of the overall `Param` and `ParamType` types. Instead, this type keeps a copy of the
/// [`serde_json::Value`] when deserialized from the `solc` json compiler output and uses it to
/// serialize the `abi` without loss.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct LosslessAbi {
    /// The complete abi as json value
    pub abi_value: serde_json::Value,
    /// The deserialised version of `abi_value`
    pub abi: Abi,
}

impl From<LosslessAbi> for Abi {
    fn from(abi: LosslessAbi) -> Self {
        abi.abi
    }
}

impl Serialize for LosslessAbi {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.abi_value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LosslessAbi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let abi_value = serde_json::Value::deserialize(deserializer)?;
        let abi = serde_json::from_value(abi_value.clone()).map_err(serde::de::Error::custom)?;
        Ok(Self { abi_value, abi })
    }
}

/// Minimal representation of a contract with a present abi and bytecode.
///
/// Unlike `CompactContractSome` which contains the `BytecodeObject`, this holds the whole
/// `Bytecode` object.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContractBytecode {
    /// The Ethereum Contract ABI. If empty, it is represented as an empty
    /// array. See https://docs.soliditylang.org/en/develop/abi-spec.html
    pub abi: Option<Abi>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytecode: Option<Bytecode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployed_bytecode: Option<DeployedBytecode>,
}

impl ContractBytecode {
    /// Returns the `ContractBytecodeSome` if all fields are `Some`
    ///
    /// # Panics
    ///
    /// Panics if any of the fields euqal `None`
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let mut output = project.compile().unwrap().output();
    /// let contract: ContractBytecode = output.remove("Greeter").unwrap().into();
    /// let contract = contract.unwrap();
    /// # }
    /// ```
    pub fn unwrap(self) -> ContractBytecodeSome {
        ContractBytecodeSome {
            abi: self.abi.unwrap(),
            bytecode: self.bytecode.unwrap(),
            deployed_bytecode: self.deployed_bytecode.unwrap(),
        }
    }

    /// Looks for all link references in deployment and runtime bytecodes
    pub fn all_link_references(&self) -> BTreeMap<String, BTreeMap<String, Vec<Offsets>>> {
        let mut links = BTreeMap::new();
        if let Some(bcode) = &self.bytecode {
            links.extend(bcode.link_references.clone());
        }

        if let Some(d_bcode) = &self.deployed_bytecode {
            if let Some(bcode) = &d_bcode.bytecode {
                links.extend(bcode.link_references.clone());
            }
        }
        links
    }
}

impl From<Contract> for ContractBytecode {
    fn from(c: Contract) -> Self {
        let (bytecode, deployed_bytecode) = if let Some(evm) = c.evm {
            (evm.bytecode, evm.deployed_bytecode)
        } else {
            (None, None)
        };

        Self { abi: c.abi.map(Into::into), bytecode, deployed_bytecode }
    }
}

/// Minimal representation of a contract with a present abi and bytecode.
///
/// Unlike `CompactContractSome` which contains the `BytecodeObject`, this holds the whole
/// `Bytecode` object.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompactContractBytecode {
    /// The Ethereum Contract ABI. If empty, it is represented as an empty
    /// array. See https://docs.soliditylang.org/en/develop/abi-spec.html
    pub abi: Option<Abi>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytecode: Option<CompactBytecode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployed_bytecode: Option<CompactDeployedBytecode>,
}

impl CompactContractBytecode {
    /// Looks for all link references in deployment and runtime bytecodes
    pub fn all_link_references(&self) -> BTreeMap<String, BTreeMap<String, Vec<Offsets>>> {
        let mut links = BTreeMap::new();
        if let Some(bcode) = &self.bytecode {
            links.extend(bcode.link_references.clone());
        }

        if let Some(d_bcode) = &self.deployed_bytecode {
            if let Some(bcode) = &d_bcode.bytecode {
                links.extend(bcode.link_references.clone());
            }
        }
        links
    }
}

impl From<Contract> for CompactContractBytecode {
    fn from(c: Contract) -> Self {
        let (bytecode, deployed_bytecode) = if let Some(evm) = c.evm {
            let evm = evm.into_compact();
            (evm.bytecode, evm.deployed_bytecode)
        } else {
            (None, None)
        };

        Self { abi: c.abi.map(Into::into), bytecode, deployed_bytecode }
    }
}

impl From<ContractBytecode> for CompactContractBytecode {
    fn from(c: ContractBytecode) -> Self {
        let (maybe_bcode, maybe_runtime) = match (c.bytecode, c.deployed_bytecode) {
            (Some(bcode), Some(dbcode)) => (Some(bcode.into()), Some(dbcode.into())),
            (None, Some(dbcode)) => (None, Some(dbcode.into())),
            (Some(bcode), None) => (Some(bcode.into()), None),
            (None, None) => (None, None),
        };
        Self { abi: c.abi, bytecode: maybe_bcode, deployed_bytecode: maybe_runtime }
    }
}

impl From<CompactContractBytecode> for ContractBytecode {
    fn from(c: CompactContractBytecode) -> Self {
        let (maybe_bcode, maybe_runtime) = match (c.bytecode, c.deployed_bytecode) {
            (Some(bcode), Some(dbcode)) => (Some(bcode.into()), Some(dbcode.into())),
            (None, Some(dbcode)) => (None, Some(dbcode.into())),
            (Some(bcode), None) => (Some(bcode.into()), None),
            (None, None) => (None, None),
        };
        Self { abi: c.abi, bytecode: maybe_bcode, deployed_bytecode: maybe_runtime }
    }
}

/// Minimal representation of a contract with a present abi and bytecode.
///
/// Unlike `CompactContractSome` which contains the `BytecodeObject`, this holds the whole
/// `Bytecode` object.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContractBytecodeSome {
    pub abi: Abi,
    pub bytecode: Bytecode,
    pub deployed_bytecode: DeployedBytecode,
}

impl TryFrom<ContractBytecode> for ContractBytecodeSome {
    type Error = ContractBytecode;

    fn try_from(value: ContractBytecode) -> Result<Self, Self::Error> {
        if value.abi.is_none() || value.bytecode.is_none() || value.deployed_bytecode.is_none() {
            return Err(value)
        }
        Ok(value.unwrap())
    }
}

/// Minimal representation of a contract's artifact with a present abi and bytecode.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct CompactContractSome {
    /// The Ethereum Contract ABI. If empty, it is represented as an empty
    /// array. See https://docs.soliditylang.org/en/develop/abi-spec.html
    pub abi: Abi,
    pub bin: BytecodeObject,
    #[serde(rename = "bin-runtime")]
    pub bin_runtime: BytecodeObject,
}

impl TryFrom<CompactContract> for CompactContractSome {
    type Error = CompactContract;

    fn try_from(value: CompactContract) -> Result<Self, Self::Error> {
        if value.abi.is_none() || value.bin.is_none() || value.bin_runtime.is_none() {
            return Err(value)
        }
        Ok(value.unwrap())
    }
}

/// The general purpose minimal representation of a contract's abi with bytecode
/// Unlike `CompactContractSome` all fields are optional so that every possible compiler output can
/// be represented by it
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct CompactContract {
    /// The Ethereum Contract ABI. If empty, it is represented as an empty
    /// array. See https://docs.soliditylang.org/en/develop/abi-spec.html
    pub abi: Option<Abi>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bin: Option<BytecodeObject>,
    #[serde(default, rename = "bin-runtime", skip_serializing_if = "Option::is_none")]
    pub bin_runtime: Option<BytecodeObject>,
}

impl CompactContract {
    /// Returns the contents of this type as a single tuple of abi, bytecode and deployed bytecode
    pub fn into_parts(self) -> (Option<Abi>, Option<Bytes>, Option<Bytes>) {
        (
            self.abi,
            self.bin.and_then(|bin| bin.into_bytes()),
            self.bin_runtime.and_then(|bin| bin.into_bytes()),
        )
    }

    /// Returns the individual parts of this contract.
    ///
    /// If the values are `None`, then `Default` is returned.
    pub fn into_parts_or_default(self) -> (Abi, Bytes, Bytes) {
        (
            self.abi.unwrap_or_default(),
            self.bin.and_then(|bin| bin.into_bytes()).unwrap_or_default(),
            self.bin_runtime.and_then(|bin| bin.into_bytes()).unwrap_or_default(),
        )
    }

    /// Returns the `CompactContractSome` if all fields are `Some`
    ///
    /// # Panics
    ///
    /// Panics if any of the fields euqal `None`
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let mut output = project.compile().unwrap().output();
    /// let contract: CompactContract = output.remove("Greeter").unwrap().into();
    /// let contract = contract.unwrap();
    /// # }
    /// ```
    pub fn unwrap(self) -> CompactContractSome {
        CompactContractSome {
            abi: self.abi.unwrap(),
            bin: self.bin.unwrap(),
            bin_runtime: self.bin_runtime.unwrap(),
        }
    }

    /// Returns the `CompactContractSome` if any if the field equals `None` the `Default` value is
    /// returned
    ///
    /// Unlike `unwrap`, this function does _not_ panic
    pub fn unwrap_or_default(self) -> CompactContractSome {
        CompactContractSome {
            abi: self.abi.unwrap_or_default(),
            bin: self.bin.unwrap_or_default(),
            bin_runtime: self.bin_runtime.unwrap_or_default(),
        }
    }
}

impl From<serde_json::Value> for CompactContract {
    fn from(mut val: serde_json::Value) -> Self {
        if let Some(map) = val.as_object_mut() {
            let abi = map.remove("abi").and_then(|val| serde_json::from_value(val).ok());
            let bin = map.remove("bin").and_then(|val| serde_json::from_value(val).ok());
            let bin_runtime =
                map.remove("bin-runtime").and_then(|val| serde_json::from_value(val).ok());
            Self { abi, bin, bin_runtime }
        } else {
            CompactContract::default()
        }
    }
}

impl From<serde_json::Value> for CompactContractBytecode {
    fn from(val: serde_json::Value) -> Self {
        serde_json::from_value(val).unwrap_or_default()
    }
}

impl From<ContractBytecode> for CompactContract {
    fn from(c: ContractBytecode) -> Self {
        let ContractBytecode { abi, bytecode, deployed_bytecode } = c;
        Self {
            abi,
            bin: bytecode.map(|c| c.object),
            bin_runtime: deployed_bytecode
                .and_then(|deployed| deployed.bytecode.map(|code| code.object)),
        }
    }
}

impl From<CompactContractBytecode> for CompactContract {
    fn from(c: CompactContractBytecode) -> Self {
        let c: ContractBytecode = c.into();
        c.into()
    }
}

impl From<ContractBytecodeSome> for CompactContract {
    fn from(c: ContractBytecodeSome) -> Self {
        Self {
            abi: Some(c.abi),
            bin: Some(c.bytecode.object),
            bin_runtime: c.deployed_bytecode.bytecode.map(|code| code.object),
        }
    }
}

impl From<Contract> for CompactContract {
    fn from(c: Contract) -> Self {
        ContractBytecode::from(c).into()
    }
}

impl From<CompactContractSome> for CompactContract {
    fn from(c: CompactContractSome) -> Self {
        Self { abi: Some(c.abi), bin: Some(c.bin), bin_runtime: Some(c.bin_runtime) }
    }
}

impl<'a> From<CompactContractRef<'a>> for CompactContract {
    fn from(c: CompactContractRef<'a>) -> Self {
        Self { abi: c.abi.cloned(), bin: c.bin.cloned(), bin_runtime: c.bin_runtime.cloned() }
    }
}

impl<'a> From<CompactContractRefSome<'a>> for CompactContract {
    fn from(c: CompactContractRefSome<'a>) -> Self {
        Self {
            abi: Some(c.abi.clone()),
            bin: Some(c.bin.clone()),
            bin_runtime: Some(c.bin_runtime.clone()),
        }
    }
}

/// Minimal representation of a contract with a present abi and bytecode that borrows.
#[derive(Copy, Clone, Debug, Serialize)]
pub struct CompactContractRefSome<'a> {
    pub abi: &'a Abi,
    pub bin: &'a BytecodeObject,
    #[serde(rename = "bin-runtime")]
    pub bin_runtime: &'a BytecodeObject,
}

impl<'a> CompactContractRefSome<'a> {
    /// Returns the individual parts of this contract.
    ///
    /// If the values are `None`, then `Default` is returned.
    pub fn into_parts(self) -> (Abi, Bytes, Bytes) {
        CompactContract::from(self).into_parts_or_default()
    }
}

impl<'a> TryFrom<CompactContractRef<'a>> for CompactContractRefSome<'a> {
    type Error = CompactContractRef<'a>;

    fn try_from(value: CompactContractRef<'a>) -> Result<Self, Self::Error> {
        if value.abi.is_none() || value.bin.is_none() || value.bin_runtime.is_none() {
            return Err(value)
        }
        Ok(value.unwrap())
    }
}

/// Helper type to serialize while borrowing from `Contract`
#[derive(Copy, Clone, Debug, Serialize)]
pub struct CompactContractRef<'a> {
    pub abi: Option<&'a Abi>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bin: Option<&'a BytecodeObject>,
    #[serde(default, rename = "bin-runtime", skip_serializing_if = "Option::is_none")]
    pub bin_runtime: Option<&'a BytecodeObject>,
}

impl<'a> CompactContractRef<'a> {
    /// Clones the referenced values and returns as tuples
    pub fn into_parts(self) -> (Option<Abi>, Option<Bytes>, Option<Bytes>) {
        CompactContract::from(self).into_parts()
    }

    /// Returns the individual parts of this contract.
    ///
    /// If the values are `None`, then `Default` is returned.
    pub fn into_parts_or_default(self) -> (Abi, Bytes, Bytes) {
        CompactContract::from(self).into_parts_or_default()
    }

    pub fn bytecode(&self) -> Option<&Bytes> {
        self.bin.as_ref().and_then(|bin| bin.as_bytes())
    }

    pub fn runtime_bytecode(&self) -> Option<&Bytes> {
        self.bin_runtime.as_ref().and_then(|bin| bin.as_bytes())
    }

    /// Returns the `CompactContractRefSome` if all fields are `Some`
    ///
    /// # Panics
    ///
    /// Panics if any of the fields equal `None`
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let output = project.compile().unwrap().output();
    /// let contract = output.find("Greeter").unwrap();
    /// let contract = contract.unwrap();
    /// # }
    /// ```
    pub fn unwrap(self) -> CompactContractRefSome<'a> {
        CompactContractRefSome {
            abi: self.abi.unwrap(),
            bin: self.bin.unwrap(),
            bin_runtime: self.bin_runtime.unwrap(),
        }
    }
}

impl<'a> From<&'a Contract> for CompactContractRef<'a> {
    fn from(c: &'a Contract) -> Self {
        let (bin, bin_runtime) = if let Some(ref evm) = c.evm {
            (
                evm.bytecode.as_ref().map(|c| &c.object),
                evm.deployed_bytecode
                    .as_ref()
                    .and_then(|deployed| deployed.bytecode.as_ref().map(|evm| &evm.object)),
            )
        } else {
            (None, None)
        };

        Self { abi: c.abi.as_ref().map(|abi| &abi.abi), bin, bin_runtime }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct UserDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub methods: BTreeMap<String, BTreeMap<String, String>>,
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
    pub methods: BTreeMap<String, MethodDoc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct MethodDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#return: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Evm {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembly: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_assembly: Option<serde_json::Value>,
    pub bytecode: Option<Bytecode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployed_bytecode: Option<DeployedBytecode>,
    /// The list of function hashes
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub method_identifiers: BTreeMap<String, String>,
    /// Function gas estimates
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_estimates: Option<GasEstimates>,
}

impl Evm {
    /// Crate internal helper do transform the underlying bytecode artifacts into a more convenient
    /// structure
    pub(crate) fn into_compact(self) -> CompactEvm {
        let Evm {
            assembly,
            legacy_assembly,
            bytecode,
            deployed_bytecode,
            method_identifiers,
            gas_estimates,
        } = self;

        let (bytecode, deployed_bytecode) = match (bytecode, deployed_bytecode) {
            (Some(bcode), Some(dbcode)) => (Some(bcode.into()), Some(dbcode.into())),
            (None, Some(dbcode)) => (None, Some(dbcode.into())),
            (Some(bcode), None) => (Some(bcode.into()), None),
            (None, None) => (None, None),
        };

        CompactEvm {
            assembly,
            legacy_assembly,
            bytecode,
            deployed_bytecode,
            method_identifiers,
            gas_estimates,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CompactEvm {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembly: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_assembly: Option<serde_json::Value>,
    pub bytecode: Option<CompactBytecode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployed_bytecode: Option<CompactDeployedBytecode>,
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
    pub object: BytecodeObject,
    /// Opcodes list (string)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opcodes: Option<String>,
    /// The source mapping as a string. See the source mapping definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_map: Option<String>,
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
pub struct CompactBytecode {
    /// The bytecode as a hex string.
    pub object: BytecodeObject,
    /// The source mapping as a string. See the source mapping definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_map: Option<String>,
    /// If given, this is an unlinked object.
    #[serde(default)]
    pub link_references: BTreeMap<String, BTreeMap<String, Vec<Offsets>>>,
}

impl CompactBytecode {
    /// Tries to link the bytecode object with the `file` and `library` name.
    /// Replaces all library placeholders with the given address.
    ///
    /// Returns true if the bytecode object is fully linked, false otherwise
    /// This is a noop if the bytecode object is already fully linked.
    pub fn link(
        &mut self,
        file: impl AsRef<str>,
        library: impl AsRef<str>,
        address: Address,
    ) -> bool {
        if !self.object.is_unlinked() {
            return true
        }

        let file = file.as_ref();
        let library = library.as_ref();
        if let Some((key, mut contracts)) = self.link_references.remove_entry(file) {
            if contracts.remove(library).is_some() {
                self.object.link(file, library, address);
            }
            if !contracts.is_empty() {
                self.link_references.insert(key, contracts);
            }
            if self.link_references.is_empty() {
                return self.object.resolve().is_some()
            }
        }
        false
    }
}

impl From<Bytecode> for CompactBytecode {
    fn from(bcode: Bytecode) -> CompactBytecode {
        CompactBytecode {
            object: bcode.object,
            source_map: bcode.source_map,
            link_references: bcode.link_references,
        }
    }
}

impl From<CompactBytecode> for Bytecode {
    fn from(bcode: CompactBytecode) -> Bytecode {
        Bytecode {
            object: bcode.object,
            source_map: bcode.source_map,
            link_references: bcode.link_references,
            function_debug_data: Default::default(),
            opcodes: Default::default(),
            generated_sources: Default::default(),
        }
    }
}

impl From<BytecodeObject> for Bytecode {
    fn from(object: BytecodeObject) -> Bytecode {
        Bytecode {
            object,
            function_debug_data: Default::default(),
            opcodes: Default::default(),
            source_map: Default::default(),
            generated_sources: Default::default(),
            link_references: Default::default(),
        }
    }
}

impl Bytecode {
    /// Returns the parsed source map
    ///
    /// See also https://docs.soliditylang.org/en/v0.8.10/internals/source_mappings.html
    pub fn source_map(&self) -> Option<Result<SourceMap, SyntaxError>> {
        self.source_map.as_ref().map(|map| sourcemap::parse(map))
    }

    /// Same as `Bytecode::link` but with fully qualified name (`file.sol:Math`)
    pub fn link_fully_qualified(&mut self, name: impl AsRef<str>, addr: Address) -> bool {
        if let Some((file, lib)) = name.as_ref().split_once(':') {
            self.link(file, lib, addr)
        } else {
            false
        }
    }

    /// Tries to link the bytecode object with the `file` and `library` name.
    /// Replaces all library placeholders with the given address.
    ///
    /// Returns true if the bytecode object is fully linked, false otherwise
    /// This is a noop if the bytecode object is already fully linked.
    pub fn link(
        &mut self,
        file: impl AsRef<str>,
        library: impl AsRef<str>,
        address: Address,
    ) -> bool {
        if !self.object.is_unlinked() {
            return true
        }

        let file = file.as_ref();
        let library = library.as_ref();
        if let Some((key, mut contracts)) = self.link_references.remove_entry(file) {
            if contracts.remove(library).is_some() {
                self.object.link(file, library, address);
            }
            if !contracts.is_empty() {
                self.link_references.insert(key, contracts);
            }
            if self.link_references.is_empty() {
                return self.object.resolve().is_some()
            }
        }
        false
    }

    /// Links the bytecode object with all provided `(file, lib, addr)`
    pub fn link_all<I, S, T>(&mut self, libs: I) -> bool
    where
        I: IntoIterator<Item = (S, T, Address)>,
        S: AsRef<str>,
        T: AsRef<str>,
    {
        for (file, lib, addr) in libs.into_iter() {
            if self.link(file, lib, addr) {
                return true
            }
        }
        false
    }

    /// Links the bytecode object with all provided `(fully_qualified, addr)`
    pub fn link_all_fully_qualified<I, S>(&mut self, libs: I) -> bool
    where
        I: IntoIterator<Item = (S, Address)>,
        S: AsRef<str>,
    {
        for (name, addr) in libs.into_iter() {
            if self.link_fully_qualified(name, addr) {
                return true
            }
        }
        false
    }
}

/// Represents the bytecode of a contracts that might be not fully linked yet.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum BytecodeObject {
    /// Fully linked bytecode object
    #[serde(deserialize_with = "serde_helpers::deserialize_bytes")]
    Bytecode(Bytes),
    /// Bytecode as hex string that's not fully linked yet and contains library placeholders
    #[serde(with = "serde_helpers::string_bytes")]
    Unlinked(String),
}

impl BytecodeObject {
    /// Returns the underlying `Bytes` if the object is a valid bytecode, and not empty
    pub fn into_bytes(self) -> Option<Bytes> {
        match self {
            BytecodeObject::Bytecode(bytes) => Some(bytes),
            BytecodeObject::Unlinked(_) => None,
        }
    }

    /// Returns a reference to the underlying `Bytes` if the object is a valid bytecode, and not
    /// empty
    pub fn as_bytes(&self) -> Option<&Bytes> {
        match self {
            BytecodeObject::Bytecode(bytes) => Some(bytes),
            BytecodeObject::Unlinked(_) => None,
        }
    }
    /// Returns a reference to the underlying `String` if the object is unlinked
    pub fn as_str(&self) -> Option<&str> {
        match self {
            BytecodeObject::Bytecode(_) => None,
            BytecodeObject::Unlinked(s) => Some(s.as_str()),
        }
    }

    /// Returns the unlinked `String` if the object is unlinked or empty
    pub fn into_unlinked(self) -> Option<String> {
        match self {
            BytecodeObject::Bytecode(_) => None,
            BytecodeObject::Unlinked(code) => Some(code),
        }
    }

    /// Whether this object is still unlinked
    pub fn is_unlinked(&self) -> bool {
        matches!(self, BytecodeObject::Unlinked(_))
    }

    /// Whether this object a valid bytecode
    pub fn is_bytecode(&self) -> bool {
        matches!(self, BytecodeObject::Bytecode(_))
    }

    /// Returns `true` if the object is a valid bytecode and not empty.
    /// Returns false the object is a valid but empty bytecode or unlinked.
    pub fn is_non_empty_bytecode(&self) -> bool {
        self.as_bytes().map(|c| !c.0.is_empty()).unwrap_or_default()
    }

    /// Tries to resolve the unlinked string object a valid bytecode object in place
    ///
    /// Returns the string if it is a valid
    pub fn resolve(&mut self) -> Option<&Bytes> {
        if let BytecodeObject::Unlinked(unlinked) = self {
            if let Ok(linked) = hex::decode(unlinked) {
                *self = BytecodeObject::Bytecode(linked.into());
            }
        }
        self.as_bytes()
    }

    /// Link using the fully qualified name of a library
    /// The fully qualified library name is the path of its source file and the library name
    /// separated by `:` like `file.sol:Math`
    ///
    /// This will replace all occurrences of the library placeholder with the given address.
    ///
    /// See also: https://docs.soliditylang.org/en/develop/using-the-compiler.html#library-linking
    pub fn link_fully_qualified(&mut self, name: impl AsRef<str>, addr: Address) -> &mut Self {
        if let BytecodeObject::Unlinked(ref mut unlinked) = self {
            let name = name.as_ref();
            let place_holder = utils::library_hash_placeholder(name);
            // the address as hex without prefix
            let hex_addr = hex::encode(addr);

            // the library placeholder used to be the fully qualified name of the library instead of
            // the hash. This is also still supported by `solc` so we handle this as well
            let fully_qualified_placeholder = utils::library_fully_qualified_placeholder(name);

            *unlinked = unlinked
                .replace(&format!("__{}__", fully_qualified_placeholder), &hex_addr)
                .replace(&format!("__{}__", place_holder), &hex_addr)
        }
        self
    }

    /// Link using the `file` and `library` names as fully qualified name `<file>:<library>`
    /// See `BytecodeObject::link_fully_qualified`
    pub fn link(
        &mut self,
        file: impl AsRef<str>,
        library: impl AsRef<str>,
        addr: Address,
    ) -> &mut Self {
        self.link_fully_qualified(format!("{}:{}", file.as_ref(), library.as_ref(),), addr)
    }

    /// Links the bytecode object with all provided `(file, lib, addr)`
    pub fn link_all<I, S, T>(&mut self, libs: I) -> &mut Self
    where
        I: IntoIterator<Item = (S, T, Address)>,
        S: AsRef<str>,
        T: AsRef<str>,
    {
        for (file, lib, addr) in libs.into_iter() {
            self.link(file, lib, addr);
        }
        self
    }

    /// Whether the bytecode contains a matching placeholder using the qualified name
    pub fn contains_fully_qualified_placeholder(&self, name: impl AsRef<str>) -> bool {
        if let BytecodeObject::Unlinked(unlinked) = self {
            let name = name.as_ref();
            unlinked.contains(&utils::library_hash_placeholder(name)) ||
                unlinked.contains(&utils::library_fully_qualified_placeholder(name))
        } else {
            false
        }
    }

    /// Whether the bytecode contains a matching placeholder
    pub fn contains_placeholder(&self, file: impl AsRef<str>, library: impl AsRef<str>) -> bool {
        self.contains_fully_qualified_placeholder(format!("{}:{}", file.as_ref(), library.as_ref()))
    }
}

// Returns a not deployable bytecode by default as empty
impl Default for BytecodeObject {
    fn default() -> Self {
        BytecodeObject::Unlinked("".to_string())
    }
}

impl AsRef<[u8]> for BytecodeObject {
    fn as_ref(&self) -> &[u8] {
        match self {
            BytecodeObject::Bytecode(code) => code.as_ref(),
            BytecodeObject::Unlinked(code) => code.as_bytes(),
        }
    }
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

impl DeployedBytecode {
    /// Returns the underlying `Bytes` if the object is a valid bytecode, and not empty
    pub fn into_bytes(self) -> Option<Bytes> {
        self.bytecode?.object.into_bytes()
    }
}

impl From<Bytecode> for DeployedBytecode {
    fn from(bcode: Bytecode) -> DeployedBytecode {
        DeployedBytecode { bytecode: Some(bcode), immutable_references: Default::default() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompactDeployedBytecode {
    #[serde(flatten)]
    pub bytecode: Option<CompactBytecode>,
    #[serde(
        default,
        rename = "immutableReferences",
        skip_serializing_if = "::std::collections::BTreeMap::is_empty"
    )]
    pub immutable_references: BTreeMap<String, Vec<Offsets>>,
}

impl From<DeployedBytecode> for CompactDeployedBytecode {
    fn from(bcode: DeployedBytecode) -> CompactDeployedBytecode {
        CompactDeployedBytecode {
            bytecode: bcode.bytecode.map(|d_bcode| d_bcode.into()),
            immutable_references: bcode.immutable_references,
        }
    }
}

impl From<CompactDeployedBytecode> for DeployedBytecode {
    fn from(bcode: CompactDeployedBytecode) -> DeployedBytecode {
        DeployedBytecode {
            bytecode: bcode.bytecode.map(|d_bcode| d_bcode.into()),
            immutable_references: bcode.immutable_references,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct GasEstimates {
    pub creation: Creation,
    #[serde(default)]
    pub external: BTreeMap<String, String>,
    #[serde(default)]
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
    #[serde(default, deserialize_with = "serde_helpers::default_for_null")]
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

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_location: Option<SourceLocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary_source_locations: Vec<SecondarySourceLocation>,
    pub r#type: String,
    pub component: String,
    pub severity: Severity,
    #[serde(default, with = "serde_helpers::display_from_str_opt")]
    pub error_code: Option<u64>,
    pub message: String,
    pub formatted_message: Option<String>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = &self.formatted_message {
            match self.severity {
                Severity::Error => msg.as_str().red().fmt(f),
                Severity::Warning | Severity::Info => msg.as_str().yellow().fmt(f),
            }
        } else {
            self.severity.fmt(f)?;
            writeln!(f, ": {}", self.message)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
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

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SourceLocation {
    pub file: String,
    pub start: i32,
    pub end: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SecondarySourceLocation {
    pub file: Option<String>,
    pub start: Option<i32>,
    pub end: Option<i32>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SourceFile {
    pub id: u32,
    #[serde(default)]
    pub ast: serde_json::Value,
}

/// A wrapper type for a list of source files
/// `path -> SourceFile`
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SourceFiles(pub BTreeMap<String, SourceFile>);

impl SourceFiles {
    /// Returns an iterator over the source files' ids and path
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use ethers_solc::artifacts::SourceFiles;
    /// # fn demo(files: SourceFiles) {
    /// let sources: BTreeMap<u32,String> = files.into_ids().collect();
    /// # }
    /// ```
    pub fn into_ids(self) -> impl Iterator<Item = (u32, String)> {
        self.0.into_iter().map(|(k, v)| (v.id, k))
    }

    /// Returns an iterator over the source files' paths and ids
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use ethers_solc::artifacts::SourceFiles;
    /// # fn demo(files: SourceFiles) {
    /// let sources :BTreeMap<String, u32> = files.into_paths().collect();
    /// # }
    /// ```
    pub fn into_paths(self) -> impl Iterator<Item = (String, u32)> {
        self.0.into_iter().map(|(k, v)| (k, v.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn can_link_bytecode() {
        // test cases taken from https://github.com/ethereum/solc-js/blob/master/test/linker.js

        #[derive(Serialize, Deserialize)]
        struct Mockject {
            object: BytecodeObject,
        }
        fn parse_bytecode(bytecode: &str) -> BytecodeObject {
            let object: Mockject =
                serde_json::from_value(serde_json::json!({ "object": bytecode })).unwrap();
            object.object
        }

        let bytecode =  "6060604052341561000f57600080fd5b60f48061001d6000396000f300606060405260043610603e5763ffffffff7c010000000000000000000000000000000000000000000000000000000060003504166326121ff081146043575b600080fd5b3415604d57600080fd5b60536055565b005b73__lib2.sol:L____________________________6326121ff06040518163ffffffff167c010000000000000000000000000000000000000000000000000000000002815260040160006040518083038186803b151560b357600080fd5b6102c65a03f4151560c357600080fd5b5050505600a165627a7a723058207979b30bd4a07c77b02774a511f2a1dd04d7e5d65b5c2735b5fc96ad61d43ae40029";

        let mut object = parse_bytecode(bytecode);
        assert!(object.is_unlinked());
        assert!(object.contains_placeholder("lib2.sol", "L"));
        assert!(object.contains_fully_qualified_placeholder("lib2.sol:L"));
        assert!(object.link("lib2.sol", "L", Address::random()).resolve().is_some());
        assert!(!object.is_unlinked());

        let mut code = Bytecode {
            function_debug_data: Default::default(),
            object: parse_bytecode(bytecode),
            opcodes: None,
            source_map: None,
            generated_sources: vec![],
            link_references: BTreeMap::from([(
                "lib2.sol".to_string(),
                BTreeMap::from([("L".to_string(), vec![])]),
            )]),
        };

        assert!(!code.link("lib2.sol", "Y", Address::random()));
        assert!(code.link("lib2.sol", "L", Address::random()));
        assert!(code.link("lib2.sol", "L", Address::random()));

        let hashed_placeholder = "6060604052341561000f57600080fd5b60f48061001d6000396000f300606060405260043610603e5763ffffffff7c010000000000000000000000000000000000000000000000000000000060003504166326121ff081146043575b600080fd5b3415604d57600080fd5b60536055565b005b73__$cb901161e812ceb78cfe30ca65050c4337$__6326121ff06040518163ffffffff167c010000000000000000000000000000000000000000000000000000000002815260040160006040518083038186803b151560b357600080fd5b6102c65a03f4151560c357600080fd5b5050505600a165627a7a723058207979b30bd4a07c77b02774a511f2a1dd04d7e5d65b5c2735b5fc96ad61d43ae40029";
        let mut object = parse_bytecode(hashed_placeholder);
        assert!(object.is_unlinked());
        assert!(object.contains_placeholder("lib2.sol", "L"));
        assert!(object.contains_fully_qualified_placeholder("lib2.sol:L"));
        assert!(object.link("lib2.sol", "L", Address::default()).resolve().is_some());
        assert!(!object.is_unlinked());
    }

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

    #[test]
    fn test_evm_version_normalization() {
        for (solc_version, evm_version, expected) in &[
            // Ensure 0.4.21 it always returns None
            ("0.4.20", EvmVersion::Homestead, None),
            // Constantinople clipping
            ("0.4.21", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            ("0.4.21", EvmVersion::Constantinople, Some(EvmVersion::Constantinople)),
            ("0.4.21", EvmVersion::London, Some(EvmVersion::Constantinople)),
            // Petersburg
            ("0.5.5", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            ("0.5.5", EvmVersion::Petersburg, Some(EvmVersion::Petersburg)),
            ("0.5.5", EvmVersion::London, Some(EvmVersion::Petersburg)),
            // Istanbul
            ("0.5.14", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            ("0.5.14", EvmVersion::Istanbul, Some(EvmVersion::Istanbul)),
            ("0.5.14", EvmVersion::London, Some(EvmVersion::Istanbul)),
            // Berlin
            ("0.8.5", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            ("0.8.5", EvmVersion::Berlin, Some(EvmVersion::Berlin)),
            ("0.8.5", EvmVersion::London, Some(EvmVersion::Berlin)),
            // London
            ("0.8.7", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            ("0.8.7", EvmVersion::London, Some(EvmVersion::London)),
            ("0.8.7", EvmVersion::London, Some(EvmVersion::London)),
        ] {
            assert_eq!(
                &evm_version.normalize_version(&Version::from_str(solc_version).unwrap()),
                expected
            )
        }
    }
}
