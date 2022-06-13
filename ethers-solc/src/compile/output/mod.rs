//! The output of a compiled project

use crate::{
    artifact_slug,
    artifacts::{
        contract::{CompactContractBytecode, CompactContractRef, Contract},
        Bytecode, DynamicallyLinkable, Error, Linkable, LinkerFn, LinkerOutput,
    },
    sources::{VersionedSourceFile, VersionedSourceFiles},
    Artifact, ArtifactId, ArtifactOutput, Artifacts, CompilerOutput, ConfigurableArtifacts,
};
use contracts::{VersionedContract, VersionedContracts};
use ethers_core::types::Address;
use semver::Version;
use std::{collections::BTreeMap, fmt, path::Path};

pub mod contracts;
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
}

impl<T: ArtifactOutput> Linkable for ProjectCompileOutput<T> {
    fn link(&mut self, file: impl AsRef<str>, library: impl AsRef<str>, address: Address) -> bool {
        let file = file.as_ref();
        let library = library.as_ref();

        // project is linked when all compiled and cached artifacts are linked
        self.compiled_artifacts.link(file, library, address) &&
            self.cached_artifacts.link(file, library, address)
    }

    fn is_unlinked(&self) -> bool {
        // if any of the compiled or cached artifacts are not linked, the project is not linked
        self.compiled_artifacts.is_unlinked() || self.cached_artifacts.is_unlinked()
    }
}
type DependencyTree<T> = BTreeMap<T, Vec<T>>;

fn link_dep<T: std::cmp::Ord + std::clone::Clone + std::fmt::Display>(
    is_linked: &mut BTreeMap<T, bool>,
    order: &mut Vec<T>,
    dependency_tree: &DependencyTree<T>,
    item: T,
    is_lib: bool,
) {
    if is_linked.contains_key(&item) {
        return
    }

    for dep in dependency_tree.get(&item).unwrap() {
        if !is_linked.contains_key(dep) {
            link_dep(is_linked, order, dependency_tree, dep.clone(), true);
        }
    }
    is_linked.insert(item.clone(), true);
    if is_lib {
        order.push(item.clone())
    }
}

fn get_link_order<T: std::cmp::Ord + std::clone::Clone + std::fmt::Display>(
    dependency_tree: &DependencyTree<T>,
) -> Vec<T> {
    let mut order = vec![];
    let mut is_linked = BTreeMap::new();

    for item in dependency_tree.keys() {
        let clone = item.clone();
        link_dep(&mut is_linked, &mut order, dependency_tree, clone, false)
    }

    return order
}

impl<T: ArtifactOutput> DynamicallyLinkable for ProjectCompileOutput<T> {
    fn link_all_dynamic<F>(&mut self, linker_fn: F) -> LinkerOutput
    where
        F: LinkerFn,
    {
        let mut dependency_tree = DependencyTree::new();
        let mut artifacts_map = BTreeMap::new();

        for (artifact_id, artifact) in self.into_artifacts() {
            let deps = artifact
                .get_bytecode()
                .expect("empty bytecode")
                .link_references
                .iter()
                .map(|(fname, link_refs)| {
                    link_refs
                        .iter()
                        .map(|(link_name, _)| {
                            artifact_slug(&Path::new(fname).to_path_buf(), link_name)
                        })
                        .collect::<Vec<String>>()
                })
                .flatten()
                .collect::<Vec<String>>();

            dependency_tree.insert(artifact_id.slug(), deps);
            artifacts_map.insert(artifact_id.slug(), (artifact_id, artifact));
        }

        let link_order = get_link_order(&dependency_tree);

        let mut output = LinkerOutput::new();

        for (artifact_slug, (artifact_id, mut artifact)) in artifacts_map {
            let mut linked_deps = vec![];

            for (idx, lib) in link_order.iter().enumerate() {
                let (lib_artifact_id, lib_artifact) = *artifacts_map.get_mut(lib).unwrap();
                let (lib_artifact_id, lib_artifact) = (&lib_artifact_id, &lib_artifact);

                let addr =
                    linker_fn((&artifact_id, &artifact), (lib_artifact_id, lib_artifact), idx);
                if let Some(addr) = addr {
                    let linked = artifact.link_fully_qualified(lib, addr);

                    let lib_code = *lib_artifact.get_bytecode().unwrap();
                    let lib_code = Into::<Bytecode>::into(lib_code);

                    linked_deps.push((addr, lib_code));

                    if linked {
                        break
                    }
                }
            }

            output.insert(artifact_id, linked_deps);
        }

        return output
    }
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

    pub fn artifacts(self) -> impl Iterator<Item = ()> {
        let Self { cached_artifacts, compiled_artifacts, .. } = self;
        cached_artifacts.artifacts().chain(compiled_artifacts.artifacts())
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
        let mut contracts = BTreeMap::new();
        let versioned_contracts = &self.compiler_output.contracts;
        for (_, name, contract, version) in versioned_contracts.contracts_with_files_and_version() {
            contracts
                .entry(version.to_owned())
                .or_insert(Vec::<(String, Contract)>::new())
                .push((name.to_string(), contract.clone()));
        }
        contracts
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
    ///
    /// let project = Project::builder().build().unwrap();
    /// let contracts: BTreeMap<ArtifactId, CompactContractBytecode> = project.compile().unwrap().into_contract_bytecodes().collect();
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
    /// All source files combined with the solc version used to compile them
    pub sources: VersionedSourceFiles,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_order() {
        let mut dependency_tree = DependencyTree::new();
        dependency_tree.insert("Contract", vec!["Library", "ThirdLibrary"]);
        dependency_tree.insert("Library", vec!["SecondLibrary"]);
        dependency_tree.insert("SecondContract", vec!["Library"]);
        dependency_tree.insert("ThirdContract", vec!["ThirdLibrary", "FourthLibrary"]);
        dependency_tree.insert("FourthLibrary", vec!["Library"]);
        dependency_tree.insert("SecondLibrary", vec![]);
        dependency_tree.insert("ThirdLibrary", vec![]);

        let link_order = get_link_order(&dependency_tree);
        assert_eq!(link_order, vec!["SecondLibrary", "Library", "ThirdLibrary"]);
    }
}
