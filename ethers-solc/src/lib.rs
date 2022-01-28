#![doc = include_str ! ("../README.md")]

pub mod artifacts;
pub mod sourcemap;

pub use artifacts::{CompilerInput, CompilerOutput, EvmVersion};
use std::collections::btree_map::BTreeMap;

mod artifact_output;
pub mod cache;
pub mod hh;
pub use artifact_output::*;

mod resolver;
pub use hh::{HardhatArtifact, HardhatArtifacts};
pub use resolver::Graph;

mod compile;
pub use compile::{output::ProjectCompileOutput2, *};

mod config;
pub use config::{AllowedLibPaths, PathStyle, ProjectPathsConfig, SolcConfig};

pub mod remappings;
use crate::{artifacts::Source, cache::SolFilesCache};

pub mod error;
pub mod utils;

use crate::{
    artifacts::Sources,
    cache::SourceUnitNameMap,
    error::{SolcError, SolcIoError},
};
use error::Result;
use std::{
    borrow::Cow,
    convert::TryInto,
    fmt,
    marker::PhantomData,
    path::{Path, PathBuf},
};

/// Utilities for creating, mocking and testing of (temporary) projects
#[cfg(feature = "project-util")]
pub mod project_util;

/// Represents a project workspace and handles `solc` compiling of all contracts in that workspace.
#[derive(Debug)]
pub struct Project<Artifacts: ArtifactOutput = MinimalCombinedArtifacts> {
    /// The layout of the
    pub paths: ProjectPathsConfig,
    /// Where to find solc
    pub solc: Solc,
    /// How solc invocation should be configured.
    pub solc_config: SolcConfig,
    /// Whether caching is enabled
    pub cached: bool,
    /// Whether writing artifacts to disk is enabled
    pub no_artifacts: bool,
    /// Whether writing artifacts to disk is enabled
    pub auto_detect: bool,
    /// How to handle compiler output
    pub artifacts: PhantomData<Artifacts>,
    /// Errors/Warnings which match these error codes are not going to be logged
    pub ignored_error_codes: Vec<u64>,
    /// The paths which will be allowed for library inclusion
    pub allowed_lib_paths: AllowedLibPaths,
    /// Maximum number of `solc` processes to run simultaneously.
    solc_jobs: usize,
}

impl Project {
    /// Convenience function to call `ProjectBuilder::default()`
    ///
    /// # Example
    ///
    /// Configure with `MinimalCombinedArtifacts` artifacts output
    ///
    /// ```rust
    /// use ethers_solc::Project;
    /// let config = Project::builder().build().unwrap();
    /// ```
    ///
    /// To configure any a project with any `ArtifactOutput` use either
    ///
    /// ```rust
    /// use ethers_solc::Project;
    /// let config = Project::builder().build().unwrap();
    /// ```
    ///
    /// or use the builder directly
    ///
    /// ```rust
    /// use ethers_solc::{MinimalCombinedArtifacts, ProjectBuilder};
    /// let config = ProjectBuilder::<MinimalCombinedArtifacts>::default().build().unwrap();
    /// ```
    pub fn builder() -> ProjectBuilder {
        ProjectBuilder::default()
    }
}

impl<Artifacts: ArtifactOutput> Project<Artifacts> {
    /// Returns the path to the artifacts directory
    pub fn artifacts_path(&self) -> &PathBuf {
        &self.paths.artifacts
    }

    /// Returns the path to the sources directory
    pub fn sources_path(&self) -> &PathBuf {
        &self.paths.sources
    }

    /// Returns the path to the cache file
    pub fn cache_path(&self) -> &PathBuf {
        &self.paths.cache
    }

    /// Returns the root directory of the project
    pub fn root(&self) -> &PathBuf {
        &self.paths.root
    }

    /// Sets the maximum number of parallel `solc` processes to run simultaneously.
    ///
    /// # Panics
    ///
    /// if `jobs == 0`
    pub fn set_solc_jobs(&mut self, jobs: usize) {
        assert!(jobs > 0);
        self.solc_jobs = jobs;
    }

    /// Returns all sources found under the project's configured sources path
    #[tracing::instrument(skip_all, fields(name = "sources"))]
    pub fn sources(&self) -> Result<Sources> {
        self.paths.read_sources()
    }

    /// This emits the cargo [`rerun-if-changed`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargorerun-if-changedpath) instruction.
    /// Which tells Cargo to re-run the build script if a file inside the project's sources
    /// directory has changed.
    ///
    /// Use this if you compile a project in a `build.rs` file.
    ///
    /// # Example `build.rs` file
    ///
    ///
    /// ```no_run
    /// use ethers_solc::{Project, ProjectPathsConfig};
    /// // configure the project with all its paths, solc, cache etc. where the root dir is the current rust project.
    /// let project = Project::builder()
    ///     .paths(ProjectPathsConfig::hardhat(env!("CARGO_MANIFEST_DIR")).unwrap())
    ///     .build()
    ///     .unwrap();
    /// let output = project.compile().unwrap();
    /// // Tell Cargo that if a source file changes, to rerun this build script.
    /// project.rerun_if_sources_changed();
    /// ```
    pub fn rerun_if_sources_changed(&self) {
        println!("cargo:rerun-if-changed={}", self.paths.sources.display())
    }

    /// Attempts to compile the contracts found at the configured source location, see
    /// `ProjectPathsConfig::sources`.
    ///
    /// NOTE: this does not check if the contracts were successfully compiled, see
    /// `CompilerOutput::has_error` instead.
    ///
    /// NB: If the `svm` feature is enabled, this function will automatically detect
    /// solc versions across files, see [`Self::svm_compile()`]
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// # fn demo(project: Project) {
    /// let project = Project::builder().build().unwrap();
    /// let output = project.compile().unwrap();
    /// # }
    /// ```
    #[tracing::instrument(skip_all, name = "compile")]
    pub fn compile(&self) -> Result<ProjectCompileOutput2<Artifacts>> {
        let sources = self.paths.read_input_files()?;
        tracing::trace!("found {} sources to compile: {:?}", sources.len(), sources.keys());

        #[cfg(all(feature = "svm", feature = "async"))]
        if self.auto_detect {
            tracing::trace!("using solc auto detection to compile sources");
            return self.svm_compile(sources)
        }

        let mut solc = self.solc.clone();
        if !self.allowed_lib_paths.0.is_empty() {
            solc = solc.arg("--allow-paths").arg(self.allowed_lib_paths.to_string());
        }

        self.compile_with_version(&solc, sources)
    }

    /// Compiles a set of contracts using `svm` managed solc installs
    ///
    /// This will autodetect the appropriate `Solc` version(s) to use when compiling the provided
    /// `Sources`. Solc auto-detection follows semver rules, see also
    /// [`crate::resolver::Graph::get_input_node_versions()`]
    ///
    /// # Errors
    ///
    /// This returns an error if contracts in the `Sources` set are incompatible (violate semver
    /// rules) with their imports, for example source contract `A(=0.8.11)` imports dependency
    /// `C(<0.8.0)`, which are incompatible.
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::{artifacts::Source, Project, utils};
    /// # fn demo(project: Project) {
    /// let project = Project::builder().build().unwrap();
    /// let files = utils::source_files("./src");
    /// let sources = Source::read_all(files).unwrap();
    /// let output = project.svm_compile(sources).unwrap();
    /// # }
    /// ```
    #[cfg(all(feature = "svm", feature = "async"))]
    pub fn svm_compile(&self, sources: Sources) -> Result<ProjectCompileOutput2<Artifacts>> {
        project::ProjectCompiler::with_sources(self, sources)?.compile()
    }

    /// Compiles the given source files with the exact `Solc` executable
    ///
    /// First all libraries for the sources are resolved by scanning all their imports.
    /// If caching is enabled for the `Project`, then all unchanged files are filtered from the
    /// sources and their existing artifacts are read instead. This will also update the cache
    /// file and cleans up entries for files which may have been removed. Unchanged files that
    /// for which an artifact exist, are not compiled again.
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::{Project, Solc};
    /// # fn demo(project: Project) {
    /// let project = Project::builder().build().unwrap();
    /// let sources = project.paths.read_sources().unwrap();
    /// project
    ///     .compile_with_version(
    ///         &Solc::find_svm_installed_version("0.8.11").unwrap().unwrap(),
    ///         sources,
    ///     )
    ///     .unwrap();
    /// # }
    /// ```
    pub fn compile_with_version(
        &self,
        solc: &Solc,
        sources: Sources,
    ) -> Result<ProjectCompileOutput2<Artifacts>> {
        project::ProjectCompiler::with_sources_and_solc(self, sources, solc.clone())?.compile()
    }

    /// Removes the project's artifacts and cache file
    ///
    /// If the cache file was the only file in the folder, this also removes the empty folder.
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// # fn demo(project: Project) {
    /// let project = Project::builder().build().unwrap();
    /// let _ = project.compile().unwrap();
    /// assert!(project.artifacts_path().exists());
    /// assert!(project.cache_path().exists());
    ///
    /// project.cleanup();
    /// assert!(!project.artifacts_path().exists());
    /// assert!(!project.cache_path().exists());
    /// # }
    /// ```
    pub fn cleanup(&self) -> std::result::Result<(), SolcIoError> {
        tracing::trace!("clean up project");
        if self.cache_path().exists() {
            std::fs::remove_file(self.cache_path())
                .map_err(|err| SolcIoError::new(err, self.cache_path()))?;
            if let Some(cache_folder) = self.cache_path().parent() {
                // remove the cache folder if the cache file was the only file
                if cache_folder
                    .read_dir()
                    .map_err(|err| SolcIoError::new(err, cache_folder))?
                    .next()
                    .is_none()
                {
                    std::fs::remove_dir(cache_folder)
                        .map_err(|err| SolcIoError::new(err, cache_folder))?;
                }
            }
            tracing::trace!("removed cache file \"{}\"", self.cache_path().display());
        }
        if self.paths.artifacts.exists() {
            std::fs::remove_dir_all(self.artifacts_path())
                .map_err(|err| SolcIoError::new(err, self.artifacts_path().clone()))?;
            tracing::trace!("removed artifacts dir \"{}\"", self.artifacts_path().display());
        }
        Ok(())
    }

    /// Flattens the target solidity file into a single string suitable for verification.
    ///
    /// This method uses a dependency graph to resolve imported files and substitute
    /// import directives with the contents of target files. It will strip the pragma
    /// version directives and SDPX license identifiers from all imported files.
    ///
    /// NB: the SDPX license identifier will be removed from the imported file
    /// only if it is found at the beginning of the file.
    pub fn flatten(&self, target: &Path) -> Result<String> {
        self.paths.flatten(target)
    }
}

pub struct ProjectBuilder<Artifacts: ArtifactOutput = MinimalCombinedArtifacts> {
    /// The layout of the
    paths: Option<ProjectPathsConfig>,
    /// Where to find solc
    solc: Option<Solc>,
    /// How solc invocation should be configured.
    solc_config: Option<SolcConfig>,
    /// Whether caching is enabled, default is true.
    cached: bool,
    /// Whether writing artifacts to disk is enabled, default is true.
    no_artifacts: bool,
    /// Whether automatic solc version detection is enabled
    auto_detect: bool,
    artifacts: PhantomData<Artifacts>,
    /// Which error codes to ignore
    pub ignored_error_codes: Vec<u64>,
    /// All allowed paths
    pub allowed_paths: Vec<PathBuf>,
    solc_jobs: Option<usize>,
}

impl<Artifacts: ArtifactOutput> ProjectBuilder<Artifacts> {
    #[must_use]
    pub fn paths(mut self, paths: ProjectPathsConfig) -> Self {
        self.paths = Some(paths);
        self
    }

    #[must_use]
    pub fn solc(mut self, solc: impl Into<Solc>) -> Self {
        self.solc = Some(solc.into());
        self
    }

    #[must_use]
    pub fn solc_config(mut self, solc_config: SolcConfig) -> Self {
        self.solc_config = Some(solc_config);
        self
    }

    #[must_use]
    pub fn ignore_error_code(mut self, code: u64) -> Self {
        self.ignored_error_codes.push(code);
        self
    }

    #[must_use]
    pub fn ignore_error_codes(mut self, codes: impl IntoIterator<Item = u64>) -> Self {
        for code in codes {
            self = self.ignore_error_code(code);
        }
        self
    }

    /// Disables cached builds
    #[must_use]
    pub fn ephemeral(self) -> Self {
        self.set_cached(false)
    }

    /// Sets the cache status
    #[must_use]
    pub fn set_cached(mut self, cached: bool) -> Self {
        self.cached = cached;
        self
    }

    /// Disables writing artifacts to disk
    #[must_use]
    pub fn no_artifacts(self) -> Self {
        self.set_no_artifacts(true)
    }

    /// Sets the no artifacts status
    #[must_use]
    pub fn set_no_artifacts(mut self, artifacts: bool) -> Self {
        self.no_artifacts = artifacts;
        self
    }

    /// Sets automatic solc version detection
    #[must_use]
    pub fn set_auto_detect(mut self, auto_detect: bool) -> Self {
        self.auto_detect = auto_detect;
        self
    }

    /// Disables automatic solc version detection
    #[must_use]
    pub fn no_auto_detect(self) -> Self {
        self.set_auto_detect(false)
    }

    /// Sets the maximum number of parallel `solc` processes to run simultaneously.
    ///
    /// # Panics
    ///
    /// `jobs` must be at least 1
    #[must_use]
    pub fn solc_jobs(mut self, jobs: usize) -> Self {
        assert!(jobs > 0);
        self.solc_jobs = Some(jobs);
        self
    }

    /// Sets the number of parallel `solc` processes to `1`, no parallelization
    #[must_use]
    pub fn single_solc_jobs(self) -> Self {
        self.solc_jobs(1)
    }

    /// Set arbitrary `ArtifactOutputHandler`
    pub fn artifacts<A: ArtifactOutput>(self) -> ProjectBuilder<A> {
        let ProjectBuilder {
            paths,
            solc,
            solc_config,
            cached,
            no_artifacts,
            auto_detect,
            ignored_error_codes,
            allowed_paths,
            solc_jobs,
            ..
        } = self;
        ProjectBuilder {
            paths,
            solc,
            solc_config,
            cached,
            no_artifacts,
            auto_detect,
            artifacts: PhantomData::default(),
            ignored_error_codes,
            allowed_paths,
            solc_jobs,
        }
    }

    /// Adds an allowed-path to the solc executable
    #[must_use]
    pub fn allowed_path<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.allowed_paths.push(path.into());
        self
    }

    /// Adds multiple allowed-path to the solc executable
    #[must_use]
    pub fn allowed_paths<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<PathBuf>,
    {
        for arg in args {
            self = self.allowed_path(arg);
        }
        self
    }

    pub fn build(self) -> Result<Project<Artifacts>> {
        let Self {
            paths,
            solc,
            solc_config,
            cached,
            no_artifacts,
            auto_detect,
            artifacts,
            ignored_error_codes,
            mut allowed_paths,
            solc_jobs,
        } = self;

        let solc = solc.unwrap_or_default();
        let solc_config = solc_config.unwrap_or_else(|| SolcConfig::builder().build());

        let paths = paths.map(Ok).unwrap_or_else(ProjectPathsConfig::current_hardhat)?;

        if allowed_paths.is_empty() {
            // allow every contract under root by default
            allowed_paths.push(paths.root.clone())
        }

        Ok(Project {
            paths,
            solc,
            solc_config,
            cached,
            no_artifacts,
            auto_detect,
            artifacts,
            ignored_error_codes,
            allowed_lib_paths: allowed_paths.try_into()?,
            solc_jobs: solc_jobs.unwrap_or_else(::num_cpus::get),
        })
    }
}

impl<Artifacts: ArtifactOutput> Default for ProjectBuilder<Artifacts> {
    fn default() -> Self {
        Self {
            paths: None,
            solc: None,
            solc_config: None,
            cached: true,
            no_artifacts: false,
            auto_detect: true,
            artifacts: PhantomData::default(),
            ignored_error_codes: Vec::new(),
            allowed_paths: vec![],
            solc_jobs: None,
        }
    }
}

/// The outcome of `Project::compile`
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProjectCompileOutput<T: ArtifactOutput> {
    /// If solc was invoked multiple times in `Project::compile` then this contains a merged
    /// version of all `CompilerOutput`s. If solc was called only once then `compiler_output`
    /// holds the `CompilerOutput` of that call.
    compiler_output: Option<CompilerOutput>,
    /// All artifacts that were read from cache
    artifacts: BTreeMap<PathBuf, T::Artifact>,
    ignored_error_codes: Vec<u64>,
}

impl<T: ArtifactOutput> ProjectCompileOutput<T> {
    pub fn with_ignored_errors(ignored_errors: Vec<u64>) -> Self {
        Self {
            compiler_output: None,
            artifacts: Default::default(),
            ignored_error_codes: ignored_errors,
        }
    }

    pub fn from_unchanged(artifacts: BTreeMap<PathBuf, T::Artifact>) -> Self {
        Self { compiler_output: None, artifacts, ignored_error_codes: vec![] }
    }

    pub fn from_compiler_output(
        compiler_output: CompilerOutput,
        ignored_error_codes: Vec<u64>,
    ) -> Self {
        Self {
            compiler_output: Some(compiler_output),
            artifacts: Default::default(),
            ignored_error_codes,
        }
    }

    pub fn from_compiler_output_and_cache(
        compiler_output: CompilerOutput,
        cache: BTreeMap<PathBuf, T::Artifact>,
        ignored_error_codes: Vec<u64>,
    ) -> Self {
        Self { compiler_output: Some(compiler_output), artifacts: cache, ignored_error_codes }
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
    pub fn output(self) -> CompilerOutput {
        self.compiler_output.unwrap_or_default()
    }

    /// Combine two outputs
    pub fn extend(&mut self, compiled: ProjectCompileOutput<T>) {
        let ProjectCompileOutput { compiler_output, artifacts, .. } = compiled;
        self.artifacts.extend(artifacts);
        if let Some(output) = compiler_output {
            self.extend_output(output);
        }
    }

    pub fn extend_output(&mut self, compiled: CompilerOutput) {
        if let Some(output) = self.compiler_output.as_mut() {
            output.errors.extend(compiled.errors);
            output.sources.extend(compiled.sources);
            output.contracts.extend(compiled.contracts);
        } else {
            self.compiler_output = Some(compiled);
        }
    }

    pub fn extend_artifacts(&mut self, artifacts: BTreeMap<PathBuf, T::Artifact>) {
        self.artifacts.extend(artifacts);
    }

    /// Whether this type does not contain compiled contracts
    pub fn is_unchanged(&self) -> bool {
        !self.has_compiled_contracts()
    }

    /// Whether this type has a compiler output
    pub fn has_compiled_contracts(&self) -> bool {
        if let Some(output) = self.compiler_output.as_ref() {
            !output.contracts.is_empty()
        } else {
            false
        }
    }

    /// Whether there were errors
    pub fn has_compiler_errors(&self) -> bool {
        self.compiler_output.as_ref().map(|o| o.has_error()).unwrap_or_default()
    }

    /// Whether there were warnings
    pub fn has_compiler_warnings(&self) -> bool {
        self.compiler_output
            .as_ref()
            .map(|o| o.has_warning(&self.ignored_error_codes))
            .unwrap_or_default()
    }

    /// Finds the first contract with the given name and removes it from the set
    pub fn remove(&mut self, contract_name: impl AsRef<str>) -> Option<T::Artifact> {
        let contract_name = contract_name.as_ref();
        if let Some(output) = self.compiler_output.as_mut() {
            if let contract @ Some(_) = output.contracts.iter_mut().find_map(|(file, c)| {
                c.remove(contract_name).map(|c| T::contract_to_artifact(file, contract_name, c))
            }) {
                return contract
            }
        }
        let key = self
            .artifacts
            .iter()
            .find_map(|(path, _)| {
                T::contract_name(path).filter(|name| name == contract_name).map(|_| path)
            })?
            .clone();
        self.artifacts.remove(&key)
    }
}

impl<T: ArtifactOutput> ProjectCompileOutput<T>
where
    T::Artifact: Clone,
{
    /// Finds the first contract with the given name
    pub fn find(&self, contract_name: impl AsRef<str>) -> Option<Cow<T::Artifact>> {
        let contract_name = contract_name.as_ref();
        if let Some(output) = self.compiler_output.as_ref() {
            if let contract @ Some(_) = output.contracts.iter().find_map(|(file, contracts)| {
                contracts
                    .get(contract_name)
                    .map(|c| T::contract_to_artifact(file, contract_name, c.clone()))
                    .map(Cow::Owned)
            }) {
                return contract
            }
        }
        self.artifacts.iter().find_map(|(path, art)| {
            T::contract_name(path).filter(|name| name == contract_name).map(|_| Cow::Borrowed(art))
        })
    }
}

impl<T: ArtifactOutput + 'static> ProjectCompileOutput<T> {
    /// All artifacts together with their contract file name and name `<file name>:<name>`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::btree_map::BTreeMap;
    /// use ethers_solc::artifacts::CompactContract;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let contracts: BTreeMap<String, CompactContract> = project.compile().unwrap().into_artifacts().collect();
    /// ```
    pub fn into_artifacts(self) -> Box<dyn Iterator<Item = (String, T::Artifact)>> {
        // let artifacts = self.artifacts.into_iter().filter_map(|(path, art)| {
        //     T::contract_name(&path).map(|name| {
        //         (format!("{}:{}", path.file_name().unwrap().to_string_lossy(), name), art)
        //     })
        // });
        //
        // let artifacts: Box<dyn Iterator<Item = (String, T::Artifact)>> = if let Some(output) =
        //     self.compiler_output.take()
        // {
        //     Box::new(artifacts.chain(T::output_to_artifacts(output).into_values().flatten().map(
        //         |(name, artifact)| {
        //             (format!("{}:{}", T::output_file_name(&name).display(), name), artifact)
        //         },
        //     )))
        // } else {
        //     Box::new(artifacts)
        // };
        // artifacts
        todo!()
    }
}

impl<T: ArtifactOutput> fmt::Display for ProjectCompileOutput<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(output) = self.compiler_output.as_ref() {
            output.diagnostics(&self.ignored_error_codes).fmt(f)
        } else {
            f.write_str("Nothing to compile")
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(all(feature = "svm", feature = "async"))]
    fn test_build_all_versions() {
        use super::*;

        let paths = ProjectPathsConfig::builder()
            .root("./test-data/test-contract-versions")
            .sources("./test-data/test-contract-versions")
            .build()
            .unwrap();
        let project = Project::builder().paths(paths).no_artifacts().ephemeral().build().unwrap();
        let compiled = project.compile().unwrap();
        assert!(!compiled.has_compiler_errors());
        let contracts = compiled.output().contracts;
        // Contracts A to F
        assert_eq!(contracts.keys().count(), 5);
    }

    #[test]
    #[cfg(all(feature = "svm", feature = "async"))]
    fn test_build_many_libs() {
        use super::*;

        let root = utils::canonicalize("./test-data/test-contract-libs").unwrap();

        let paths = ProjectPathsConfig::builder()
            .root(&root)
            .sources(root.join("src"))
            .lib(root.join("lib1"))
            .lib(root.join("lib2"))
            .build()
            .unwrap();
        let project = Project::builder()
            .paths(paths)
            .no_artifacts()
            .ephemeral()
            .no_artifacts()
            .build()
            .unwrap();
        let compiled = project.compile().unwrap();
        assert!(!compiled.has_compiler_errors());
        let contracts = compiled.output().contracts;
        assert_eq!(contracts.keys().count(), 3);
    }

    #[test]
    #[cfg(all(feature = "svm", feature = "async"))]
    fn test_build_remappings() {
        use super::*;

        let root = utils::canonicalize("./test-data/test-contract-remappings").unwrap();
        let paths = ProjectPathsConfig::builder()
            .root(&root)
            .sources(root.join("src"))
            .lib(root.join("lib"))
            .build()
            .unwrap();
        let project = Project::builder().no_artifacts().paths(paths).ephemeral().build().unwrap();
        let compiled = project.compile().unwrap();
        assert!(!compiled.has_compiler_errors());
        let contracts = compiled.output().contracts;
        assert_eq!(contracts.keys().count(), 2);
    }
}
