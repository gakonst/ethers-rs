//! Solc artifact types
use crate::{
    compile::*, error::SolcIoError, remappings::Remapping, utils, ProjectPathsConfig, SolcError,
};
use ethers_core::abi::Abi;
use md5::Digest;
use semver::Version;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::{BTreeMap, HashSet},
    fmt, fs,
    ops::Range,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use tracing::warn;
use yansi::{Color, Paint, Style};

pub mod ast;
pub use ast::*;
pub mod bytecode;
pub mod contract;
pub mod output_selection;
pub mod serde_helpers;
use crate::{
    artifacts::output_selection::{ContractOutputSelection, OutputSelection},
    filter::FilteredSources,
};
pub use bytecode::*;
pub use contract::*;
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
pub(crate) type VersionedSources = BTreeMap<Solc, (Version, Sources)>;

/// A set of different Solc installations with their version and the sources to be compiled
pub(crate) type VersionedFilteredSources = BTreeMap<Solc, (Version, FilteredSources)>;

const SOLIDITY: &str = "Solidity";
const YUL: &str = "Yul";

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

    /// Creates a new [CompilerInput]s with default settings and the given sources
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
                language: SOLIDITY.to_string(),
                sources: solidity_sources,
                settings: Default::default(),
            });
        }
        if !yul_sources.is_empty() {
            res.push(Self {
                language: YUL.to_string(),
                sources: yul_sources,
                settings: Default::default(),
            });
        }
        res
    }

    /// This will remove/adjust values in the `CompilerInput` that are not compatible with this
    /// version
    pub fn sanitize(&mut self, version: &Version) {
        self.settings.sanitize(version)
    }

    /// Consumes the type and returns a [CompilerInput::sanitized] version
    pub fn sanitized(mut self, version: &Version) -> Self {
        self.settings.sanitize(version);
        self
    }

    /// Sets the settings for compilation
    #[must_use]
    pub fn settings(mut self, mut settings: Settings) -> Self {
        if self.is_yul() {
            if !settings.remappings.is_empty() {
                warn!("omitting remappings supplied for the yul sources");
                settings.remappings = vec![];
            }
            if let Some(debug) = settings.debug.as_mut() {
                if debug.revert_strings.is_some() {
                    warn!("omitting revertStrings supplied for the yul sources");
                    debug.revert_strings = None;
                }
            }
        }
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
        if let Some(evm_version) = &mut self.settings.evm_version {
            self.settings.evm_version = evm_version.normalize_version(version);
        }
        self
    }

    #[must_use]
    pub fn with_remappings(mut self, remappings: Vec<Remapping>) -> Self {
        if self.is_yul() {
            warn!("omitting remappings supplied for the yul sources");
        } else {
            self.settings.remappings = remappings;
        }
        self
    }

    /// Sets the path of the source files to `root` adjoined to the existing path
    #[must_use]
    pub fn join_path(mut self, root: impl AsRef<Path>) -> Self {
        let root = root.as_ref();
        self.sources = self.sources.into_iter().map(|(path, s)| (root.join(path), s)).collect();
        self
    }

    /// Removes the `base` path from all source files
    pub fn strip_prefix(mut self, base: impl AsRef<Path>) -> Self {
        let base = base.as_ref();
        self.sources = self
            .sources
            .into_iter()
            .map(|(path, s)| (path.strip_prefix(base).map(Into::into).unwrap_or(path), s))
            .collect();
        self
    }

    /// Similar to `Self::strip_prefix()`. Remove a base path from all
    /// sources _and_ all paths in solc settings such as remappings
    ///
    /// See also `solc --base-path`
    pub fn with_base_path(mut self, base: impl AsRef<Path>) -> Self {
        let base = base.as_ref();
        self.settings = self.settings.with_base_path(base);
        self.strip_prefix(base)
    }

    /// The flag indicating whether the current [CompilerInput] is
    /// constructed for the yul sources
    pub fn is_yul(&self) -> bool {
        self.language == YUL
    }
}

/// A `CompilerInput` representation used for verify
///
/// This type is an alternative `CompilerInput` but uses non-alphabetic ordering of the `sources`
/// and instead emits the (Path -> Source) path in the same order as the pairs in the `sources`
/// `Vec`. This is used over a map, so we can determine the order in which etherscan will display
/// the verified contracts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StandardJsonCompilerInput {
    pub language: String,
    #[serde(with = "serde_helpers::tuple_vec_map")]
    pub sources: Vec<(PathBuf, Source)>,
    pub settings: Settings,
}

// === impl StandardJsonCompilerInput ===

impl StandardJsonCompilerInput {
    pub fn new(sources: Vec<(PathBuf, Source)>, settings: Settings) -> Self {
        Self { language: SOLIDITY.to_string(), sources, settings }
    }

    /// Normalizes the EVM version used in the settings to be up to the latest one
    /// supported by the provided compiler version.
    #[must_use]
    pub fn normalize_evm_version(mut self, version: &Version) -> Self {
        if let Some(evm_version) = &mut self.settings.evm_version {
            self.settings.evm_version = evm_version.normalize_version(version);
        }
        self
    }
}

impl From<StandardJsonCompilerInput> for CompilerInput {
    fn from(input: StandardJsonCompilerInput) -> Self {
        let StandardJsonCompilerInput { language, sources, settings } = input;
        CompilerInput { language, sources: sources.into_iter().collect(), settings }
    }
}

impl From<CompilerInput> for StandardJsonCompilerInput {
    fn from(input: CompilerInput) -> Self {
        let CompilerInput { language, sources, settings } = input;
        StandardJsonCompilerInput { language, sources: sources.into_iter().collect(), settings }
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
    /// Model Checker options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_checker: Option<ModelCheckerSettings>,
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
    /// Change compilation pipeline to go through the Yul intermediate representation. This is
    /// false by default.
    #[serde(rename = "viaIR", default, skip_serializing_if = "Option::is_none")]
    pub via_ir: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug: Option<DebuggingSettings>,
    /// Addresses of the libraries. If not all libraries are given here,
    /// it can result in unlinked objects whose output data is different.
    ///
    /// The top level key is the name of the source file where the library is used.
    /// If remappings are used, this source file should match the global path
    /// after remappings were applied.
    /// If this key is an empty string, that refers to a global level.
    #[serde(default)]
    pub libraries: Libraries,
}

impl Settings {
    /// Creates a new `Settings` instance with the given `output_selection`
    pub fn new(output_selection: impl Into<OutputSelection>) -> Self {
        Self { output_selection: output_selection.into(), ..Default::default() }
    }

    /// Consumes the type and returns a [Settings::sanitize] version
    pub fn sanitized(mut self, version: &Version) -> Self {
        self.sanitize(version);
        self
    }

    /// This will remove/adjust values in the settings that are not compatible with this version.
    pub fn sanitize(&mut self, version: &Version) {
        const V0_6_0: Version = Version::new(0, 6, 0);
        if *version < V0_6_0 {
            if let Some(meta) = &mut self.metadata {
                // introduced in <https://docs.soliditylang.org/en/v0.6.0/using-the-compiler.html#compiler-api>
                // missing in <https://docs.soliditylang.org/en/v0.5.17/using-the-compiler.html#compiler-api>
                meta.bytecode_hash = None;
            }
            // introduced in <https://docs.soliditylang.org/en/v0.6.0/using-the-compiler.html#compiler-api>
            self.debug = None;
        }

        const V0_7_5: Version = Version::new(0, 7, 5);
        if *version < V0_7_5 {
            // introduced in 0.7.5 <https://github.com/ethereum/solidity/releases/tag/v0.7.5>
            self.via_ir = None;
        }

        const V0_8_7: Version = Version::new(0, 8, 7);
        if *version < V0_8_7 {
            // lower the disable version from 0.8.10 to 0.8.7, due to `divModNoSlacks`,
            // `showUnproved` and `solvers` are implemented
            // introduced in <https://github.com/ethereum/solidity/releases/tag/v0.8.7>
            self.model_checker = None;
        }

        const V0_8_10: Version = Version::new(0, 8, 10);
        if *version < V0_8_10 {
            if let Some(debug) = &mut self.debug {
                // introduced in <https://docs.soliditylang.org/en/v0.8.10/using-the-compiler.html#compiler-api>
                // <https://github.com/ethereum/solidity/releases/tag/v0.8.10>
                debug.debug_info.clear();
            }

            if let Some(model_checker) = &mut self.model_checker {
                // introduced in <https://github.com/ethereum/solidity/releases/tag/v0.8.10>
                model_checker.invariants = None;
            }
        }

        const V0_8_18: Version = Version::new(0, 8, 18);
        if *version < V0_8_18 {
            // introduced in 0.8.18 <https://github.com/ethereum/solidity/releases/tag/v0.8.18>
            if let Some(meta) = &mut self.metadata {
                meta.cbor_metadata = None;
            }

            if let Some(model_checker) = &mut self.model_checker {
                if let Some(solvers) = &mut model_checker.solvers {
                    // elf solver introduced in 0.8.18 <https://github.com/ethereum/solidity/releases/tag/v0.8.18>
                    solvers.retain(|solver| *solver != ModelCheckerSolver::Eld);
                }
            }
        }

        if *version < SHANGHAI_SOLC {
            // introduced in 0.8.20 <https://github.com/ethereum/solidity/releases/tag/v0.8.20>
            if let Some(model_checker) = &mut self.model_checker {
                model_checker.show_proved_safe = None;
                model_checker.show_unsupported = None;
            }
        }
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

    /// Sets the ``viaIR` valu
    #[must_use]
    pub fn set_via_ir(mut self, via_ir: bool) -> Self {
        self.via_ir = Some(via_ir);
        self
    }

    /// Enables `viaIR`
    #[must_use]
    pub fn with_via_ir(self) -> Self {
        self.set_via_ir(true)
    }

    /// Enable `viaIR` and use the minimum optimization settings
    ///
    /// This is useful in the following scenarios:
    /// - When compiling for test coverage, this can resolve the "stack too deep" error while still
    ///   giving a relatively accurate source mapping
    /// - When compiling for test, this can reduce the compilation time
    pub fn with_via_ir_minimum_optimization(mut self) -> Self {
        // https://github.com/foundry-rs/foundry/pull/5349
        // https://github.com/ethereum/solidity/issues/12533#issuecomment-1013073350
        self.via_ir = Some(true);
        self.optimizer.details = Some(OptimizerDetails {
            peephole: Some(false),
            inliner: Some(false),
            jumpdest_remover: Some(false),
            order_literals: Some(false),
            deduplicate: Some(false),
            cse: Some(false),
            constant_optimizer: Some(false),
            yul: Some(true), // enable yul optimizer
            yul_details: Some(YulDetails {
                stack_allocation: Some(true),
                // with only unused prunner step
                optimizer_steps: Some("u".to_string()),
            }),
        });
        self
    }

    /// Adds `ast` to output
    #[must_use]
    pub fn with_ast(mut self) -> Self {
        let output =
            self.output_selection.as_mut().entry("*".to_string()).or_insert_with(BTreeMap::default);
        output.insert("".to_string(), vec!["ast".to_string()]);
        self
    }

    /// Strips `base` from all paths
    pub fn with_base_path(mut self, base: impl AsRef<Path>) -> Self {
        let base = base.as_ref();
        self.remappings.iter_mut().for_each(|r| {
            r.strip_prefix(base);
        });

        self.libraries.libs = self
            .libraries
            .libs
            .into_iter()
            .map(|(file, libs)| (file.strip_prefix(base).map(Into::into).unwrap_or(file), libs))
            .collect();

        self.output_selection = OutputSelection(
            self.output_selection
                .0
                .into_iter()
                .map(|(file, selection)| {
                    (
                        Path::new(&file)
                            .strip_prefix(base)
                            .map(|p| format!("{}", p.display()))
                            .unwrap_or(file),
                        selection,
                    )
                })
                .collect(),
        );

        if let Some(mut model_checker) = self.model_checker.take() {
            model_checker.contracts = model_checker
                .contracts
                .into_iter()
                .map(|(path, contracts)| {
                    (
                        Path::new(&path)
                            .strip_prefix(base)
                            .map(|p| format!("{}", p.display()))
                            .unwrap_or(path),
                        contracts,
                    )
                })
                .collect();
            self.model_checker = Some(model_checker);
        }

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
            via_ir: None,
            debug: None,
            libraries: Default::default(),
            remappings: Default::default(),
            model_checker: None,
        }
        .with_ast()
    }
}

/// A wrapper type for all libraries in the form of `<file>:<lib>:<addr>`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Libraries {
    /// All libraries, `(file path -> (Lib name -> Address))
    pub libs: BTreeMap<PathBuf, BTreeMap<String, String>>,
}

// === impl Libraries ===

impl Libraries {
    /// Parses all libraries in the form of
    /// `<file>:<lib>:<addr>`
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::artifacts::Libraries;
    /// let libs = Libraries::parse(&[
    ///     "src/DssSpell.sol:DssExecLib:0xfD88CeE74f7D78697775aBDAE53f9Da1559728E4".to_string(),
    /// ])
    /// .unwrap();
    /// ```
    pub fn parse(libs: &[String]) -> Result<Self, SolcError> {
        let mut libraries = BTreeMap::default();
        for lib in libs {
            let mut items = lib.split(':');
            let file = items.next().ok_or_else(|| {
                SolcError::msg(format!("failed to parse path to library file: {lib}"))
            })?;
            let lib = items
                .next()
                .ok_or_else(|| SolcError::msg(format!("failed to parse library name: {lib}")))?;
            let addr = items
                .next()
                .ok_or_else(|| SolcError::msg(format!("failed to parse library address: {lib}")))?;
            if items.next().is_some() {
                return Err(SolcError::msg(format!(
                    "failed to parse, too many arguments passed: {lib}"
                )))
            }
            libraries
                .entry(file.into())
                .or_insert_with(BTreeMap::default)
                .insert(lib.to_string(), addr.to_string());
        }
        Ok(Self { libs: libraries })
    }

    pub fn is_empty(&self) -> bool {
        self.libs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.libs.len()
    }

    /// Solc expects the lib paths to match the global path after remappings were applied
    ///
    /// See also [ProjectPathsConfig::resolve_import]
    pub fn with_applied_remappings(mut self, config: &ProjectPathsConfig) -> Self {
        self.libs = self
            .libs
            .into_iter()
            .map(|(file, target)| {
                let file = config.resolve_import(&config.root, &file).unwrap_or_else(|err| {
                    warn!(target: "libs", "Failed to resolve library `{}` for linking: {:?}", file.display(), err);
                    file
                });
                (file, target)
            })
            .collect();
        self
    }
}

impl From<BTreeMap<PathBuf, BTreeMap<String, String>>> for Libraries {
    fn from(libs: BTreeMap<PathBuf, BTreeMap<String, String>>) -> Self {
        Self { libs }
    }
}

impl AsRef<BTreeMap<PathBuf, BTreeMap<String, String>>> for Libraries {
    fn as_ref(&self) -> &BTreeMap<PathBuf, BTreeMap<String, String>> {
        &self.libs
    }
}

impl AsMut<BTreeMap<PathBuf, BTreeMap<String, String>>> for Libraries {
    fn as_mut(&mut self) -> &mut BTreeMap<PathBuf, BTreeMap<String, String>> {
        &mut self.libs
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

// === impl OptimizerDetails ===

impl OptimizerDetails {
    /// Returns true if no settings are set.
    pub fn is_empty(&self) -> bool {
        self.peephole.is_none() &&
            self.inliner.is_none() &&
            self.jumpdest_remover.is_none() &&
            self.order_literals.is_none() &&
            self.deduplicate.is_none() &&
            self.cse.is_none() &&
            self.constant_optimizer.is_none() &&
            self.yul.is_none() &&
            self.yul_details.as_ref().map(|yul| yul.is_empty()).unwrap_or(true)
    }
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

// === impl YulDetails ===

impl YulDetails {
    /// Returns true if no settings are set.
    pub fn is_empty(&self) -> bool {
        self.stack_allocation.is_none() && self.optimizer_steps.is_none()
    }
}

/// EVM versions.
///
/// Kept in sync with: <https://github.com/ethereum/solidity/blob/develop/liblangutil/EVMVersion.h>
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    Paris,
    #[default]
    Shanghai,
}

impl EvmVersion {
    /// Normalizes this EVM version by checking against the given Solc [`Version`].
    pub fn normalize_version(self, version: &Version) -> Option<Self> {
        // The EVM version flag was only added in 0.4.21; we work our way backwards
        if *version >= BYZANTIUM_SOLC {
            // If the Solc version is at least at Shanghai, it supports all EVM versions.
            // For all other cases, cap at the at-the-time highest possible fork.
            let normalized = if *version >= SHANGHAI_SOLC {
                self
            } else if self >= Self::Paris && *version >= PARIS_SOLC {
                Self::Paris
            } else if self >= Self::London && *version >= LONDON_SOLC {
                Self::London
            } else if self >= Self::Berlin && *version >= BERLIN_SOLC {
                Self::Berlin
            } else if self >= Self::Istanbul && *version >= ISTANBUL_SOLC {
                Self::Istanbul
            } else if self >= Self::Petersburg && *version >= PETERSBURG_SOLC {
                Self::Petersburg
            } else if self >= Self::Constantinople && *version >= CONSTANTINOPLE_SOLC {
                Self::Constantinople
            } else if self >= Self::Byzantium {
                Self::Byzantium
            } else {
                self
            };
            Some(normalized)
        } else {
            None
        }
    }

    /// Returns the EVM version as a string.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Homestead => "homestead",
            Self::TangerineWhistle => "tangerineWhistle",
            Self::SpuriousDragon => "spuriousDragon",
            Self::Byzantium => "byzantium",
            Self::Constantinople => "constantinople",
            Self::Petersburg => "petersburg",
            Self::Istanbul => "istanbul",
            Self::Berlin => "berlin",
            Self::London => "london",
            Self::Paris => "paris",
            Self::Shanghai => "shanghai",
        }
    }

    /// Has the `RETURNDATACOPY` and `RETURNDATASIZE` opcodes.
    pub fn supports_returndata(&self) -> bool {
        *self >= Self::Byzantium
    }

    pub fn has_static_call(&self) -> bool {
        *self >= Self::Byzantium
    }

    pub fn has_bitwise_shifting(&self) -> bool {
        *self >= Self::Constantinople
    }

    pub fn has_create2(&self) -> bool {
        *self >= Self::Constantinople
    }

    pub fn has_ext_code_hash(&self) -> bool {
        *self >= Self::Constantinople
    }

    pub fn has_chain_id(&self) -> bool {
        *self >= Self::Istanbul
    }

    pub fn has_self_balance(&self) -> bool {
        *self >= Self::Istanbul
    }

    pub fn has_base_fee(&self) -> bool {
        *self >= Self::London
    }

    pub fn has_prevrandao(&self) -> bool {
        *self >= Self::Paris
    }

    pub fn has_push0(&self) -> bool {
        *self >= Self::Shanghai
    }
}

impl fmt::Display for EvmVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EvmVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "homestead" => Ok(Self::Homestead),
            "tangerineWhistle" => Ok(Self::TangerineWhistle),
            "spuriousDragon" => Ok(Self::SpuriousDragon),
            "byzantium" => Ok(Self::Byzantium),
            "constantinople" => Ok(Self::Constantinople),
            "petersburg" => Ok(Self::Petersburg),
            "istanbul" => Ok(Self::Istanbul),
            "berlin" => Ok(Self::Berlin),
            "london" => Ok(Self::London),
            "paris" => Ok(Self::Paris),
            "shanghai" => Ok(Self::Shanghai),
            s => Err(format!("Unknown evm version: {s}")),
        }
    }
}

/// Debugging settings for solc
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebuggingSettings {
    #[serde(
        default,
        with = "serde_helpers::display_from_str_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub revert_strings: Option<RevertStrings>,
    ///How much extra debug information to include in comments in the produced EVM assembly and
    /// Yul code.
    /// Available components are:
    // - `location`: Annotations of the form `@src <index>:<start>:<end>` indicating the location of
    //   the corresponding element in the original Solidity file, where:
    //     - `<index>` is the file index matching the `@use-src` annotation,
    //     - `<start>` is the index of the first byte at that location,
    //     - `<end>` is the index of the first byte after that location.
    // - `snippet`: A single-line code snippet from the location indicated by `@src`. The snippet is
    //   quoted and follows the corresponding `@src` annotation.
    // - `*`: Wildcard value that can be used to request everything.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub debug_info: Vec<String>,
}

/// How to treat revert (and require) reason strings.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RevertStrings {
    /// "default" does not inject compiler-generated revert strings and keeps user-supplied ones.
    #[default]
    Default,
    /// "strip" removes all revert strings (if possible, i.e. if literals are used) keeping
    /// side-effects
    Strip,
    /// "debug" injects strings for compiler-generated internal reverts, implemented for ABI
    /// encoders V1 and V2 for now.
    Debug,
    /// "verboseDebug" even appends further information to user-supplied revert strings (not yet
    /// implemented)
    VerboseDebug,
}

impl fmt::Display for RevertStrings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            RevertStrings::Default => "default",
            RevertStrings::Strip => "strip",
            RevertStrings::Debug => "debug",
            RevertStrings::VerboseDebug => "verboseDebug",
        };
        write!(f, "{string}")
    }
}

impl FromStr for RevertStrings {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(RevertStrings::Default),
            "strip" => Ok(RevertStrings::Strip),
            "debug" => Ok(RevertStrings::Debug),
            "verboseDebug" | "verbosedebug" => Ok(RevertStrings::VerboseDebug),
            s => Err(format!("Unknown evm version: {s}")),
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
    #[serde(
        default,
        rename = "bytecodeHash",
        skip_serializing_if = "Option::is_none",
        with = "serde_helpers::display_from_str_opt"
    )]
    pub bytecode_hash: Option<BytecodeHash>,
    #[serde(default, rename = "appendCBOR", skip_serializing_if = "Option::is_none")]
    pub cbor_metadata: Option<bool>,
}

impl SettingsMetadata {
    pub fn new(hash: BytecodeHash, cbor: bool) -> Self {
        Self { use_literal_content: None, bytecode_hash: Some(hash), cbor_metadata: Some(cbor) }
    }
}

impl From<BytecodeHash> for SettingsMetadata {
    fn from(hash: BytecodeHash) -> Self {
        Self { use_literal_content: None, bytecode_hash: Some(hash), cbor_metadata: None }
    }
}

/// Determines the hash method for the metadata hash that is appended to the bytecode.
///
/// Solc's default is `Ipfs`, see <https://docs.soliditylang.org/en/latest/using-the-compiler.html#compiler-api>.
#[derive(Clone, Debug, Default, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BytecodeHash {
    #[default]
    Ipfs,
    None,
    Bzzr1,
}

impl FromStr for BytecodeHash {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(BytecodeHash::None),
            "ipfs" => Ok(BytecodeHash::Ipfs),
            "bzzr1" => Ok(BytecodeHash::Bzzr1),
            s => Err(format!("Unknown bytecode hash: {s}")),
        }
    }
}

impl fmt::Display for BytecodeHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BytecodeHash::Ipfs => "ipfs",
            BytecodeHash::None => "none",
            BytecodeHash::Bzzr1 => "bzzr1",
        };
        f.write_str(s)
    }
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

/// A helper type that ensures lossless (de)serialisation so we can preserve the exact String
/// metadata value that's being hashed by solc
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LosslessMetadata {
    /// The complete abi as json value
    pub raw_metadata: String,
    /// The deserialised metadata of `raw_metadata`
    pub metadata: Metadata,
}

// === impl LosslessMetadata ===

impl LosslessMetadata {
    /// Returns the whole string raw metadata as `serde_json::Value`
    pub fn raw_json(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::from_str(&self.raw_metadata)
    }
}

impl Serialize for LosslessMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.raw_metadata)
    }
}

impl<'de> Deserialize<'de> for LosslessMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LosslessMetadataVisitor;

        impl<'de> Visitor<'de> for LosslessMetadataVisitor {
            type Value = LosslessMetadata;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "metadata string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let metadata = serde_json::from_str(value).map_err(serde::de::Error::custom)?;
                let raw_metadata = value.to_string();
                Ok(LosslessMetadata { raw_metadata, metadata })
            }
        }
        deserializer.deserialize_str(LosslessMetadataVisitor)
    }
}

/// Compiler settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataSettings {
    #[serde(default)]
    pub remappings: Vec<Remapping>,
    pub optimizer: Optimizer,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SettingsMetadata>,
    /// Required for Solidity: File and name of the contract or library this metadata is created
    /// for.
    #[serde(default, rename = "compilationTarget")]
    pub compilation_target: BTreeMap<String, String>,
    /// Metadata settings
    ///
    /// Note: this differs from `Libraries` and does not require another mapping for file name
    /// since metadata is per file
    #[serde(default)]
    pub libraries: BTreeMap<String, String>,
    /// Change compilation pipeline to go through the Yul intermediate representation. This is
    /// false by default.
    #[serde(rename = "viaIR", default, skip_serializing_if = "Option::is_none")]
    pub via_ir: Option<bool>,
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

/// Model checker settings for solc
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCheckerSettings {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub contracts: BTreeMap<String, Vec<String>>,
    #[serde(
        default,
        with = "serde_helpers::display_from_str_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub engine: Option<ModelCheckerEngine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targets: Option<Vec<ModelCheckerTarget>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invariants: Option<Vec<ModelCheckerInvariant>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_unproved: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub div_mod_with_slacks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solvers: Option<Vec<ModelCheckerSolver>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_unsupported: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_proved_safe: Option<bool>,
}

/// Which model checker engine to run.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelCheckerEngine {
    #[default]
    Default,
    All,
    BMC,
    CHC,
}

impl fmt::Display for ModelCheckerEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            ModelCheckerEngine::Default => "none",
            ModelCheckerEngine::All => "all",
            ModelCheckerEngine::BMC => "bmc",
            ModelCheckerEngine::CHC => "chc",
        };
        write!(f, "{string}")
    }
}

impl FromStr for ModelCheckerEngine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(ModelCheckerEngine::Default),
            "all" => Ok(ModelCheckerEngine::All),
            "bmc" => Ok(ModelCheckerEngine::BMC),
            "chc" => Ok(ModelCheckerEngine::CHC),
            s => Err(format!("Unknown model checker engine: {s}")),
        }
    }
}

/// Which model checker targets to check.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelCheckerTarget {
    Assert,
    Underflow,
    Overflow,
    DivByZero,
    ConstantCondition,
    PopEmptyArray,
    OutOfBounds,
    Balance,
}

impl fmt::Display for ModelCheckerTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            ModelCheckerTarget::Assert => "assert",
            ModelCheckerTarget::Underflow => "underflow",
            ModelCheckerTarget::Overflow => "overflow",
            ModelCheckerTarget::DivByZero => "divByZero",
            ModelCheckerTarget::ConstantCondition => "constantCondition",
            ModelCheckerTarget::PopEmptyArray => "popEmptyArray",
            ModelCheckerTarget::OutOfBounds => "outOfBounds",
            ModelCheckerTarget::Balance => "balance",
        };
        write!(f, "{string}")
    }
}

impl FromStr for ModelCheckerTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "assert" => Ok(ModelCheckerTarget::Assert),
            "underflow" => Ok(ModelCheckerTarget::Underflow),
            "overflow" => Ok(ModelCheckerTarget::Overflow),
            "divByZero" => Ok(ModelCheckerTarget::DivByZero),
            "constantCondition" => Ok(ModelCheckerTarget::ConstantCondition),
            "popEmptyArray" => Ok(ModelCheckerTarget::PopEmptyArray),
            "outOfBounds" => Ok(ModelCheckerTarget::OutOfBounds),
            "balance" => Ok(ModelCheckerTarget::Balance),
            s => Err(format!("Unknown model checker target: {s}")),
        }
    }
}

/// Which model checker invariants to check.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelCheckerInvariant {
    Contract,
    Reentrancy,
}

impl fmt::Display for ModelCheckerInvariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            ModelCheckerInvariant::Contract => "contract",
            ModelCheckerInvariant::Reentrancy => "reentrancy",
        };
        write!(f, "{string}")
    }
}

impl FromStr for ModelCheckerInvariant {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "contract" => Ok(ModelCheckerInvariant::Contract),
            "reentrancy" => Ok(ModelCheckerInvariant::Reentrancy),
            s => Err(format!("Unknown model checker invariant: {s}")),
        }
    }
}

/// Which model checker solvers to check.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelCheckerSolver {
    Cvc4,
    Eld,
    Smtlib2,
    Z3,
}

impl fmt::Display for ModelCheckerSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            ModelCheckerSolver::Cvc4 => "cvc4",
            ModelCheckerSolver::Eld => "eld",
            ModelCheckerSolver::Smtlib2 => "smtlib2",
            ModelCheckerSolver::Z3 => "z3",
        };
        write!(f, "{string}")
    }
}

impl FromStr for ModelCheckerSolver {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cvc4" => Ok(ModelCheckerSolver::Cvc4),
            "eld" => Ok(ModelCheckerSolver::Cvc4),
            "smtlib2" => Ok(ModelCheckerSolver::Smtlib2),
            "z3" => Ok(ModelCheckerSolver::Z3),
            s => Err(format!("Unknown model checker invariant: {s}")),
        }
    }
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
    #[serde(default)]
    pub inputs: Vec<Item>,
    #[serde(rename = "stateMutability", skip_serializing_if = "Option::is_none")]
    pub state_mutability: Option<String>,
    #[serde(rename = "type")]
    pub abi_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<Item>,
    // required to satisfy solidity events
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anonymous: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "internalType")]
    pub internal_type: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub put_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<Item>,
    /// Indexed flag. for solidity events
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexed: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Doc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub methods: Option<DocLibraries>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DocLibraries {
    #[serde(flatten)]
    pub libs: BTreeMap<String, serde_json::Value>,
}

/// Content of a solidity file
///
/// This contains the actual source code of a file
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Source {
    /// Content of the file
    ///
    /// This is an `Arc` because it may be cloned. If the [Graph](crate::resolver::Graph) of the
    /// project contains multiple conflicting versions then the same [Source] may be required by
    /// conflicting versions and needs to be duplicated.
    pub content: Arc<String>,
}

impl Source {
    /// Creates a new instance of [Source] with the given content.
    pub fn new(content: impl Into<String>) -> Self {
        Self { content: Arc::new(content.into()) }
    }

    /// Reads the file's content
    pub fn read(file: impl AsRef<Path>) -> Result<Self, SolcIoError> {
        let file = file.as_ref();
        Ok(Self::new(fs::read_to_string(file).map_err(|err| SolcIoError::new(err, file))?))
    }

    /// Recursively finds all source files under the given dir path and reads them all
    pub fn read_all_from(dir: impl AsRef<Path>) -> Result<Sources, SolcIoError> {
        Self::read_all_files(utils::source_files(dir))
    }

    /// Reads all source files of the given vec
    ///
    /// Depending on the len of the vec it will try to read the files in parallel
    pub fn read_all_files(files: Vec<PathBuf>) -> Result<Sources, SolcIoError> {
        Self::read_all(files)
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
    /// contains at least several paths or the files are rather large.
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
        hasher.update(self);
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
        Ok(Self::new(
            tokio::fs::read_to_string(file).await.map_err(|err| SolcIoError::new(err, file))?,
        ))
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

impl AsRef<[u8]> for Source {
    fn as_ref(&self) -> &[u8] {
        self.content.as_bytes()
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
        // Note: use `to_lowercase` here because solc not necessarily emits the exact file name,
        // e.g. `src/utils/upgradeProxy.sol` is emitted as `src/utils/UpgradeProxy.sol`
        let files: HashSet<_> = files.into_iter().map(|s| s.to_lowercase()).collect();
        self.contracts.retain(|f, _| files.contains(f.to_lowercase().as_str()));
        self.sources.retain(|f, _| files.contains(f.to_lowercase().as_str()));
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

/// A helper type that ensures lossless (de)serialisation unlike [`ethers_core::abi::Abi`] which
/// omits some information of (nested) components in a serde roundtrip. This is a problem for
/// abienconderv2 structs because [`ethers_core::abi::Contract`]'s representation of those are
/// [`ethers_core::abi::Param`] and the `kind` field of type [`ethers_core::abi::ParamType`] does
/// not support deeply nested components as it's the case for structs. This is not easily fixable in
/// ethabi as it would require a redesign of the overall `Param` and `ParamType` types. Instead,
/// this type keeps a copy of the [`serde_json::Value`] when deserialized from the `solc` json
/// compiler output and uses it to serialize the `abi` without loss.
#[derive(Clone, Debug, PartialEq)]
pub struct LosslessAbi {
    /// The complete abi as json value
    pub abi_value: serde_json::Value,
    /// The deserialised version of `abi_value`
    pub abi: Abi,
}

impl Default for LosslessAbi {
    fn default() -> Self {
        LosslessAbi { abi_value: serde_json::json!([]), abi: Default::default() }
    }
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct UserDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub methods: BTreeMap<String, UserDocNotice>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub events: BTreeMap<String, UserDocNotice>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub errors: BTreeMap<String, Vec<UserDocNotice>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notice: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum UserDocNotice {
    // NOTE: this a variant used for constructors on older solc versions
    Constructor(String),
    Notice { notice: String },
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
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub events: BTreeMap<String, EventDoc>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub errors: BTreeMap<String, Vec<ErrorDoc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct MethodDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub returns: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct EventDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct ErrorDoc {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(default, skip_serializing_if = "::std::collections::BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wast: Option<String>,
    pub wasm: String,
}

/// Represents the `storage-layout` section of the `CompilerOutput` if selected.
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub label: String,
    #[serde(rename = "numberOfBytes")]
    pub number_of_bytes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// additional fields
    #[serde(flatten)]
    pub other: BTreeMap<String, serde_json::Value>,
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

/// Tries to mimic Solidity's own error formatting.
///
/// <https://github.com/ethereum/solidity/blob/a297a687261a1c634551b1dac0e36d4573c19afe/liblangutil/SourceReferenceFormatter.cpp#L105>
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !Paint::is_enabled() {
            let msg = self.formatted_message.as_ref().unwrap_or(&self.message);
            self.fmt_severity(f)?;
            f.write_str(": ")?;
            return f.write_str(msg)
        }

        // Error (XXXX): Error Message
        styled(f, self.severity.color().style().bold(), |f| self.fmt_severity(f))?;
        fmt_msg(f, &self.message)?;

        if let Some(msg) = &self.formatted_message {
            let mut lines = msg.lines();

            // skip first line, it should be similar to the error message we wrote above
            lines.next();

            // format the main source location
            fmt_source_location(f, &mut lines)?;

            // format remaining lines as secondary locations
            while let Some(line) = lines.next() {
                f.write_str("\n")?;

                if let Some((note, msg)) = line.split_once(':') {
                    styled(f, Self::secondary_style(), |f| f.write_str(note))?;
                    fmt_msg(f, msg)?;
                } else {
                    f.write_str(line)?;
                }

                fmt_source_location(f, &mut lines)?;
            }
        }

        Ok(())
    }
}

impl Error {
    /// The style of the diagnostic severity.
    pub fn error_style(&self) -> Style {
        self.severity.color().style().bold()
    }

    /// The style of the diagnostic message.
    pub fn message_style() -> Style {
        Color::White.style().bold()
    }

    /// The style of the secondary source location.
    pub fn secondary_style() -> Style {
        Color::Cyan.style().bold()
    }

    /// The style of the source location highlight.
    pub fn highlight_style() -> Style {
        Color::Yellow.style()
    }

    /// The style of the diagnostics.
    pub fn diag_style() -> Style {
        Color::Yellow.style().bold()
    }

    /// The style of the source location frame.
    pub fn frame_style() -> Style {
        Color::Blue.style()
    }

    /// Formats the diagnostic severity:
    ///
    /// ```text
    /// Error (XXXX)
    /// ```
    fn fmt_severity(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.severity.as_str())?;
        if let Some(code) = self.error_code {
            write!(f, " ({code})")?;
        }
        Ok(())
    }
}

/// Calls `fun` in between [`Style::fmt_prefix`] and [`Style::fmt_suffix`].
fn styled<F>(f: &mut fmt::Formatter, style: Style, fun: F) -> fmt::Result
where
    F: FnOnce(&mut fmt::Formatter) -> fmt::Result,
{
    style.fmt_prefix(f)?;
    fun(f)?;
    style.fmt_suffix(f)
}

/// Formats the diagnostic message.
fn fmt_msg(f: &mut fmt::Formatter, msg: &str) -> fmt::Result {
    styled(f, Error::message_style(), |f| {
        f.write_str(": ")?;
        f.write_str(msg.trim_start())
    })
}

/// Colors a Solidity source location:
///
/// ```text
/// --> /home/user/contract.sol:420:69:
///     |
/// 420 |       bad_code()
///     |                ^
/// ```
fn fmt_source_location(f: &mut fmt::Formatter, lines: &mut std::str::Lines) -> fmt::Result {
    // --> source
    if let Some(line) = lines.next() {
        f.write_str("\n")?;

        let arrow = "-->";
        if let Some((left, loc)) = line.split_once(arrow) {
            f.write_str(left)?;
            styled(f, Error::frame_style(), |f| f.write_str(arrow))?;
            f.write_str(loc)?;
        } else {
            f.write_str(line)?;
        }
    }

    // get the next 3 lines
    // FIXME: Somehow do this without allocating
    let next_3 = lines.take(3).collect::<Vec<_>>();
    let [line1, line2, line3] = next_3[..] else {
        for line in next_3 {
            f.write_str("\n")?;
            f.write_str(line)?;
        }
        return Ok(())
    };

    // line 1, just a frame
    fmt_framed_location(f, line1, None)?;

    // line 2, frame and code; highlight the text based on line 3's carets
    let hl_start = line3.find('^');
    let highlight = hl_start.map(|start| {
        let end = if line3.contains("^ (") {
            // highlight the entire line because of "spans across multiple lines" diagnostic
            line2.len()
        } else if let Some(carets) = line3[start..].find(|c: char| c != '^') {
            // highlight the text that the carets point to
            start + carets
        } else {
            // the carets span the entire third line
            line3.len()
        }
        // bound in case carets span longer than the code they point to
        .min(line2.len());
        (start.min(end)..end, Error::highlight_style())
    });
    fmt_framed_location(f, line2, highlight)?;

    // line 3, frame and maybe highlight, this time till the end unconditionally
    let highlight = hl_start.map(|i| (i..line3.len(), Error::diag_style()));
    fmt_framed_location(f, line3, highlight)
}

/// Colors a single Solidity framed source location line. Part of [`fmt_source_location`].
fn fmt_framed_location(
    f: &mut fmt::Formatter,
    line: &str,
    highlight: Option<(Range<usize>, Style)>,
) -> fmt::Result {
    f.write_str("\n")?;

    if let Some((space_or_line_number, rest)) = line.split_once('|') {
        // if the potential frame is not just whitespace or numbers, don't color it
        if !space_or_line_number.chars().all(|c| c.is_whitespace() || c.is_numeric()) {
            return f.write_str(line)
        }

        styled(f, Error::frame_style(), |f| {
            f.write_str(space_or_line_number)?;
            f.write_str("|")
        })?;

        if let Some((range, style)) = highlight {
            let Range { start, end } = range;
            let rest_start = line.len() - rest.len();
            f.write_str(&line[rest_start..start])?;
            styled(f, style, |f| f.write_str(&line[range]))?;
            f.write_str(&line[end..])
        } else {
            f.write_str(rest)
        }
    } else {
        f.write_str(line)
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    #[default]
    Error,
    Warning,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Error" | "error" => Ok(Self::Error),
            "Warning" | "warning" => Ok(Self::Warning),
            "Info" | "info" => Ok(Self::Info),
            s => Err(format!("Invalid severity: {s}")),
        }
    }
}

impl Severity {
    /// Returns `true` if the severity is `Error`.
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }

    /// Returns `true` if the severity is `Warning`.
    pub const fn is_warning(&self) -> bool {
        matches!(self, Self::Warning)
    }

    /// Returns `true` if the severity is `Info`.
    pub const fn is_info(&self) -> bool {
        matches!(self, Self::Info)
    }

    /// Returns the string representation of the severity.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Info => "Info",
        }
    }

    /// Returns the color to format the severity with.
    pub const fn color(&self) -> Color {
        match self {
            Self::Error => Color::Red,
            Self::Warning => Color::Yellow,
            Self::Info => Color::White,
        }
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
    #[serde(default, with = "serde_helpers::empty_json_object_opt")]
    pub ast: Option<Ast>,
}

// === impl SourceFile ===

impl SourceFile {
    /// Returns `true` if the source file contains at least 1 `ContractDefinition` such as
    /// `contract`, `abstract contract`, `interface` or `library`
    pub fn contains_contract_definition(&self) -> bool {
        if let Some(ref ast) = self.ast {
            // contract definitions are only allowed at the source-unit level <https://docs.soliditylang.org/en/latest/grammar.html>
            return ast.nodes.iter().any(|node| node.node_type == NodeType::ContractDefinition)
            // abstract contract, interfaces: ContractDefinition
        }

        false
    }
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
    use crate::AggregatedCompilerOutput;
    use ethers_core::types::Address;
    use std::{fs, path::PathBuf};

    #[test]
    fn can_parse_declaration_error() {
        let s = r#"{
  "errors": [
    {
      "component": "general",
      "errorCode": "7576",
      "formattedMessage": "DeclarationError: Undeclared identifier. Did you mean \"revert\"?\n  --> /Users/src/utils/UpgradeProxy.sol:35:17:\n   |\n35 |                 refert(\"Transparent ERC1967 proxies do not have upgradeable implementations\");\n   |                 ^^^^^^\n\n",
      "message": "Undeclared identifier. Did you mean \"revert\"?",
      "severity": "error",
      "sourceLocation": {
        "end": 1623,
        "file": "/Users/src/utils/UpgradeProxy.sol",
        "start": 1617
      },
      "type": "DeclarationError"
    }
  ],
  "sources": { }
}"#;

        let out: CompilerOutput = serde_json::from_str(s).unwrap();
        assert_eq!(out.errors.len(), 1);

        let mut aggregated = AggregatedCompilerOutput::default();
        aggregated.extend("0.8.12".parse().unwrap(), out);
        assert!(!aggregated.is_unchanged());
    }

    #[test]
    fn can_link_bytecode() {
        // test cases taken from <https://github.com/ethereum/solc-js/blob/master/test/linker.js>

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
            let compiler_input = fs::read_to_string(&path).unwrap();
            serde_json::from_str::<CompilerInput>(&compiler_input).unwrap_or_else(|err| {
                panic!("Failed to read compiler input of {} {}", path.display(), err)
            });
        }
    }

    #[test]
    fn can_parse_standard_json_compiler_input() {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("test-data/in");

        for path in fs::read_dir(dir).unwrap() {
            let path = path.unwrap().path();
            let compiler_input = fs::read_to_string(&path).unwrap();
            let val = serde_json::from_str::<StandardJsonCompilerInput>(&compiler_input)
                .unwrap_or_else(|err| {
                    panic!("Failed to read compiler output of {} {}", path.display(), err)
                });

            let pretty = serde_json::to_string_pretty(&val).unwrap();
            serde_json::from_str::<CompilerInput>(&pretty).unwrap_or_else(|err| {
                panic!("Failed to read converted compiler input of {} {}", path.display(), err)
            });
        }
    }

    #[test]
    fn test_evm_version_normalization() {
        for (solc_version, evm_version, expected) in &[
            // Everything before 0.4.21 should always return None
            ("0.4.20", EvmVersion::Homestead, None),
            // Byzantium clipping
            ("0.4.21", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            ("0.4.21", EvmVersion::Constantinople, Some(EvmVersion::Byzantium)),
            ("0.4.21", EvmVersion::London, Some(EvmVersion::Byzantium)),
            // Constantinople bug fix
            ("0.4.22", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            ("0.4.22", EvmVersion::Constantinople, Some(EvmVersion::Constantinople)),
            ("0.4.22", EvmVersion::London, Some(EvmVersion::Constantinople)),
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
            ("0.8.7", EvmVersion::Paris, Some(EvmVersion::London)),
            // Paris
            ("0.8.18", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            ("0.8.18", EvmVersion::Paris, Some(EvmVersion::Paris)),
            ("0.8.18", EvmVersion::Shanghai, Some(EvmVersion::Paris)),
            // Shanghai
            ("0.8.20", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            ("0.8.20", EvmVersion::Paris, Some(EvmVersion::Paris)),
            ("0.8.20", EvmVersion::Shanghai, Some(EvmVersion::Shanghai)),
        ] {
            let version = Version::from_str(solc_version).unwrap();
            assert_eq!(
                &evm_version.normalize_version(&version),
                expected,
                "({version}, {evm_version:?})"
            )
        }
    }

    #[test]
    fn can_sanitize_byte_code_hash() {
        let version: Version = "0.6.0".parse().unwrap();

        let settings = Settings { metadata: Some(BytecodeHash::Ipfs.into()), ..Default::default() };

        let input = CompilerInput {
            language: "Solidity".to_string(),
            sources: Default::default(),
            settings,
        };

        let i = input.clone().sanitized(&version);
        assert_eq!(i.settings.metadata.unwrap().bytecode_hash, Some(BytecodeHash::Ipfs));

        let version: Version = "0.5.17".parse().unwrap();
        let i = input.sanitized(&version);
        assert!(i.settings.metadata.unwrap().bytecode_hash.is_none());
    }

    #[test]
    fn can_sanitize_cbor_metadata() {
        let version: Version = "0.8.18".parse().unwrap();

        let settings = Settings {
            metadata: Some(SettingsMetadata::new(BytecodeHash::Ipfs, true)),
            ..Default::default()
        };

        let input = CompilerInput {
            language: "Solidity".to_string(),
            sources: Default::default(),
            settings,
        };

        let i = input.clone().sanitized(&version);
        assert_eq!(i.settings.metadata.unwrap().cbor_metadata, Some(true));

        let i = input.sanitized(&Version::new(0, 8, 0));
        assert!(i.settings.metadata.unwrap().cbor_metadata.is_none());
    }

    #[test]
    fn can_parse_libraries() {
        let libraries = ["./src/lib/LibraryContract.sol:Library:0xaddress".to_string()];

        let libs = Libraries::parse(&libraries[..]).unwrap().libs;

        assert_eq!(
            libs,
            BTreeMap::from([(
                PathBuf::from("./src/lib/LibraryContract.sol"),
                BTreeMap::from([("Library".to_string(), "0xaddress".to_string())])
            )])
        );
    }

    #[test]
    fn can_parse_many_libraries() {
        let libraries= [
            "./src/SizeAuctionDiscount.sol:Chainlink:0xffedba5e171c4f15abaaabc86e8bd01f9b54dae5".to_string(),
            "./src/SizeAuction.sol:ChainlinkTWAP:0xffedba5e171c4f15abaaabc86e8bd01f9b54dae5".to_string(),
            "./src/SizeAuction.sol:Math:0x902f6cf364b8d9470d5793a9b2b2e86bddd21e0c".to_string(),
            "./src/test/ChainlinkTWAP.t.sol:ChainlinkTWAP:0xffedba5e171c4f15abaaabc86e8bd01f9b54dae5".to_string(),
            "./src/SizeAuctionDiscount.sol:Math:0x902f6cf364b8d9470d5793a9b2b2e86bddd21e0c".to_string(),
        ];

        let libs = Libraries::parse(&libraries[..]).unwrap().libs;

        pretty_assertions::assert_eq!(
            libs,
            BTreeMap::from([
                (
                    PathBuf::from("./src/SizeAuctionDiscount.sol"),
                    BTreeMap::from([
                        (
                            "Chainlink".to_string(),
                            "0xffedba5e171c4f15abaaabc86e8bd01f9b54dae5".to_string()
                        ),
                        (
                            "Math".to_string(),
                            "0x902f6cf364b8d9470d5793a9b2b2e86bddd21e0c".to_string()
                        )
                    ])
                ),
                (
                    PathBuf::from("./src/SizeAuction.sol"),
                    BTreeMap::from([
                        (
                            "ChainlinkTWAP".to_string(),
                            "0xffedba5e171c4f15abaaabc86e8bd01f9b54dae5".to_string()
                        ),
                        (
                            "Math".to_string(),
                            "0x902f6cf364b8d9470d5793a9b2b2e86bddd21e0c".to_string()
                        )
                    ])
                ),
                (
                    PathBuf::from("./src/test/ChainlinkTWAP.t.sol"),
                    BTreeMap::from([(
                        "ChainlinkTWAP".to_string(),
                        "0xffedba5e171c4f15abaaabc86e8bd01f9b54dae5".to_string()
                    )])
                ),
            ])
        );
    }

    #[test]
    fn test_lossless_metadata() {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Contract {
            #[serde(
                default,
                skip_serializing_if = "Option::is_none",
                with = "serde_helpers::json_string_opt"
            )]
            pub metadata: Option<LosslessMetadata>,
        }

        let s = r#"{"metadata":"{\"compiler\":{\"version\":\"0.4.18+commit.9cf6e910\"},\"language\":\"Solidity\",\"output\":{\"abi\":[{\"constant\":true,\"inputs\":[],\"name\":\"owner\",\"outputs\":[{\"name\":\"\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"view\",\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"newOwner\",\"type\":\"address\"}],\"name\":\"transferOwnership\",\"outputs\":[],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"}],\"devdoc\":{\"methods\":{\"transferOwnership(address)\":{\"details\":\"Allows the current owner to transfer control of the contract to a newOwner.\",\"params\":{\"newOwner\":\"The address to transfer ownership to.\"}}},\"title\":\"Ownable\"},\"userdoc\":{\"methods\":{}}},\"settings\":{\"compilationTarget\":{\"src/Contract.sol\":\"Ownable\"},\"libraries\":{},\"optimizer\":{\"enabled\":true,\"runs\":1000000},\"remappings\":[\":src/=src/\"]},\"sources\":{\"src/Contract.sol\":{\"keccak256\":\"0x3e0d611f53491f313ae035797ed7ecfd1dfd8db8fef8f82737e6f0cd86d71de7\",\"urls\":[\"bzzr://9c33025fa9d1b8389e4c7c9534a1d70fad91c6c2ad70eb5e4b7dc3a701a5f892\"]}},\"version\":1}"}"#;

        let value: serde_json::Value = serde_json::from_str(s).unwrap();
        let c: Contract = serde_json::from_value(value).unwrap();
        assert_eq!(c.metadata.as_ref().unwrap().raw_metadata, "{\"compiler\":{\"version\":\"0.4.18+commit.9cf6e910\"},\"language\":\"Solidity\",\"output\":{\"abi\":[{\"constant\":true,\"inputs\":[],\"name\":\"owner\",\"outputs\":[{\"name\":\"\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"view\",\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"newOwner\",\"type\":\"address\"}],\"name\":\"transferOwnership\",\"outputs\":[],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"}],\"devdoc\":{\"methods\":{\"transferOwnership(address)\":{\"details\":\"Allows the current owner to transfer control of the contract to a newOwner.\",\"params\":{\"newOwner\":\"The address to transfer ownership to.\"}}},\"title\":\"Ownable\"},\"userdoc\":{\"methods\":{}}},\"settings\":{\"compilationTarget\":{\"src/Contract.sol\":\"Ownable\"},\"libraries\":{},\"optimizer\":{\"enabled\":true,\"runs\":1000000},\"remappings\":[\":src/=src/\"]},\"sources\":{\"src/Contract.sol\":{\"keccak256\":\"0x3e0d611f53491f313ae035797ed7ecfd1dfd8db8fef8f82737e6f0cd86d71de7\",\"urls\":[\"bzzr://9c33025fa9d1b8389e4c7c9534a1d70fad91c6c2ad70eb5e4b7dc3a701a5f892\"]}},\"version\":1}");

        let value = serde_json::to_string(&c).unwrap();
        pretty_assertions::assert_eq!(s, value);
    }

    #[test]
    fn test_lossless_storage_layout() {
        let input = include_str!("../../test-data/foundryissue2462.json").trim();
        let layout: StorageLayout = serde_json::from_str(input).unwrap();
        pretty_assertions::assert_eq!(input, &serde_json::to_string(&layout).unwrap());
    }

    // <https://github.com/foundry-rs/foundry/issues/3012>
    #[test]
    fn can_parse_compiler_output_spells_0_6_12() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/0.6.12-with-libs.json");
        let content = fs::read_to_string(path).unwrap();
        let _output: CompilerOutput = serde_json::from_str(&content).unwrap();
    }
}
