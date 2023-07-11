//! The output of a compiled project

use crate::{
    artifacts::{
        contract::{CompactContractBytecode, CompactContractRef, Contract},
        Error, Severity,
    },
    buildinfo::RawBuildInfo,
    info::ContractInfoRef,
    sources::{VersionedSourceFile, VersionedSourceFiles},
    ArtifactId, ArtifactOutput, Artifacts, CompilerOutput, ConfigurableArtifacts, SolcIoError,
};
use contracts::{VersionedContract, VersionedContracts};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, path::Path};
use tracing::trace;
use yansi::Paint;

pub mod contracts;
pub mod info;
pub mod sources;

/// Contains a mixture of already compiled/cached artifacts and the input set of sources that still
/// need to be compiled.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProjectCompileOutput<T: ArtifactOutput = ConfigurableArtifacts> {
    /// contains the aggregated `CompilerOutput`
    pub(crate) compiler_output: AggregatedCompilerOutput,
    /// all artifact files from `output` that were freshly compiled and written
    pub(crate) compiled_artifacts: Artifacts<T::Artifact>,
    /// All artifacts that were read from cache
    pub(crate) cached_artifacts: Artifacts<T::Artifact>,
    /// errors that should be omitted
    pub(crate) ignored_error_codes: Vec<u64>,
    /// set minimum level of severity that is treated as an error
    pub(crate) compiler_severity_filter: Severity,
}

impl<T: ArtifactOutput> ProjectCompileOutput<T> {
    /// Converts all `\\` separators in _all_ paths to `/`
    pub fn slash_paths(&mut self) {
        self.compiler_output.slash_paths();
        self.compiled_artifacts.slash_paths();
        self.cached_artifacts.slash_paths();
    }

    /// Convenience function fo [`Self::slash_paths()`]
    pub fn with_slashed_paths(mut self) -> Self {
        self.slash_paths();
        self
    }

    /// All artifacts together with their contract file name and name `<file name>:<name>`
    ///
    /// This returns a chained iterator of both cached and recompiled contract artifacts
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::btree_map::BTreeMap;
    /// use ethers_solc::ConfigurableContractArtifact;
    /// use ethers_solc::{ArtifactId, Project};
    ///
    /// let project = Project::builder().build().unwrap();
    /// let contracts: BTreeMap<ArtifactId, ConfigurableContractArtifact> = project.compile().unwrap().into_artifacts().collect();
    /// ```
    pub fn into_artifacts(self) -> impl Iterator<Item = (ArtifactId, T::Artifact)> {
        let Self { cached_artifacts, compiled_artifacts, .. } = self;
        cached_artifacts.into_artifacts::<T>().chain(compiled_artifacts.into_artifacts::<T>())
    }

    /// This returns a chained iterator of both cached and recompiled contract artifacts that yields
    /// the contract name and the corresponding artifact
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::btree_map::BTreeMap;
    /// use ethers_solc::ConfigurableContractArtifact;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let artifacts: BTreeMap<String, &ConfigurableContractArtifact> = project.compile().unwrap().artifacts().collect();
    /// ```
    pub fn artifacts(&self) -> impl Iterator<Item = (String, &T::Artifact)> {
        self.versioned_artifacts().map(|(name, (artifact, _))| (name, artifact))
    }

    /// This returns a chained iterator of both cached and recompiled contract artifacts that yields
    /// the contract name and the corresponding artifact
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::btree_map::BTreeMap;
    /// use semver::Version;
    /// use ethers_solc::ConfigurableContractArtifact;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let artifacts: BTreeMap<String, (&ConfigurableContractArtifact, &Version)> = project.compile().unwrap().versioned_artifacts().collect();
    /// ```
    pub fn versioned_artifacts(&self) -> impl Iterator<Item = (String, (&T::Artifact, &Version))> {
        self.cached_artifacts
            .artifact_files()
            .chain(self.compiled_artifacts.artifact_files())
            .filter_map(|artifact| {
                T::contract_name(&artifact.file)
                    .map(|name| (name, (&artifact.artifact, &artifact.version)))
            })
    }

    /// All artifacts together with their contract file and name as tuple `(file, contract
    /// name, artifact)`
    ///
    /// This returns a chained iterator of both cached and recompiled contract artifacts
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::btree_map::BTreeMap;
    /// use ethers_solc::{ConfigurableContractArtifact, Project};
    ///
    /// let project = Project::builder().build().unwrap();
    /// let contracts: Vec<(String, String, ConfigurableContractArtifact)> = project.compile().unwrap().into_artifacts_with_files().collect();
    /// ```
    ///
    /// **NOTE** the `file` will be returned as is, see also [`Self::with_stripped_file_prefixes()`]
    pub fn into_artifacts_with_files(self) -> impl Iterator<Item = (String, String, T::Artifact)> {
        let Self { cached_artifacts, compiled_artifacts, .. } = self;
        cached_artifacts
            .into_artifacts_with_files()
            .chain(compiled_artifacts.into_artifacts_with_files())
    }

    /// All artifacts together with their ID and the sources of the project.
    ///
    /// Note: this only returns the `SourceFiles` for freshly compiled contracts because, if not
    /// included in the `Artifact` itself (see
    /// [`crate::ConfigurableContractArtifact::source_file()`]), is only available via the solc
    /// `CompilerOutput`
    pub fn into_artifacts_with_sources(
        self,
    ) -> (BTreeMap<ArtifactId, T::Artifact>, VersionedSourceFiles) {
        let Self { cached_artifacts, compiled_artifacts, compiler_output, .. } = self;

        (
            cached_artifacts
                .into_artifacts::<T>()
                .chain(compiled_artifacts.into_artifacts::<T>())
                .collect(),
            compiler_output.sources,
        )
    }

    /// Strips the given prefix from all artifact file paths to make them relative to the given
    /// `base` argument
    ///
    /// # Example
    ///
    /// Make all artifact files relative to the project's root directory
    ///
    /// ```no_run
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let output = project.compile().unwrap().with_stripped_file_prefixes(project.root());
    /// ```
    pub fn with_stripped_file_prefixes(mut self, base: impl AsRef<Path>) -> Self {
        let base = base.as_ref();
        self.cached_artifacts = self.cached_artifacts.into_stripped_file_prefixes(base);
        self.compiled_artifacts = self.compiled_artifacts.into_stripped_file_prefixes(base);
        self.compiler_output.strip_prefix_all(base);
        self
    }

    /// Get the (merged) solc compiler output
    /// ```no_run
    /// use std::collections::btree_map::BTreeMap;
    /// use ethers_solc::artifacts::contract::Contract;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let contracts: BTreeMap<String, Contract> =
    ///     project.compile().unwrap().output().contracts_into_iter().collect();
    /// ```
    pub fn output(self) -> AggregatedCompilerOutput {
        self.compiler_output
    }

    /// Returns whether this type has a compiler output.
    pub fn has_compiled_contracts(&self) -> bool {
        self.compiler_output.is_empty()
    }

    /// Returns whether this type does not contain compiled contracts.
    pub fn is_unchanged(&self) -> bool {
        self.compiler_output.is_unchanged()
    }

    /// Returns whether any errors were emitted by the compiler.
    pub fn has_compiler_errors(&self) -> bool {
        self.compiler_output.has_error(&self.ignored_error_codes, &self.compiler_severity_filter)
    }

    /// Returns whether any warnings were emitted by the compiler.
    pub fn has_compiler_warnings(&self) -> bool {
        self.compiler_output.has_warning(&self.ignored_error_codes)
    }

    /// Panics if any errors were emitted by the compiler.
    #[track_caller]
    pub fn succeeded(self) -> Self {
        self.assert_success();
        self
    }

    /// Panics if any errors were emitted by the compiler.
    #[track_caller]
    pub fn assert_success(&self) {
        assert!(!self.has_compiler_errors(), "\n{self}\n");
    }

    /// Returns the set of `Artifacts` that were cached and got reused during
    /// [`crate::Project::compile()`]
    pub fn cached_artifacts(&self) -> &Artifacts<T::Artifact> {
        &self.cached_artifacts
    }

    /// Returns the set of `Artifacts` that were compiled with `solc` in
    /// [`crate::Project::compile()`]
    pub fn compiled_artifacts(&self) -> &Artifacts<T::Artifact> {
        &self.compiled_artifacts
    }

    /// Returns a `BTreeMap` that maps the compiler version used during
    /// [`crate::Project::compile()`] to a Vector of tuples containing the contract name and the
    /// `Contract`
    pub fn compiled_contracts_by_compiler_version(
        &self,
    ) -> BTreeMap<Version, Vec<(String, Contract)>> {
        let mut contracts: BTreeMap<_, Vec<_>> = BTreeMap::new();
        let versioned_contracts = &self.compiler_output.contracts;
        for (_, name, contract, version) in versioned_contracts.contracts_with_files_and_version() {
            contracts
                .entry(version.to_owned())
                .or_default()
                .push((name.to_string(), contract.clone()));
        }
        contracts
    }

    /// Removes the contract with matching path and name using the `<path>:<contractname>` pattern
    /// where `path` is optional.
    ///
    /// If the `path` segment is `None`, then the first matching `Contract` is returned, see
    /// [Self::remove_first]
    ///
    /// # Example
    ///
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// use ethers_solc::info::ContractInfo;
    ///
    /// # fn demo(project: Project) {
    /// let  output = project.compile().unwrap();
    /// let info = ContractInfo::new("src/Greeter.sol:Greeter");
    /// let contract = output.find_contract(&info).unwrap();
    /// # }
    /// ```
    pub fn find_contract<'a>(&self, info: impl Into<ContractInfoRef<'a>>) -> Option<&T::Artifact> {
        let ContractInfoRef { path, name } = info.into();
        if let Some(path) = path {
            self.find(path, name)
        } else {
            self.find_first(name)
        }
    }

    /// Finds the artifact with matching path and name
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let output = project.compile().unwrap();
    /// let contract = output.find("src/Greeter.sol", "Greeter").unwrap();
    /// # }
    /// ```
    pub fn find(&self, path: impl AsRef<str>, contract: impl AsRef<str>) -> Option<&T::Artifact> {
        let contract_path = path.as_ref();
        let contract_name = contract.as_ref();
        if let artifact @ Some(_) = self.compiled_artifacts.find(contract_path, contract_name) {
            return artifact
        }
        self.cached_artifacts.find(contract_path, contract_name)
    }

    /// Finds the first contract with the given name
    pub fn find_first(&self, contract_name: impl AsRef<str>) -> Option<&T::Artifact> {
        let contract_name = contract_name.as_ref();
        if let artifact @ Some(_) = self.compiled_artifacts.find_first(contract_name) {
            return artifact
        }
        self.cached_artifacts.find_first(contract_name)
    }

    /// Finds the artifact with matching path and name
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let output = project.compile().unwrap();
    /// let contract = output.find("src/Greeter.sol", "Greeter").unwrap();
    /// # }
    /// ```
    pub fn remove(
        &mut self,
        path: impl AsRef<str>,
        contract: impl AsRef<str>,
    ) -> Option<T::Artifact> {
        let contract_path = path.as_ref();
        let contract_name = contract.as_ref();
        if let artifact @ Some(_) = self.compiled_artifacts.remove(contract_path, contract_name) {
            return artifact
        }
        self.cached_artifacts.remove(contract_path, contract_name)
    }

    /// Removes the _first_ contract with the given name from the set
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let mut output = project.compile().unwrap();
    /// let contract = output.remove_first("Greeter").unwrap();
    /// # }
    /// ```
    pub fn remove_first(&mut self, contract_name: impl AsRef<str>) -> Option<T::Artifact> {
        let contract_name = contract_name.as_ref();
        if let artifact @ Some(_) = self.compiled_artifacts.remove_first(contract_name) {
            return artifact
        }
        self.cached_artifacts.remove_first(contract_name)
    }

    /// Removes the contract with matching path and name using the `<path>:<contractname>` pattern
    /// where `path` is optional.
    ///
    /// If the `path` segment is `None`, then the first matching `Contract` is returned, see
    /// [Self::remove_first]
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// use ethers_solc::info::ContractInfo;
    /// # fn demo(project: Project) {
    /// let mut output = project.compile().unwrap();
    /// let info = ContractInfo::new("src/Greeter.sol:Greeter");
    /// let contract = output.remove_contract(&info).unwrap();
    /// # }
    /// ```
    pub fn remove_contract<'a>(
        &mut self,
        info: impl Into<ContractInfoRef<'a>>,
    ) -> Option<T::Artifact> {
        let ContractInfoRef { path, name } = info.into();
        if let Some(path) = path {
            self.remove(path, name)
        } else {
            self.remove_first(name)
        }
    }
}

impl ProjectCompileOutput<ConfigurableArtifacts> {
    /// A helper functions that extracts the underlying [`CompactContractBytecode`] from the
    /// [`crate::ConfigurableContractArtifact`]
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::btree_map::BTreeMap;
    /// use ethers_solc::artifacts::contract::CompactContractBytecode;
    /// use ethers_solc::{ArtifactId, Project};
    /// use ethers_solc::contracts::ArtifactContracts;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let contracts: ArtifactContracts = project.compile().unwrap().into_contract_bytecodes().collect();
    /// ```
    pub fn into_contract_bytecodes(
        self,
    ) -> impl Iterator<Item = (ArtifactId, CompactContractBytecode)> {
        self.into_artifacts()
            .map(|(artifact_id, artifact)| (artifact_id, artifact.into_contract_bytecode()))
    }
}

impl<T: ArtifactOutput> fmt::Display for ProjectCompileOutput<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.compiler_output.is_unchanged() {
            f.write_str("Nothing to compile")
        } else {
            self.compiler_output
                .diagnostics(&self.ignored_error_codes, self.compiler_severity_filter)
                .fmt(f)
        }
    }
}

/// The aggregated output of (multiple) compile jobs
///
/// This is effectively a solc version aware `CompilerOutput`
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AggregatedCompilerOutput {
    /// all errors from all `CompilerOutput`
    pub errors: Vec<Error>,
    /// All source files combined with the solc version used to compile them
    pub sources: VersionedSourceFiles,
    /// All compiled contracts combined with the solc version used to compile them
    pub contracts: VersionedContracts,
    // All the `BuildInfo`s of solc invocations.
    pub build_infos: BTreeMap<Version, RawBuildInfo>,
}

impl AggregatedCompilerOutput {
    /// Converts all `\\` separators in _all_ paths to `/`
    pub fn slash_paths(&mut self) {
        self.sources.slash_paths();
        self.contracts.slash_paths();
    }

    /// Whether the output contains a compiler error
    pub fn has_error(
        &self,
        ignored_error_codes: &[u64],
        compiler_severity_filter: &Severity,
    ) -> bool {
        self.errors.iter().any(|err| {
            if compiler_severity_filter.ge(&err.severity) {
                if compiler_severity_filter.is_warning() {
                    return self.has_warning(ignored_error_codes)
                }
                return true
            }
            false
        })
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

    pub fn diagnostics<'a>(
        &'a self,
        ignored_error_codes: &'a [u64],
        compiler_severity_filter: Severity,
    ) -> OutputDiagnostics {
        OutputDiagnostics { compiler_output: self, ignored_error_codes, compiler_severity_filter }
    }

    pub fn is_empty(&self) -> bool {
        self.contracts.is_empty()
    }

    pub fn is_unchanged(&self) -> bool {
        self.contracts.is_empty() && self.errors.is_empty()
    }

    pub fn extend_all<I>(&mut self, out: I)
    where
        I: IntoIterator<Item = (Version, CompilerOutput)>,
    {
        for (v, o) in out {
            self.extend(v, o)
        }
    }

    /// adds a new `CompilerOutput` to the aggregated output
    pub fn extend(&mut self, version: Version, output: CompilerOutput) {
        let CompilerOutput { errors, sources, contracts } = output;
        self.errors.extend(errors);

        for (path, source_file) in sources {
            let sources = self.sources.as_mut().entry(path).or_default();
            sources.push(VersionedSourceFile { source_file, version: version.clone() });
        }

        for (file_name, new_contracts) in contracts {
            let contracts = self.contracts.as_mut().entry(file_name).or_default();
            for (contract_name, contract) in new_contracts {
                let versioned = contracts.entry(contract_name).or_default();
                versioned.push(VersionedContract { contract, version: version.clone() });
            }
        }
    }

    /// Creates all `BuildInfo` files in the given `build_info_dir`
    ///
    /// There can be multiple `BuildInfo`, since we support multiple versions.
    ///
    /// The created files have the md5 hash `{_format,solcVersion,solcLongVersion,input}` as their
    /// file name
    pub fn write_build_infos(&self, build_info_dir: impl AsRef<Path>) -> Result<(), SolcIoError> {
        if self.build_infos.is_empty() {
            return Ok(())
        }
        let build_info_dir = build_info_dir.as_ref();
        std::fs::create_dir_all(build_info_dir)
            .map_err(|err| SolcIoError::new(err, build_info_dir))?;
        for (version, build_info) in &self.build_infos {
            trace!("writing build info file for solc {}", version);
            let file_name = format!("{}.json", build_info.id);
            let file = build_info_dir.join(file_name);
            std::fs::write(&file, &build_info.build_info)
                .map_err(|err| SolcIoError::new(err, file))?;
        }
        Ok(())
    }

    /// Finds the _first_ contract with the given name
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let output = project.compile().unwrap().output();
    /// let contract = output.find_first("Greeter").unwrap();
    /// # }
    /// ```
    pub fn find_first(&self, contract: impl AsRef<str>) -> Option<CompactContractRef> {
        self.contracts.find_first(contract)
    }

    /// Removes the _first_ contract with the given name from the set
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let mut output = project.compile().unwrap().output();
    /// let contract = output.remove_first("Greeter").unwrap();
    /// # }
    /// ```
    pub fn remove_first(&mut self, contract: impl AsRef<str>) -> Option<Contract> {
        self.contracts.remove_first(contract)
    }

    /// Removes the contract with matching path and name
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let mut output = project.compile().unwrap().output();
    /// let contract = output.remove("src/Greeter.sol", "Greeter").unwrap();
    /// # }
    /// ```
    pub fn remove(&mut self, path: impl AsRef<str>, contract: impl AsRef<str>) -> Option<Contract> {
        self.contracts.remove(path, contract)
    }

    /// Removes the contract with matching path and name using the `<path>:<contractname>` pattern
    /// where `path` is optional.
    ///
    /// If the `path` segment is `None`, then the first matching `Contract` is returned, see
    /// [Self::remove_first]
    ///
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// use ethers_solc::info::ContractInfo;
    /// # fn demo(project: Project) {
    /// let mut output = project.compile().unwrap().output();
    /// let info = ContractInfo::new("src/Greeter.sol:Greeter");
    /// let contract = output.remove_contract(&info).unwrap();
    /// # }
    /// ```
    pub fn remove_contract<'a>(
        &mut self,
        info: impl Into<ContractInfoRef<'a>>,
    ) -> Option<Contract> {
        let ContractInfoRef { path, name } = info.into();
        if let Some(path) = path {
            self.remove(path, name)
        } else {
            self.remove_first(name)
        }
    }

    /// Iterate over all contracts and their names
    pub fn contracts_iter(&self) -> impl Iterator<Item = (&String, &Contract)> {
        self.contracts.contracts()
    }

    /// Iterate over all contracts and their names
    pub fn contracts_into_iter(self) -> impl Iterator<Item = (String, Contract)> {
        self.contracts.into_contracts()
    }

    /// Returns an iterator over (`file`, `name`, `Contract`)
    pub fn contracts_with_files_iter(&self) -> impl Iterator<Item = (&String, &String, &Contract)> {
        self.contracts.contracts_with_files()
    }

    /// Returns an iterator over (`file`, `name`, `Contract`)
    pub fn contracts_with_files_into_iter(
        self,
    ) -> impl Iterator<Item = (String, String, Contract)> {
        self.contracts.into_contracts_with_files()
    }

    /// Returns an iterator over (`file`, `name`, `Contract`, `Version`)
    pub fn contracts_with_files_and_version_iter(
        &self,
    ) -> impl Iterator<Item = (&String, &String, &Contract, &Version)> {
        self.contracts.contracts_with_files_and_version()
    }

    /// Returns an iterator over (`file`, `name`, `Contract`, `Version`)
    pub fn contracts_with_files_and_version_into_iter(
        self,
    ) -> impl Iterator<Item = (String, String, Contract, Version)> {
        self.contracts.into_contracts_with_files_and_version()
    }

    /// Given the contract file's path and the contract's name, tries to return the contract's
    /// bytecode, runtime bytecode, and abi
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let output = project.compile().unwrap().output();
    /// let contract = output.get("src/Greeter.sol", "Greeter").unwrap();
    /// # }
    /// ```
    pub fn get(
        &self,
        path: impl AsRef<str>,
        contract: impl AsRef<str>,
    ) -> Option<CompactContractRef> {
        self.contracts.get(path, contract)
    }

    /// Returns the output's source files and contracts separately, wrapped in helper types that
    /// provide several helper methods
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// # fn demo(project: Project) {
    /// let output = project.compile().unwrap().output();
    /// let (sources, contracts) = output.split();
    /// # }
    /// ```
    pub fn split(self) -> (VersionedSourceFiles, VersionedContracts) {
        (self.sources, self.contracts)
    }

    /// Joins all file path with `root`
    pub fn join_all(&mut self, root: impl AsRef<Path>) -> &mut Self {
        let root = root.as_ref();
        self.contracts.join_all(root);
        self.sources.join_all(root);
        self
    }

    /// Strips the given prefix from all file paths to make them relative to the given
    /// `base` argument.
    ///
    /// Convenience method for [Self::strip_prefix_all()] that consumes the type.
    ///
    /// # Example
    ///
    /// Make all sources and contracts relative to the project's root directory
    ///
    /// ```no_run
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let output = project.compile().unwrap().output().with_stripped_file_prefixes(project.root());
    /// ```
    pub fn with_stripped_file_prefixes(mut self, base: impl AsRef<Path>) -> Self {
        let base = base.as_ref();
        self.contracts.strip_prefix_all(base);
        self.sources.strip_prefix_all(base);
        self
    }

    /// Removes `base` from all contract paths
    pub fn strip_prefix_all(&mut self, base: impl AsRef<Path>) -> &mut Self {
        let base = base.as_ref();
        self.contracts.strip_prefix_all(base);
        self.sources.strip_prefix_all(base);
        self
    }
}

/// Helper type to implement display for solc errors
#[derive(Clone, Debug)]
pub struct OutputDiagnostics<'a> {
    /// output of the compiled project
    compiler_output: &'a AggregatedCompilerOutput,
    /// the error codes to ignore
    ignored_error_codes: &'a [u64],
    /// set minimum level of severity that is treated as an error
    compiler_severity_filter: Severity,
}

impl<'a> OutputDiagnostics<'a> {
    /// Returns true if there is at least one error of high severity
    pub fn has_error(&self) -> bool {
        self.compiler_output.has_error(self.ignored_error_codes, &self.compiler_severity_filter)
    }

    /// Returns true if there is at least one warning
    pub fn has_warning(&self) -> bool {
        self.compiler_output.has_warning(self.ignored_error_codes)
    }

    /// Returns true if the contract is a expected to be a test
    fn is_test<T: AsRef<str>>(&self, contract_path: T) -> bool {
        if contract_path.as_ref().ends_with(".t.sol") {
            return true
        }

        self.compiler_output.find_first(&contract_path).map_or(false, |contract| {
            contract.abi.map_or(false, |abi| abi.functions.contains_key("IS_TEST"))
        })
    }
}

impl<'a> fmt::Display for OutputDiagnostics<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Compiler run ")?;
        if self.has_error() {
            Paint::red("failed:")
        } else if self.has_warning() {
            Paint::yellow("successful with warnings:")
        } else {
            Paint::green("successful!")
        }
        .fmt(f)?;

        for err in &self.compiler_output.errors {
            let mut ignored = false;
            if err.severity.is_warning() {
                if let Some(code) = err.error_code {
                    if let Some(source_location) = &err.source_location {
                        // we ignore spdx and contract size warnings in test
                        // files. if we are looking at one of these warnings
                        // from a test file we skip
                        ignored =
                            self.is_test(&source_location.file) && (code == 1878 || code == 5574);
                    }

                    ignored |= self.ignored_error_codes.contains(&code);
                }
            }

            if !ignored {
                f.write_str("\n")?;
                err.fmt(f)?;
            }
        }

        Ok(())
    }
}
