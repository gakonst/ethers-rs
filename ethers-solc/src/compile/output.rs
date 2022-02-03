//! The output of a compiled project

use crate::{
    artifacts::{CompactContractRef, Contract, Error, SourceFile, SourceFiles},
    contracts::{VersionedContract, VersionedContracts},
    ArtifactOutput, Artifacts, CompilerOutput,
};
use semver::Version;
use std::{collections::BTreeMap, fmt, path::Path};

/// Contains a mixture of already compiled/cached artifacts and the input set of sources that still
/// need to be compiled.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProjectCompileOutput<T: ArtifactOutput> {
    /// contains the aggregated `CompilerOutput`
    ///
    /// See [`CompilerSources::compile`]
    pub(crate) compiler_output: AggregatedCompilerOutput,
    /// all artifact files from `output` that were freshly compiled and written
    pub(crate) compiled_artifacts: Artifacts<T::Artifact>,
    /// All artifacts that were read from cache
    pub(crate) cached_artifacts: Artifacts<T::Artifact>,
    /// errors that should be omitted
    pub(crate) ignored_error_codes: Vec<u64>,
}

impl<T: ArtifactOutput> ProjectCompileOutput<T> {
    /// All artifacts together with their contract file name and name `<file name>:<name>`
    ///
    /// This returns a chained iterator of both cached and recompiled contract artifacts
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::btree_map::BTreeMap;
    /// use ethers_solc::artifacts::CompactContractBytecode;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let contracts: BTreeMap<String, CompactContractBytecode> = project.compile().unwrap().into_artifacts().collect();
    /// ```
    pub fn into_artifacts(self) -> impl Iterator<Item = (String, T::Artifact)> {
        let Self { cached_artifacts, compiled_artifacts, .. } = self;
        cached_artifacts.into_artifacts::<T>().chain(compiled_artifacts.into_artifacts::<T>())
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
    /// use ethers_solc::artifacts::CompactContractBytecode;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let contracts: Vec<(String, String, CompactContractBytecode)> = project.compile().unwrap().into_artifacts_with_files().collect();
    /// ```
    ///
    /// **NOTE** the `file` will be returned as is, see also [`Self::with_stripped_file_prefixes()`]
    pub fn into_artifacts_with_files(self) -> impl Iterator<Item = (String, String, T::Artifact)> {
        let Self { cached_artifacts, compiled_artifacts, .. } = self;
        cached_artifacts
            .into_artifacts_with_files()
            .chain(compiled_artifacts.into_artifacts_with_files())
    }

    /// Strips the given prefix from all artifact file paths to make them relative to the given
    /// `base` argument
    ///
    /// # Example
    ///
    /// Make all artifact files relative tot the project's root directory
    ///
    /// ```no_run
    /// use ethers_solc::artifacts::CompactContractBytecode;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let output = project.compile().unwrap().with_stripped_file_prefixes(project.root());
    /// ```
    pub fn with_stripped_file_prefixes(mut self, base: impl AsRef<Path>) -> Self {
        let base = base.as_ref();
        self.cached_artifacts = self.cached_artifacts.into_stripped_file_prefixes(base);
        self.compiled_artifacts = self.compiled_artifacts.into_stripped_file_prefixes(base);
        self
    }

    /// Get the (merged) solc compiler output
    /// ```no_run
    /// use std::collections::btree_map::BTreeMap;
    /// use ethers_solc::artifacts::Contract;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let contracts: BTreeMap<String, Contract> =
    ///     project.compile().unwrap().output().contracts_into_iter().collect();
    /// ```
    pub fn output(self) -> AggregatedCompilerOutput {
        self.compiler_output
    }

    /// Whether this type has a compiler output
    pub fn has_compiled_contracts(&self) -> bool {
        self.compiler_output.is_empty()
    }

    /// Whether this type does not contain compiled contracts
    pub fn is_unchanged(&self) -> bool {
        self.compiler_output.is_unchanged()
    }

    /// Whether there were errors
    pub fn has_compiler_errors(&self) -> bool {
        self.compiler_output.has_error()
    }

    /// Whether there were warnings
    pub fn has_compiler_warnings(&self) -> bool {
        self.compiler_output.has_warning(&self.ignored_error_codes)
    }

    /// Finds the first contract with the given name and removes it from the set
    pub fn remove(&mut self, contract_name: impl AsRef<str>) -> Option<T::Artifact> {
        let contract_name = contract_name.as_ref();
        if let artifact @ Some(_) = self.compiled_artifacts.remove(contract_name) {
            return artifact
        }
        self.cached_artifacts.remove(contract_name)
    }

    /// Returns the set of `Artifacts` that were cached and got reused during [`Project::compile()`]
    pub fn cached_artifacts(&self) -> &Artifacts<T::Artifact> {
        &self.cached_artifacts
    }

    /// Returns the set of `Artifacts` that were compiled with `solc` in [`Project::compile()`]
    pub fn compiled_artifacts(&self) -> &Artifacts<T::Artifact> {
        &self.compiled_artifacts
    }
}

impl<T: ArtifactOutput> ProjectCompileOutput<T>
where
    T::Artifact: Clone,
{
    /// Finds the first contract with the given name
    pub fn find(&self, contract_name: impl AsRef<str>) -> Option<&T::Artifact> {
        let contract_name = contract_name.as_ref();
        if let artifact @ Some(_) = self.compiled_artifacts.find(contract_name) {
            return artifact
        }
        self.cached_artifacts.find(contract_name)
    }
}

impl<T: ArtifactOutput> fmt::Display for ProjectCompileOutput<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.compiler_output.is_unchanged() {
            f.write_str("Nothing to compile")
        } else {
            self.compiler_output.diagnostics(&self.ignored_error_codes).fmt(f)
        }
    }
}

/// The aggregated output of (multiple) compile jobs
///
/// This is effectively a solc version aware `CompilerOutput`
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AggregatedCompilerOutput {
    /// all errors from all `CompilerOutput`
    pub errors: Vec<Error>,
    /// All source files
    pub sources: BTreeMap<String, SourceFile>,
    /// All compiled contracts combined with the solc version used to compile them
    pub contracts: VersionedContracts,
}

impl AggregatedCompilerOutput {
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

    pub fn diagnostics<'a>(&'a self, ignored_error_codes: &'a [u64]) -> OutputDiagnostics {
        OutputDiagnostics { compiler_output: self, ignored_error_codes }
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
        self.errors.extend(output.errors);
        self.sources.extend(output.sources);

        for (file_name, new_contracts) in output.contracts {
            let contracts = self.contracts.as_mut().entry(file_name).or_default();
            for (contract_name, contract) in new_contracts {
                let versioned = contracts.entry(contract_name).or_default();
                versioned.push(VersionedContract { contract, version: version.clone() });
            }
        }
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
    /// let contract = output.find("Greeter").unwrap();
    /// # }
    /// ```
    pub fn find(&self, contract: impl AsRef<str>) -> Option<CompactContractRef> {
        self.contracts.find(contract)
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
    /// let contract = output.remove("Greeter").unwrap();
    /// # }
    /// ```
    pub fn remove(&mut self, contract: impl AsRef<str>) -> Option<Contract> {
        self.contracts.remove(contract)
    }

    /// Iterate over all contracts and their names
    pub fn contracts_iter(&self) -> impl Iterator<Item = (&String, &Contract)> {
        self.contracts.contracts()
    }

    /// Iterate over all contracts and their names
    pub fn contracts_into_iter(self) -> impl Iterator<Item = (String, Contract)> {
        self.contracts.into_contracts()
    }

    /// Given the contract file's path and the contract's name, tries to return the contract's
    /// bytecode, runtime bytecode, and abi
    pub fn get(&self, path: &str, contract: &str) -> Option<CompactContractRef> {
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
    pub fn split(self) -> (SourceFiles, VersionedContracts) {
        (SourceFiles(self.sources), self.contracts)
    }
}

/// Helper type to implement display for solc errors
#[derive(Clone, Debug)]
pub struct OutputDiagnostics<'a> {
    compiler_output: &'a AggregatedCompilerOutput,
    ignored_error_codes: &'a [u64],
}

impl<'a> OutputDiagnostics<'a> {
    /// Returns true if there is at least one error of high severity
    pub fn has_error(&self) -> bool {
        self.compiler_output.has_error()
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

        self.compiler_output.find(&contract_path).map_or(false, |contract| {
            contract.abi.map_or(false, |abi| abi.functions.contains_key("IS_TEST"))
        })
    }
}

impl<'a> fmt::Display for OutputDiagnostics<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has_error() {
            f.write_str("Compiler run failed")?;
        } else if self.has_warning() {
            f.write_str("Compiler run successful (with warnings)")?;
        } else {
            f.write_str("Compiler run successful")?;
        }
        for err in &self.compiler_output.errors {
            if err.severity.is_warning() {
                let is_ignored = err.error_code.as_ref().map_or(false, |code| {
                    if let Some(source_location) = &err.source_location {
                        // we ignore spdx and contract size warnings in test
                        // files. if we are looking at one of these warnings
                        // from a test file we skip
                        if self.is_test(&source_location.file) && (*code == 1878 || *code == 5574) {
                            return true
                        }
                    }

                    self.ignored_error_codes.contains(code)
                });

                if !is_ignored {
                    writeln!(f, "\n{}", err)?;
                }
            } else {
                writeln!(f, "\n{}", err)?;
            }
        }
        Ok(())
    }
}
