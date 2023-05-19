#![doc = include_str!("../README.md")]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod artifacts;
pub mod sourcemap;

pub use artifacts::{CompilerInput, CompilerOutput, EvmVersion};
use std::collections::{BTreeMap, HashSet};

mod artifact_output;
pub mod buildinfo;
pub mod cache;
pub mod hh;
pub use artifact_output::*;

pub mod resolver;
pub use hh::{HardhatArtifact, HardhatArtifacts};
pub use resolver::Graph;

mod compile;
pub use compile::{
    output::{AggregatedCompilerOutput, ProjectCompileOutput},
    *,
};

mod config;
pub use config::{AllowedLibPaths, PathStyle, ProjectPaths, ProjectPathsConfig, SolcConfig};

pub mod remappings;
use crate::artifacts::{Source, SourceFile, StandardJsonCompilerInput};

pub mod error;
mod filter;
pub mod report;
pub mod utils;
pub use filter::{FileFilter, TestFileFilter};

use crate::{
    artifacts::Sources,
    cache::SolFilesCache,
    config::IncludePaths,
    error::{SolcError, SolcIoError},
    sources::{VersionedSourceFile, VersionedSourceFiles},
};
use artifacts::{contract::Contract, Severity};
use compile::output::contracts::VersionedContracts;
use error::Result;
use semver::Version;
use std::path::{Path, PathBuf};

/// Utilities for creating, mocking and testing of (temporary) projects
#[cfg(feature = "project-util")]
pub mod project_util;

/// Represents a project workspace and handles `solc` compiling of all contracts in that workspace.
#[derive(Debug)]
pub struct Project<T: ArtifactOutput = ConfigurableArtifacts> {
    /// The layout of the project
    pub paths: ProjectPathsConfig,
    /// Where to find solc
    pub solc: Solc,
    /// How solc invocation should be configured.
    pub solc_config: SolcConfig,
    /// Whether caching is enabled
    pub cached: bool,
    /// Whether to output build information with each solc call.
    pub build_info: bool,
    /// Whether writing artifacts to disk is enabled
    pub no_artifacts: bool,
    /// Whether writing artifacts to disk is enabled
    pub auto_detect: bool,
    /// Handles all artifacts related tasks, reading and writing from the artifact dir.
    pub artifacts: T,
    /// Errors/Warnings which match these error codes are not going to be logged
    pub ignored_error_codes: Vec<u64>,
    /// The minimum severity level that is treated as a compiler error
    pub compiler_severity_filter: Severity,
    /// The paths which will be allowed for library inclusion
    pub allowed_paths: AllowedLibPaths,
    /// The paths which will be used with solc's `--include-path` attribute
    pub include_paths: IncludePaths,
    /// Maximum number of `solc` processes to run simultaneously.
    solc_jobs: usize,
    /// Offline mode, if set, network access (download solc) is disallowed
    pub offline: bool,
    /// Windows only config value to ensure the all paths use `/` instead of `\\`, same as `solc`
    ///
    /// This is a noop on other platforms
    pub slash_paths: bool,
}

impl Project {
    /// Convenience function to call `ProjectBuilder::default()`
    ///
    /// # Example
    ///
    /// Configure with `ConfigurableArtifacts` artifacts output
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
    /// use ethers_solc::{ConfigurableArtifacts, ProjectBuilder};
    /// let config = ProjectBuilder::<ConfigurableArtifacts>::default().build().unwrap();
    /// ```
    pub fn builder() -> ProjectBuilder {
        ProjectBuilder::default()
    }
}

impl<T: ArtifactOutput> Project<T> {
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

    /// Returns the path to the `build-info` directory nested in the artifacts dir
    pub fn build_info_path(&self) -> &PathBuf {
        &self.paths.build_infos
    }

    /// Returns the root directory of the project
    pub fn root(&self) -> &PathBuf {
        &self.paths.root
    }

    /// Returns the handler that takes care of processing all artifacts
    pub fn artifacts_handler(&self) -> &T {
        &self.artifacts
    }

    /// Convenience function to read the cache file.
    /// See also [SolFilesCache::read_joined()]
    pub fn read_cache_file(&self) -> Result<SolFilesCache> {
        SolFilesCache::read_joined(&self.paths)
    }

    /// Applies the configured arguments to the given `Solc`
    ///
    /// See [Self::configure_solc_with_version()]
    pub(crate) fn configure_solc(&self, solc: Solc) -> Solc {
        let version = solc.version().ok();
        self.configure_solc_with_version(solc, version, Default::default())
    }

    /// Applies the configured arguments to the given `Solc`
    ///
    /// This will set the `--allow-paths` to the paths configured for the `Project`, if any.
    ///
    /// If a version is provided and it is applicable it will also set `--base-path` and
    /// `--include-path` This will set the `--allow-paths` to the paths configured for the
    /// `Project`, if any.
    /// This also accepts additional `include_paths`
    pub(crate) fn configure_solc_with_version(
        &self,
        mut solc: Solc,
        version: Option<Version>,
        mut include_paths: IncludePaths,
    ) -> Solc {
        if !solc.args.iter().any(|arg| arg == "--allow-paths") {
            if let Some([allow, libs]) = self.allowed_paths.args() {
                solc = solc.arg(allow).arg(libs);
            }
        }
        if let Some(version) = version {
            if SUPPORTS_BASE_PATH.matches(&version) {
                let base_path = format!("{}", self.root().display());
                if !base_path.is_empty() {
                    solc = solc.with_base_path(self.root());
                    if SUPPORTS_INCLUDE_PATH.matches(&version) {
                        include_paths.extend(self.include_paths.paths().cloned());
                        // `--base-path` and `--include-path` conflict if set to the same path, so
                        // as a precaution, we ensure here that the `--base-path` is not also used
                        // for `--include-path`
                        include_paths.remove(self.root());
                        solc = solc.args(include_paths.args());
                    }
                }
            } else {
                solc.base_path.take();
            }
        }
        solc
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
    /// solc versions across files.
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
    pub fn compile(&self) -> Result<ProjectCompileOutput<T>> {
        let sources = self.paths.read_input_files()?;
        tracing::trace!("found {} sources to compile: {:?}", sources.len(), sources.keys());

        #[cfg(all(feature = "svm-solc", not(target_arch = "wasm32")))]
        if self.auto_detect {
            tracing::trace!("using solc auto detection to compile sources");
            return self.svm_compile(sources)
        }

        self.compile_with_version(&self.solc, sources)
    }

    /// Compiles a set of contracts using `svm` managed solc installs
    ///
    /// This will autodetect the appropriate `Solc` version(s) to use when compiling the provided
    /// `Sources`. Solc auto-detection follows semver rules, see also
    /// `Graph::get_input_node_versions`
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
    #[cfg(all(feature = "svm-solc", not(target_arch = "wasm32")))]
    pub fn svm_compile(&self, sources: Sources) -> Result<ProjectCompileOutput<T>> {
        project::ProjectCompiler::with_sources(self, sources)?.compile()
    }

    /// Convenience function to compile a single solidity file with the project's settings.
    /// Same as [`Self::svm_compile()`] but with the given `file` as input.
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// # fn demo(project: Project) {
    /// let project = Project::builder().build().unwrap();
    /// let output = project.compile_file("example/Greeter.sol").unwrap();
    /// # }
    /// ```
    #[cfg(all(feature = "svm-solc", not(target_arch = "wasm32")))]
    pub fn compile_file(&self, file: impl Into<PathBuf>) -> Result<ProjectCompileOutput<T>> {
        let file = file.into();
        let source = Source::read(&file)?;
        project::ProjectCompiler::with_sources(self, Sources::from([(file, source)]))?.compile()
    }

    /// Convenience function to compile a series of solidity files with the project's settings.
    /// Same as [`Self::compile()`] but with the given `files` as input.
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// # fn demo(project: Project) {
    /// let project = Project::builder().build().unwrap();
    /// let output = project
    ///     .compile_files(
    ///         vec!["examples/Foo.sol", "examples/Bar.sol"]
    ///     ).unwrap();
    /// # }
    /// ```
    pub fn compile_files<P, I>(&self, files: I) -> Result<ProjectCompileOutput<T>>
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        let sources = Source::read_all(files)?;

        #[cfg(all(feature = "svm-solc", not(target_arch = "wasm32")))]
        if self.auto_detect {
            return project::ProjectCompiler::with_sources(self, sources)?.compile()
        }

        let solc = self.configure_solc(self.solc.clone());
        self.compile_with_version(&solc, sources)
    }

    /// Convenience function to compile only (re)compile files that match the provided [FileFilter].
    /// Same as [`Self::compile()`] but with only with those files as input that match
    /// [FileFilter::is_match()].
    ///
    /// # Example - Only compile Test files
    ///
    /// ```
    /// use ethers_solc::{Project, TestFileFilter};
    /// # fn demo(project: Project) {
    /// let project = Project::builder().build().unwrap();
    /// let output = project
    ///     .compile_sparse(
    ///         TestFileFilter::default()
    ///     ).unwrap();
    /// # }
    /// ```
    ///
    /// # Example - Apply a custom filter
    ///
    /// ```
    /// use std::path::Path;
    /// use ethers_solc::Project;
    /// # fn demo(project: Project) {
    /// let project = Project::builder().build().unwrap();
    /// let output = project
    ///     .compile_sparse(
    ///         |path: &Path| path.ends_with("Greeter.sol")
    ///     ).unwrap();
    /// # }
    /// ```
    pub fn compile_sparse<F: FileFilter + 'static>(
        &self,
        filter: F,
    ) -> Result<ProjectCompileOutput<T>> {
        let sources =
            Source::read_all(self.paths.input_files().into_iter().filter(|p| filter.is_match(p)))?;
        let filter: Box<dyn FileFilter> = Box::new(filter);

        #[cfg(all(feature = "svm-solc", not(target_arch = "wasm32")))]
        if self.auto_detect {
            return project::ProjectCompiler::with_sources(self, sources)?
                .with_sparse_output(filter)
                .compile()
        }

        project::ProjectCompiler::with_sources_and_solc(self, sources, self.solc.clone())?
            .with_sparse_output(filter)
            .compile()
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
    ) -> Result<ProjectCompileOutput<T>> {
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

    /// Returns standard-json-input to compile the target contract
    pub fn standard_json_input(
        &self,
        target: impl AsRef<Path>,
    ) -> Result<StandardJsonCompilerInput> {
        use path_slash::PathExt;

        let target = target.as_ref();
        tracing::trace!("Building standard-json-input for {:?}", target);
        let graph = Graph::resolve(&self.paths)?;
        let target_index = graph.files().get(target).ok_or_else(|| {
            SolcError::msg(format!("cannot resolve file at {:?}", target.display()))
        })?;

        let mut sources = Vec::new();
        let mut unique_paths = HashSet::new();
        let (path, source) = graph.node(*target_index).unpack();
        unique_paths.insert(path.clone());
        sources.push((path, source));
        sources.extend(
            graph
                .all_imported_nodes(*target_index)
                .map(|index| graph.node(index).unpack())
                .filter(|(p, _)| unique_paths.insert(p.to_path_buf())),
        );

        let root = self.root();
        let sources = sources
            .into_iter()
            .map(|(path, source)| {
                let path: PathBuf = if let Ok(stripped) = path.strip_prefix(root) {
                    stripped.to_slash_lossy().into_owned().into()
                } else {
                    path.to_slash_lossy().into_owned().into()
                };
                (path, source.clone())
            })
            .collect();

        let mut settings = self.solc_config.settings.clone();
        // strip the path to the project root from all remappings
        settings.remappings = self
            .paths
            .remappings
            .clone()
            .into_iter()
            .map(|r| r.into_relative(self.root()).to_relative_remapping())
            .collect::<Vec<_>>();

        let input = StandardJsonCompilerInput::new(sources, settings);

        Ok(input)
    }
}

pub struct ProjectBuilder<T: ArtifactOutput = ConfigurableArtifacts> {
    /// The layout of the
    paths: Option<ProjectPathsConfig>,
    /// Where to find solc
    solc: Option<Solc>,
    /// How solc invocation should be configured.
    solc_config: Option<SolcConfig>,
    /// Whether caching is enabled, default is true.
    cached: bool,
    /// Whether to output build information with each solc call.
    build_info: bool,
    /// Whether writing artifacts to disk is enabled, default is true.
    no_artifacts: bool,
    /// Whether automatic solc version detection is enabled
    auto_detect: bool,
    /// Use offline mode
    offline: bool,
    /// Whether to slash paths of the `ProjectCompilerOutput`
    slash_paths: bool,
    /// handles all artifacts related tasks
    artifacts: T,
    /// Which error codes to ignore
    pub ignored_error_codes: Vec<u64>,
    /// The minimum severity level that is treated as a compiler error
    compiler_severity_filter: Severity,
    /// All allowed paths for solc's `--allowed-paths`
    allowed_paths: AllowedLibPaths,
    /// Paths to use for solc's `--include-path`
    include_paths: IncludePaths,
    solc_jobs: Option<usize>,
}

impl<T: ArtifactOutput> ProjectBuilder<T> {
    /// Create a new builder with the given artifacts handler
    pub fn new(artifacts: T) -> Self {
        Self {
            paths: None,
            solc: None,
            solc_config: None,
            cached: true,
            build_info: false,
            no_artifacts: false,
            auto_detect: true,
            offline: false,
            slash_paths: true,
            artifacts,
            ignored_error_codes: Vec::new(),
            compiler_severity_filter: Severity::Error,
            allowed_paths: Default::default(),
            include_paths: Default::default(),
            solc_jobs: None,
        }
    }

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

    #[must_use]
    pub fn set_compiler_severity_filter(mut self, compiler_severity_filter: Severity) -> Self {
        self.compiler_severity_filter = compiler_severity_filter;
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

    /// Sets the build info value
    #[must_use]
    pub fn set_build_info(mut self, build_info: bool) -> Self {
        self.build_info = build_info;
        self
    }

    /// Activates offline mode
    ///
    /// Prevents network possible access to download/check solc installs
    #[must_use]
    pub fn offline(self) -> Self {
        self.set_offline(true)
    }

    /// Sets the offline status
    #[must_use]
    pub fn set_offline(mut self, offline: bool) -> Self {
        self.offline = offline;
        self
    }

    /// Sets whether to slash all paths on windows
    ///
    /// If set to `true` all `\\` separators are replaced with `/`, same as solc
    #[must_use]
    pub fn set_slashed_paths(mut self, slashed_paths: bool) -> Self {
        self.slash_paths = slashed_paths;
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
    pub fn artifacts<A: ArtifactOutput>(self, artifacts: A) -> ProjectBuilder<A> {
        let ProjectBuilder {
            paths,
            solc,
            solc_config,
            cached,
            no_artifacts,
            auto_detect,
            ignored_error_codes,
            compiler_severity_filter,
            allowed_paths,
            include_paths,
            solc_jobs,
            offline,
            build_info,
            slash_paths,
            ..
        } = self;
        ProjectBuilder {
            paths,
            solc,
            solc_config,
            cached,
            no_artifacts,
            auto_detect,
            offline,
            slash_paths,
            artifacts,
            ignored_error_codes,
            compiler_severity_filter,
            allowed_paths,
            include_paths,
            solc_jobs,
            build_info,
        }
    }

    /// Adds an allowed-path to the solc executable
    #[must_use]
    pub fn allowed_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.allowed_paths.insert(path.into());
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

    /// Adds an `--include-path` to the solc executable
    #[must_use]
    pub fn include_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.include_paths.insert(path.into());
        self
    }

    /// Adds multiple include-path to the solc executable
    #[must_use]
    pub fn include_paths<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<PathBuf>,
    {
        for arg in args {
            self = self.include_path(arg);
        }
        self
    }

    pub fn build(self) -> Result<Project<T>> {
        let Self {
            paths,
            solc,
            solc_config,
            cached,
            no_artifacts,
            auto_detect,
            artifacts,
            ignored_error_codes,
            compiler_severity_filter,
            mut allowed_paths,
            include_paths,
            solc_jobs,
            offline,
            build_info,
            slash_paths,
        } = self;

        let mut paths = paths.map(Ok).unwrap_or_else(ProjectPathsConfig::current_hardhat)?;

        if slash_paths {
            // ensures we always use `/` paths
            paths.slash_paths();
        }

        let solc = solc.unwrap_or_default();
        let solc_config = solc_config.unwrap_or_else(|| SolcConfig::builder().build());

        // allow every contract under root by default
        allowed_paths.insert(paths.root.clone());

        Ok(Project {
            paths,
            solc,
            solc_config,
            cached,
            build_info,
            no_artifacts,
            auto_detect,
            artifacts,
            ignored_error_codes,
            compiler_severity_filter,
            allowed_paths,
            include_paths,
            solc_jobs: solc_jobs.unwrap_or_else(num_cpus::get),
            offline,
            slash_paths,
        })
    }
}

impl<T: ArtifactOutput + Default> Default for ProjectBuilder<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ArtifactOutput> ArtifactOutput for Project<T> {
    type Artifact = T::Artifact;

    fn on_output(
        &self,
        contracts: &VersionedContracts,
        sources: &VersionedSourceFiles,
        layout: &ProjectPathsConfig,
        ctx: OutputContext,
    ) -> Result<Artifacts<Self::Artifact>> {
        self.artifacts_handler().on_output(contracts, sources, layout, ctx)
    }

    fn write_contract_extras(&self, contract: &Contract, file: &Path) -> Result<()> {
        self.artifacts_handler().write_contract_extras(contract, file)
    }

    fn write_extras(
        &self,
        contracts: &VersionedContracts,
        artifacts: &Artifacts<Self::Artifact>,
    ) -> Result<()> {
        self.artifacts_handler().write_extras(contracts, artifacts)
    }

    fn output_file_name(name: impl AsRef<str>) -> PathBuf {
        T::output_file_name(name)
    }

    fn output_file_name_versioned(name: impl AsRef<str>, version: &Version) -> PathBuf {
        T::output_file_name_versioned(name, version)
    }

    fn output_file(contract_file: impl AsRef<Path>, name: impl AsRef<str>) -> PathBuf {
        T::output_file(contract_file, name)
    }

    fn output_file_versioned(
        contract_file: impl AsRef<Path>,
        name: impl AsRef<str>,
        version: &Version,
    ) -> PathBuf {
        T::output_file_versioned(contract_file, name, version)
    }

    fn contract_name(file: impl AsRef<Path>) -> Option<String> {
        T::contract_name(file)
    }

    fn output_exists(
        contract_file: impl AsRef<Path>,
        name: impl AsRef<str>,
        root: impl AsRef<Path>,
    ) -> bool {
        T::output_exists(contract_file, name, root)
    }

    fn read_cached_artifact(path: impl AsRef<Path>) -> Result<Self::Artifact> {
        T::read_cached_artifact(path)
    }

    fn read_cached_artifacts<P, I>(files: I) -> Result<BTreeMap<PathBuf, Self::Artifact>>
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        T::read_cached_artifacts(files)
    }

    fn contract_to_artifact(
        &self,
        file: &str,
        name: &str,
        contract: Contract,
        source_file: Option<&SourceFile>,
    ) -> Self::Artifact {
        self.artifacts_handler().contract_to_artifact(file, name, contract, source_file)
    }

    fn output_to_artifacts(
        &self,
        contracts: &VersionedContracts,
        sources: &VersionedSourceFiles,
        ctx: OutputContext,
        layout: &ProjectPathsConfig,
    ) -> Artifacts<Self::Artifact> {
        self.artifacts_handler().output_to_artifacts(contracts, sources, ctx, layout)
    }

    fn standalone_source_file_to_artifact(
        &self,
        path: &str,
        file: &VersionedSourceFile,
    ) -> Option<Self::Artifact> {
        self.artifacts_handler().standalone_source_file_to_artifact(path, file)
    }
}

#[cfg(test)]
#[cfg(all(feature = "svm-solc", not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::remappings::Remapping;

    #[test]
    fn test_build_all_versions() {
        let paths = ProjectPathsConfig::builder()
            .root("./test-data/test-contract-versions")
            .sources("./test-data/test-contract-versions")
            .build()
            .unwrap();
        let project = Project::builder().paths(paths).no_artifacts().ephemeral().build().unwrap();
        let contracts = project.compile().unwrap().succeeded().output().contracts;
        // Contracts A to F
        assert_eq!(contracts.contracts().count(), 5);
    }

    #[test]
    fn test_build_many_libs() {
        let root = utils::canonicalize("./test-data/test-contract-libs").unwrap();

        let paths = ProjectPathsConfig::builder()
            .root(&root)
            .sources(root.join("src"))
            .lib(root.join("lib1"))
            .lib(root.join("lib2"))
            .remappings(
                Remapping::find_many(root.join("lib1"))
                    .into_iter()
                    .chain(Remapping::find_many(root.join("lib2"))),
            )
            .build()
            .unwrap();
        let project = Project::builder()
            .paths(paths)
            .no_artifacts()
            .ephemeral()
            .no_artifacts()
            .build()
            .unwrap();
        let contracts = project.compile().unwrap().succeeded().output().contracts;
        assert_eq!(contracts.contracts().count(), 3);
    }

    #[test]
    fn test_build_remappings() {
        let root = utils::canonicalize("./test-data/test-contract-remappings").unwrap();
        let paths = ProjectPathsConfig::builder()
            .root(&root)
            .sources(root.join("src"))
            .lib(root.join("lib"))
            .remappings(Remapping::find_many(root.join("lib")))
            .build()
            .unwrap();
        let project = Project::builder().no_artifacts().paths(paths).ephemeral().build().unwrap();
        let contracts = project.compile().unwrap().succeeded().output().contracts;
        assert_eq!(contracts.contracts().count(), 2);
    }
}
