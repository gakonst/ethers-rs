// pub mod artifacts;
// pub mod sourcemap;

// pub use artifacts::{CompilerInput, CompilerOutput, EvmVersion};
// use std::collections::BTreeMap;

// mod artifact_output;
// pub mod cache;
// pub mod hh;
// pub use artifact_output::*;

pub mod resolver;
pub use hh::{HardhatArtifact, HardhatArtifacts};

// TODO: Refactor Graphing
// pub use resolver::Graph;

mod compile;
pub use compile::{
    output::{AggregatedCompilerOutput, ProjectCompileOutput},
    *,
};

mod config;
pub use config::{AllowedLibPaths, PathStyle, ProjectPathsConfig, SolcConfig};

pub mod remappings;
use crate::artifacts::Source;

pub mod error;
pub mod report;
pub mod utils;

use crate::{
    artifacts::{Contract, Sources},
    contracts::VersionedContracts,
    error::{SolcError, SolcIoError},
};
use error::Result;
use semver::Version;
use std::path::{Path, PathBuf};

/// Utilities for creating, mocking and testing of (temporary) projects
#[cfg(feature = "project-util")]
pub mod project_util;

/// Represents a project workspace
#[derive(Debug)]
pub struct Project<T: ArtifactOutput = ConfigurableArtifacts> {
    /// The layout of the
    pub paths: ProjectPathsConfig,
    /// Whether caching is enabled
    pub cached: bool,
    /// Whether writing artifacts to disk is enabled
    pub no_artifacts: bool,
    /// Whether writing artifacts to disk is enabled
    pub auto_detect: bool,
    /// Handles all artifacts related tasks, reading and writing from the artifact dir.
    pub artifacts: T,
    /// Errors/Warnings which match these error codes are not going to be logged
    pub ignored_error_codes: Vec<u64>,
    /// The paths which will be allowed for library inclusion
    pub allowed_lib_paths: AllowedLibPaths,
    /// Maximum number of compiling processes to run simultaneously.
    pub processes: usize,
    /// Offline mode, if set, network access (downloading lang compilers) is disallowed
    pub offline: bool,
}

impl Project {
    /// Convenience function to call `ProjectBuilder::default()`
    ///
    /// # Example
    ///
    /// Configure with `ConfigurableArtifacts` artifacts output
    ///
    /// ```rust
    /// use ethers_compile::Project;
    /// let config = Project::builder().build().unwrap();
    /// ```
    ///
    /// To configure any a project with any `ArtifactOutput` use either
    ///
    /// ```rust
    /// use ethers_compile::Project;
    /// let config = Project::builder().build().unwrap();
    /// ```
    ///
    /// or use the builder directly
    ///
    /// ```rust
    /// use ethers_compile::{ConfigurableArtifacts, ProjectBuilder};
    /// let config = ProjectBuilder::<ConfigurableArtifacts>::default().build().unwrap();
    /// ```
    pub fn builder() -> ProjectBuilder {
        ProjectBuilder::default()
    }
}

// TODO: Refactor ArtifactOutput
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

    /// Returns the root directory of the project
    pub fn root(&self) -> &PathBuf {
        &self.paths.root
    }

    /// Returns the handler that takes care of processing all artifacts
    pub fn artifacts_handler(&self) -> &T {
        &self.artifacts
    }

    /// Sets the maximum number of parallel compiling processes to run simultaneously.
    ///
    /// **Panics if `count == 0`**
    pub fn set_processes(&mut self, count: usize) {
        assert!(count > 0);
        self.processes = Some(count);
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
    pub fn compile(&self) -> Result<ProjectCompileOutput<T>> {
        let sources = self.paths.read_input_files()?;
        tracing::trace!("found {} sources to compile: {:?}", sources.len(), sources.keys());

        #[cfg(all(feature = "svm", feature = "async"))]
        if self.auto_detect {
            tracing::trace!("using solc auto detection to compile sources");
            return self.svm_compile(sources)
        }

        let solc = self.configure_solc(self.solc.clone());

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
    #[cfg(all(feature = "svm", feature = "async"))]
    pub fn compile_file(&self, file: impl Into<PathBuf>) -> Result<ProjectCompileOutput<T>> {
        let file = file.into();
        let source = Source::read(&file)?;
        project::ProjectCompiler::with_sources(self, Sources::from([(file, source)]))?.compile()
    }

    /// Convenience function to compile a series of solidity files with the project's settings.
    /// Same as [`Self::svm_compile()`] but with the given `files` as input.
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
    #[cfg(all(feature = "svm", feature = "async"))]
    pub fn compile_files<P, I>(&self, files: I) -> Result<ProjectCompileOutput<T>>
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        project::ProjectCompiler::with_sources(self, Source::read_all(files)?)?.compile()
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
        project::ProjectCompiler::with_sources_and_solc(
            self,
            sources,
            self.configure_solc(solc.clone()),
        )?
        .compile()
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
