#![doc = include_str ! ("../README.md")]

pub mod artifacts;

pub use artifacts::{CompilerInput, CompilerOutput, EvmVersion};
use std::collections::btree_map::Entry;

pub mod cache;
pub mod hh;
pub use hh::{HardhatArtifact, HardhatArtifacts};

mod compile;

pub use compile::*;

mod config;

pub use config::{
    AllowedLibPaths, Artifact, ArtifactOutput, MinimalCombinedArtifacts, ProjectPathsConfig,
    SolcConfig,
};

pub mod remappings;

use crate::{artifacts::Source, cache::SolFilesCache};

pub mod error;
pub mod utils;

use crate::{
    artifacts::Sources,
    cache::PathMap,
    error::{SolcError, SolcIoError},
};
use error::Result;
use std::{
    borrow::Cow, collections::BTreeMap, convert::TryInto, fmt, fs, marker::PhantomData,
    path::PathBuf,
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

    /// Sets the maximum number of parallel `solc` processes to run simultaneously.
    pub fn set_solc_jobs(&mut self, jobs: usize) {
        assert!(jobs > 0);
        self.solc_jobs = jobs;
    }

    #[tracing::instrument(skip_all, name = "Project::write_cache_file")]
    fn write_cache_file(
        &self,
        sources: Sources,
        artifacts: Vec<(PathBuf, Vec<String>)>,
    ) -> Result<()> {
        tracing::trace!("inserting {} sources in file cache", sources.len());
        let mut cache = SolFilesCache::builder()
            .root(&self.paths.root)
            .solc_config(self.solc_config.clone())
            .insert_files(sources, Some(self.paths.cache.clone()))?;
        tracing::trace!("source files inserted");

        // add the artifacts for each file to the cache entry
        for (file, artifacts) in artifacts {
            if let Some(entry) = cache.files.get_mut(&file) {
                entry.artifacts = artifacts;
            }
        }

        if let Some(cache_dir) = self.paths.cache.parent() {
            tracing::trace!("creating cache file parent directory \"{}\"", cache_dir.display());
            fs::create_dir_all(cache_dir).map_err(|err| SolcError::io(err, cache_dir))?
        }

        tracing::trace!("writing cache file to \"{}\"", self.paths.cache.display());
        cache.write(&self.paths.cache)?;

        Ok(())
    }

    /// Returns all sources found under the project's configured sources path
    #[tracing::instrument(skip_all, fields(name = "sources"))]
    pub fn sources(&self) -> Result<Sources> {
        tracing::trace!("reading all sources from \"{}\"", self.paths.sources.display());
        Ok(Source::read_all_from(&self.paths.sources)?)
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

    /// Attempts to read all unique libraries that are used as imports like "hardhat/console.sol"
    fn resolved_libraries(
        &self,
        sources: &Sources,
    ) -> Result<BTreeMap<PathBuf, (Source, PathBuf)>> {
        let mut libs = BTreeMap::default();
        for source in sources.values() {
            for import in source.parse_imports() {
                if let Some(lib) = utils::resolve_library(&self.paths.libraries, import) {
                    if let Entry::Vacant(entry) = libs.entry(import.into()) {
                        tracing::trace!(
                            "resolved library import \"{}\" at \"{}\"",
                            import,
                            lib.display()
                        );
                        entry.insert((Source::read(&lib)?, lib));
                    }
                }
            }
        }
        Ok(libs)
    }

    /// Attempts to compile the contracts found at the configured location.
    ///
    /// NOTE: this does not check if the contracts were successfully compiled, see
    /// `CompilerOutput::has_error` instead.
    /// NB: If the `svm` feature is enabled, this function will automatically detect
    /// solc versions across files.
    #[tracing::instrument(skip_all, name = "compile")]
    pub fn compile(&self) -> Result<ProjectCompileOutput<Artifacts>> {
        let sources = self.sources()?;
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

    #[cfg(all(feature = "svm", feature = "async"))]
    #[tracing::instrument(skip(self, sources))]
    fn svm_compile(&self, sources: Sources) -> Result<ProjectCompileOutput<Artifacts>> {
        use semver::{Version, VersionReq};
        use std::collections::hash_map::{self, HashMap};

        // split them by version
        let mut sources_by_version = BTreeMap::new();
        // we store the solc versions by path, in case there exists a corrupt solc binary
        let mut solc_versions = HashMap::new();

        // tracks unique version requirements to minimize install effort
        let mut solc_version_req = HashMap::<VersionReq, Version>::new();

        tracing::trace!("preprocessing source files and solc installs");
        for (path, source) in sources.into_iter() {
            // will detect and install the solc version if it's missing
            tracing::trace!("detecting solc version for \"{}\"", path.display());
            let version_req = Solc::version_req(&source)?;

            let version = match solc_version_req.entry(version_req) {
                hash_map::Entry::Occupied(version) => version.get().clone(),
                hash_map::Entry::Vacant(entry) => {
                    let version = Solc::ensure_installed(entry.key())?;
                    entry.insert(version.clone());
                    version
                }
            };
            tracing::trace!("found installed solc \"{}\"", version);

            // gets the solc binary for that version, it is expected tha this will succeed
            // AND find the solc since it was installed right above
            let mut solc = Solc::find_svm_installed_version(version.to_string())?
                .unwrap_or_else(|| panic!("solc \"{}\" should have been installed", version));

            if !self.allowed_lib_paths.0.is_empty() {
                solc = solc.arg("--allow-paths").arg(self.allowed_lib_paths.to_string());
            }
            solc_versions.insert(solc.solc.clone(), version);
            let entry = sources_by_version.entry(solc).or_insert_with(BTreeMap::new);
            entry.insert(path.clone(), source);
        }
        tracing::trace!("solc version preprocessing finished");

        tracing::trace!("verifying solc checksums");
        for solc in sources_by_version.keys() {
            // verify that this solc version's checksum matches the checksum found remotely. If
            // not, re-install the same version.
            let version = &solc_versions[&solc.solc];
            if solc.verify_checksum().is_err() {
                tracing::trace!("corrupted solc version, redownloading  \"{}\"", version);
                Solc::blocking_install(version)?;
                tracing::trace!("reinstalled solc: \"{}\"", version);
            }
        }

        // run the compilation step for each version
        let compiled = if self.solc_jobs > 1 && sources_by_version.len() > 1 {
            self.compile_many(sources_by_version)?
        } else {
            self.compile_sources(sources_by_version)?
        };
        tracing::trace!("compiled all sources");

        Ok(compiled)
    }

    #[cfg(all(feature = "svm", feature = "async"))]
    fn compile_sources(
        &self,
        sources_by_version: BTreeMap<Solc, BTreeMap<PathBuf, Source>>,
    ) -> Result<ProjectCompileOutput<Artifacts>> {
        tracing::trace!("compiling sources using a single solc job");
        let mut compiled =
            ProjectCompileOutput::with_ignored_errors(self.ignored_error_codes.clone());
        for (solc, sources) in sources_by_version {
            tracing::trace!(
                "compiling {} sources with solc \"{}\"",
                sources.len(),
                solc.as_ref().display()
            );
            compiled.extend(self.compile_with_version(&solc, sources)?);
        }
        Ok(compiled)
    }

    #[cfg(all(feature = "svm", feature = "async"))]
    fn compile_many(
        &self,
        sources_by_version: BTreeMap<Solc, BTreeMap<PathBuf, Source>>,
    ) -> Result<ProjectCompileOutput<Artifacts>> {
        tracing::trace!("compile sources in parallel using {} solc jobs", self.solc_jobs);
        let mut compiled =
            ProjectCompileOutput::with_ignored_errors(self.ignored_error_codes.clone());
        let mut paths = PathMap::default();
        let mut jobs = Vec::with_capacity(sources_by_version.len());

        let mut all_sources = BTreeMap::default();
        let mut all_artifacts = Vec::with_capacity(sources_by_version.len());

        // preprocess all sources
        for (solc, sources) in sources_by_version {
            match self.preprocess_sources(sources)? {
                PreprocessedJob::Unchanged(artifacts) => {
                    compiled.extend(ProjectCompileOutput::from_unchanged(artifacts));
                }
                PreprocessedJob::Items(sources, map, cached_artifacts) => {
                    tracing::trace!("cached artifacts: \"{:?}\"", cached_artifacts.keys());
                    tracing::trace!("compile sources: \"{:?}\"", sources.keys());

                    compiled.extend_artifacts(cached_artifacts);
                    // replace absolute path with source name to make solc happy
                    let sources = map.set_source_names(sources);
                    paths.extend(map);

                    let input = CompilerInput::with_sources(sources)
                        .settings(self.solc_config.settings.clone())
                        .normalize_evm_version(&solc.version()?)
                        .with_remappings(self.paths.remappings.clone());

                    jobs.push((solc, input))
                }
            };
        }
        tracing::trace!("execute {} compile jobs in parallel", jobs.len());

        let outputs = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Solc::compile_many(jobs, self.solc_jobs));

        for (res, _, input) in outputs.into_outputs() {
            let output = res?;
            if !output.has_error() {
                if self.cached {
                    // get all contract names of the files and map them to the disk file
                    all_sources.extend(paths.set_disk_paths(input.sources));
                    all_artifacts.extend(paths.get_artifacts(&output.contracts));
                }

                if !self.no_artifacts {
                    Artifacts::on_output(&output, &self.paths)?;
                }
            }
            compiled.extend_output(output);
        }

        // write the cache file
        if self.cached {
            self.write_cache_file(all_sources, all_artifacts)?;
        }

        Ok(compiled)
    }

    /// Compiles the given source files with the exact `Solc` executable
    ///
    /// First all libraries for the sources are resolved by scanning all their imports.
    /// If caching is enabled for the `Project`, then all unchanged files are filtered from the
    /// sources and their existing artifacts are read instead. This will also update the cache
    /// file and cleans up entries for files which may have been removed. Unchanged files that
    /// for which an artifact exist, are not compiled again.
    pub fn compile_with_version(
        &self,
        solc: &Solc,
        sources: Sources,
    ) -> Result<ProjectCompileOutput<Artifacts>> {
        let (sources, paths, cached_artifacts) = match self.preprocess_sources(sources)? {
            PreprocessedJob::Unchanged(artifacts) => {
                return Ok(ProjectCompileOutput::from_unchanged(artifacts))
            }
            PreprocessedJob::Items(a, b, c) => (a, b, c),
        };

        let version = solc.version()?;
        tracing::trace!(
            "compiling {} files with {}. Using {} cached files",
            sources.len(),
            version,
            cached_artifacts.len()
        );
        tracing::trace!("cached artifacts: \"{:?}\"", cached_artifacts.keys());
        tracing::trace!("compile sources: \"{:?}\"", sources.keys());

        // replace absolute path with source name to make solc happy
        let sources = paths.set_source_names(sources);

        let input = CompilerInput::with_sources(sources)
            .settings(self.solc_config.settings.clone())
            .normalize_evm_version(&version)
            .with_remappings(self.paths.remappings.clone());

        tracing::trace!("calling solc with {} sources", input.sources.len());
        let output = solc.compile(&input)?;
        tracing::trace!("compiled input, output has error: {}", output.has_error());

        if output.has_error() {
            return Ok(ProjectCompileOutput::from_compiler_output(
                output,
                self.ignored_error_codes.clone(),
            ))
        }

        if self.cached {
            // get all contract names of the files and map them to the disk file
            let artifacts = paths.get_artifacts(&output.contracts);
            // reapply to disk paths
            let sources = paths.set_disk_paths(input.sources);
            // create cache file
            self.write_cache_file(sources, artifacts)?;
        }

        // TODO: There seems to be some type redundancy here, c.f. discussion with @mattsse
        if !self.no_artifacts {
            Artifacts::on_output(&output, &self.paths)?;
        }

        Ok(ProjectCompileOutput::from_compiler_output_and_cache(
            output,
            cached_artifacts,
            self.ignored_error_codes.clone(),
        ))
    }

    /// Preprocesses the given source files by resolving their libs and check against cache if
    /// configured
    fn preprocess_sources(&self, mut sources: Sources) -> Result<PreprocessedJob<Artifacts>> {
        tracing::trace!("start preprocessing {} sources files", sources.len());

        // keeps track of source names / disk paths
        let mut paths = PathMap::default();

        tracing::trace!("start resolving libraries");
        for (import, (source, path)) in self.resolved_libraries(&sources)? {
            // inserting with absolute path here and keep track of the source name <-> path mappings
            sources.insert(path.clone(), source);
            paths.path_to_source_name.insert(path.clone(), import.clone());
            paths.source_name_to_path.insert(import, path);
        }
        tracing::trace!("resolved all libraries");

        // If there's a cache set, filter to only re-compile the files which were changed
        let (sources, cached_artifacts) = if self.cached && self.paths.cache.exists() {
            tracing::trace!("start reading solfiles cache for incremental compilation");
            let mut cache = SolFilesCache::read(&self.paths.cache)?;
            cache.remove_missing_files();
            let changed_files = cache.get_changed_or_missing_artifacts_files::<Artifacts>(
                sources,
                Some(&self.solc_config),
                &self.paths.artifacts,
            );
            tracing::trace!("detected {} changed files", changed_files.len());
            cache.remove_changed_files(&changed_files);

            let cached_artifacts = if self.paths.artifacts.exists() {
                tracing::trace!("reading artifacts from cache..");
                let artifacts = cache.read_artifacts::<Artifacts>(&self.paths.artifacts)?;
                tracing::trace!("read {} artifacts from cache", artifacts.len());
                artifacts
            } else {
                BTreeMap::default()
            };

            // if nothing changed and all artifacts still exist
            if changed_files.is_empty() {
                tracing::trace!(
                    "unchanged source files, reusing artifacts {:?}",
                    cached_artifacts.keys()
                );
                return Ok(PreprocessedJob::Unchanged(cached_artifacts))
            }
            // There are changed files and maybe some cached files
            (changed_files, cached_artifacts)
        } else {
            (sources, BTreeMap::default())
        };
        Ok(PreprocessedJob::Items(sources, paths, cached_artifacts))
    }

    /// Removes the project's artifacts and cache file
    pub fn cleanup(&self) -> std::result::Result<(), SolcIoError> {
        tracing::trace!("clean up project");
        if self.cache_path().exists() {
            std::fs::remove_file(self.cache_path())
                .map_err(|err| SolcIoError::new(err, self.cache_path()))?;
            tracing::trace!("removed cache file \"{}\"", self.cache_path().display());
        }
        if self.paths.artifacts.exists() {
            std::fs::remove_dir_all(self.artifacts_path())
                .map_err(|err| SolcIoError::new(err, self.artifacts_path().clone()))?;
            tracing::trace!("removed artifacts dir \"{}\"", self.artifacts_path().display());
        }
        Ok(())
    }
}

enum PreprocessedJob<T: ArtifactOutput> {
    Unchanged(BTreeMap<PathBuf, T::Artifact>),
    Items(Sources, PathMap, BTreeMap<PathBuf, T::Artifact>),
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

    /// Disables cached builds
    #[must_use]
    pub fn ephemeral(mut self) -> Self {
        self.cached = false;
        self
    }

    /// Disables writing artifacts to disk
    #[must_use]
    pub fn no_artifacts(mut self) -> Self {
        self.no_artifacts = true;
        self
    }

    /// Disables automatic solc version detection
    #[must_use]
    pub fn no_auto_detect(mut self) -> Self {
        self.auto_detect = false;
        self
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
        let solc_config = solc_config.map(Ok).unwrap_or_else(|| SolcConfig::builder().build())?;

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
    /// use std::collections::BTreeMap;
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
        if let Some(output) = self.compiler_output.as_ref() {
            output.has_error()
        } else {
            false
        }
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
    /// All artifacts together with their contract name
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::BTreeMap;
    /// use ethers_solc::artifacts::CompactContract;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let contracts: BTreeMap<String, CompactContract> = project.compile().unwrap().into_artifacts().collect();
    /// ```
    pub fn into_artifacts(mut self) -> Box<dyn Iterator<Item = (String, T::Artifact)>> {
        let artifacts = self.artifacts.into_iter().filter_map(|(path, art)| {
            T::contract_name(&path)
                .map(|name| (format!("{:?}:{}", path.file_name().unwrap(), name), art))
        });

        let artifacts: Box<dyn Iterator<Item = (String, T::Artifact)>> =
            if let Some(output) = self.compiler_output.take() {
                Box::new(artifacts.chain(T::output_to_artifacts(output).into_values().flatten()))
            } else {
                Box::new(artifacts)
            };
        artifacts
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

        let root = dunce::canonicalize("./test-data/test-contract-libs").unwrap();

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

        let root = dunce::canonicalize("./test-data/test-contract-remappings").unwrap();
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
