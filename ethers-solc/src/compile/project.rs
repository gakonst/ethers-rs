//! Manages compiling of a `Project`
//!
//! The compilation of a project is performed in several steps.
//!
//! First the project's dependency graph [`crate::Graph`] is constructed and all imported
//! dependencies are resolved. The graph holds all the relationships between the files and their
//! versions. From there the appropriate version set is derived
//! [`crate::Graph::into_sources_by_version()`] which need to be compiled with different
//! [`crate::Solc`] versions.
//!
//! At this point we check if we need to compile a source file or whether we can reuse an _existing_
//! `Artifact`. We don't to compile if:
//!     - caching is enabled
//!     - the file is **not** dirty [`Cache::is_dirty()`]
//!     - the artifact for that file exists
//!
//! This concludes the preprocessing, and we now have either
//!    - only `Source` files that need to be compiled
//!    - only cached `Artifacts`, compilation can be skipped. This is considered an unchanged,
//!      cached project
//!    - Mix of both `Source` and `Artifacts`, only the `Source` files need to be compiled, the
//!      `Artifacts` can be reused.
//!
//! The final step is invoking `Solc` via the standard JSON format.
//!
//! ### Notes on [Import Path Resolution](https://docs.soliditylang.org/en/develop/path-resolution.html#path-resolution)
//!
//! In order to be able to support reproducible builds on all platforms, the Solidity compiler has
//! to abstract away the details of the filesystem where source files are stored. Paths used in
//! imports must work the same way everywhere while the command-line interface must be able to work
//! with platform-specific paths to provide good user experience. This section aims to explain in
//! detail how Solidity reconciles these requirements.
//!
//! The compiler maintains an internal database (virtual filesystem or VFS for short) where each
//! source unit is assigned a unique source unit name which is an opaque and unstructured
//! identifier. When you use the import statement, you specify an import path that references a
//! source unit name. If the compiler does not find any source unit name matching the import path in
//! the VFS, it invokes the callback, which is responsible for obtaining the source code to be
//! placed under that name.
//!
//! This becomes relevant when dealing with resolved imports
//!
//! #### Relative Imports
//!
//! ```solidity
//! import "./math/math.sol";
//! import "contracts/tokens/token.sol";
//! ```
//! In the above `./math/math.sol` and `contracts/tokens/token.sol` are import paths while the
//! source unit names they translate to are `contracts/math/math.sol` and
//! `contracts/tokens/token.sol` respectively.
//!
//! #### Direct Imports
//!
//! An import that does not start with `./` or `../` is a direct import.
//!
//! ```solidity
//! import "/project/lib/util.sol";         // source unit name: /project/lib/util.sol
//! import "lib/util.sol";                  // source unit name: lib/util.sol
//! import "@openzeppelin/address.sol";     // source unit name: @openzeppelin/address.sol
//! import "https://example.com/token.sol"; // source unit name: https://example.com/token.sol
//! ```
//!
//! After applying any import remappings the import path simply becomes the source unit name.
//!
//! ##### Import Remapping
//!
//! ```solidity
//! import "github.com/ethereum/dapp-bin/library/math.sol"; // source unit name: dapp-bin/library/math.sol
//! ```
//!
//! If compiled with `solc github.com/ethereum/dapp-bin/=dapp-bin/` the compiler will look for the
//! file in the VFS under `dapp-bin/library/math.sol`. If the file is not available there, the
//! source unit name will be passed to the Host Filesystem Loader, which will then look in
//! `/project/dapp-bin/library/iterable_mapping.sol`

use crate::{
    artifact_output::Artifacts,
    artifacts::{Settings, VersionedSources},
    cache::ArtifactsCache,
    error::Result,
    output::AggregatedCompilerOutput,
    report,
    resolver::GraphEdges,
    ArtifactOutput, CompilerInput, Graph, Project, ProjectCompileOutput, ProjectPathsConfig, Solc,
    Sources,
};
use rayon::prelude::*;

use std::collections::btree_map::BTreeMap;

#[derive(Debug)]
pub struct ProjectCompiler<'a, T: ArtifactOutput> {
    /// Contains the relationship of the source files and their imports
    edges: GraphEdges,
    project: &'a Project<T>,
    /// how to compile all the sources
    sources: CompilerSources,
}

impl<'a, T: ArtifactOutput> ProjectCompiler<'a, T> {
    /// Create a new `ProjectCompiler` to bootstrap the compilation process of the project's
    /// sources.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let output = project.compile().unwrap();
    /// ```
    #[cfg(all(feature = "svm", feature = "async"))]
    pub fn new(project: &'a Project<T>) -> Result<Self> {
        Self::with_sources(project, project.paths.read_input_files()?)
    }

    /// Bootstraps the compilation process by resolving the dependency graph of all sources and the
    /// appropriate `Solc` -> `Sources` set as well as the compile mode to use (parallel,
    /// sequential)
    ///
    /// Multiple (`Solc` -> `Sources`) pairs can be compiled in parallel if the `Project` allows
    /// multiple `jobs`, see [`crate::Project::set_solc_jobs()`].
    #[cfg(all(feature = "svm", feature = "async"))]
    pub fn with_sources(project: &'a Project<T>, sources: Sources) -> Result<Self> {
        let graph = Graph::resolve_sources(&project.paths, sources)?;
        let (versions, edges) = graph.into_sources_by_version(project.offline)?;
        let sources_by_version = versions.get(&project.allowed_lib_paths)?;

        let sources = if project.solc_jobs > 1 && sources_by_version.len() > 1 {
            // if there are multiple different versions, and we can use multiple jobs we can compile
            // them in parallel
            CompilerSources::Parallel(sources_by_version, project.solc_jobs)
        } else {
            CompilerSources::Sequential(sources_by_version)
        };

        Ok(Self { edges, project, sources })
    }

    /// Compiles the sources with a pinned `Solc` instance
    pub fn with_sources_and_solc(
        project: &'a Project<T>,
        sources: Sources,
        solc: Solc,
    ) -> Result<Self> {
        let version = solc.version()?;
        let (sources, edges) = Graph::resolve_sources(&project.paths, sources)?.into_sources();
        let sources_by_version = BTreeMap::from([(solc, (version, sources))]);
        let sources = CompilerSources::Sequential(sources_by_version);

        Ok(Self { edges, project, sources })
    }

    /// Compiles all the sources of the `Project` in the appropriate mode
    ///
    /// If caching is enabled, the sources are filtered and only _dirty_ sources are recompiled.
    ///
    /// The output of the compile process can be a mix of reused artifacts and freshly compiled
    /// `Contract`s
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let output = project.compile().unwrap();
    /// ```
    pub fn compile(self) -> Result<ProjectCompileOutput<T>> {
        // drive the compiler statemachine to completion
        self.preprocess()?.compile()?.write_artifacts()?.write_cache()
    }

    /// Does basic preprocessing
    ///   - sets proper source unit names
    ///   - check cache
    fn preprocess(self) -> Result<PreprocessedState<'a, T>> {
        let Self { edges, project, mut sources } = self;

        let mut cache = ArtifactsCache::new(project, edges)?;
        // retain and compile only dirty sources and all their imports
        sources = sources.filtered(&mut cache);

        Ok(PreprocessedState { sources, cache })
    }
}

/// A series of states that comprise the [`ProjectCompiler::compile()`] state machine
///
/// The main reason is to debug all states individually
#[derive(Debug)]
struct PreprocessedState<'a, T: ArtifactOutput> {
    sources: CompilerSources,
    cache: ArtifactsCache<'a, T>,
}

impl<'a, T: ArtifactOutput> PreprocessedState<'a, T> {
    /// advance to the next state by compiling all sources
    fn compile(self) -> Result<CompiledState<'a, T>> {
        let PreprocessedState { sources, cache } = self;
        let output =
            sources.compile(&cache.project().solc_config.settings, &cache.project().paths)?;

        Ok(CompiledState { output, cache })
    }
}

/// Represents the state after `solc` was successfully invoked
#[derive(Debug)]
struct CompiledState<'a, T: ArtifactOutput> {
    output: AggregatedCompilerOutput,
    cache: ArtifactsCache<'a, T>,
}

impl<'a, T: ArtifactOutput> CompiledState<'a, T> {
    /// advance to the next state by handling all artifacts
    ///
    /// Writes all output contracts to disk if enabled in the `Project` and if the build was
    /// successful
    fn write_artifacts(self) -> Result<ArtifactsState<'a, T>> {
        let CompiledState { output, cache } = self;

        // write all artifacts via the handler but only if the build succeeded
        let compiled_artifacts = if cache.project().no_artifacts {
            cache.project().artifacts_handler().output_to_artifacts(&output.contracts)
        } else if output.has_error() {
            tracing::trace!("skip writing cache file due to solc errors: {:?}", output.errors);
            cache.project().artifacts_handler().output_to_artifacts(&output.contracts)
        } else {
            cache
                .project()
                .artifacts_handler()
                .on_output(&output.contracts, &cache.project().paths)?
        };

        Ok(ArtifactsState { output, cache, compiled_artifacts })
    }
}

/// Represents the state after all artifacts were written to disk
#[derive(Debug)]
struct ArtifactsState<'a, T: ArtifactOutput> {
    output: AggregatedCompilerOutput,
    cache: ArtifactsCache<'a, T>,
    compiled_artifacts: Artifacts<T::Artifact>,
}

impl<'a, T: ArtifactOutput> ArtifactsState<'a, T> {
    /// Writes the cache file
    ///
    /// this concludes the [`Project::compile()`] statemachine
    fn write_cache(self) -> Result<ProjectCompileOutput<T>> {
        let ArtifactsState { output, cache, compiled_artifacts } = self;
        let ignored_error_codes = cache.project().ignored_error_codes.clone();
        let cached_artifacts = cache.write_cache(&compiled_artifacts)?;
        Ok(ProjectCompileOutput {
            compiler_output: output,
            compiled_artifacts,
            cached_artifacts,
            ignored_error_codes,
        })
    }
}

/// Determines how the `solc <-> sources` pairs are executed
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum CompilerSources {
    /// Compile all these sequentially
    Sequential(VersionedSources),
    /// Compile all these in parallel using a certain amount of jobs
    Parallel(VersionedSources, usize),
}

impl CompilerSources {
    /// Filters out all sources that don't need to be compiled, see [`ArtifactsCache::filter`]
    fn filtered<T: ArtifactOutput>(self, cache: &mut ArtifactsCache<T>) -> Self {
        fn filtered_sources<T: ArtifactOutput>(
            sources: VersionedSources,
            cache: &mut ArtifactsCache<T>,
        ) -> VersionedSources {
            sources
                .into_iter()
                .map(|(solc, (version, sources))| {
                    let sources = cache.filter(sources, &version);
                    (solc, (version, sources))
                })
                .collect()
        }

        match self {
            CompilerSources::Sequential(s) => {
                CompilerSources::Sequential(filtered_sources(s, cache))
            }
            CompilerSources::Parallel(s, j) => {
                CompilerSources::Parallel(filtered_sources(s, cache), j)
            }
        }
    }

    /// Compiles all the files with `Solc`
    fn compile(
        self,
        settings: &Settings,
        paths: &ProjectPathsConfig,
    ) -> Result<AggregatedCompilerOutput> {
        match self {
            CompilerSources::Sequential(input) => compile_sequential(input, settings, paths),
            CompilerSources::Parallel(input, j) => compile_parallel(input, j, settings, paths),
        }
    }

    #[cfg(test)]
    #[allow(unused)]
    fn sources(&self) -> &VersionedSources {
        match self {
            CompilerSources::Sequential(v) => v,
            CompilerSources::Parallel(v, _) => v,
        }
    }
}

/// Compiles the input set sequentially and returns an aggregated set of the solc `CompilerOutput`s
fn compile_sequential(
    input: VersionedSources,
    settings: &Settings,
    paths: &ProjectPathsConfig,
) -> Result<AggregatedCompilerOutput> {
    let mut aggregated = AggregatedCompilerOutput::default();
    tracing::trace!("compiling {} jobs sequentially", input.len());
    for (solc, (version, sources)) in input {
        if sources.is_empty() {
            // nothing to compile
            continue
        }
        tracing::trace!(
            "compiling {} sources with solc \"{}\" {:?}",
            sources.len(),
            solc.as_ref().display(),
            solc.args
        );

        for input in CompilerInput::with_sources(sources) {
            let input = input
                .settings(settings.clone())
                .normalize_evm_version(&version)
                .with_remappings(paths.remappings.clone());
            tracing::trace!(
                "calling solc `{}` with {} sources {:?}",
                version,
                input.sources.len(),
                input.sources.keys()
            );
            report::solc_spawn(&solc, &version, &input);
            let output = solc.compile_exact(&input)?;
            report::solc_success(&solc, &version, &output);
            tracing::trace!("compiled input, output has error: {}", output.has_error());
            aggregated.extend(version.clone(), output);
        }
    }
    Ok(aggregated)
}

/// compiles the input set using `num_jobs` threads
fn compile_parallel(
    input: VersionedSources,
    num_jobs: usize,
    settings: &Settings,
    paths: &ProjectPathsConfig,
) -> Result<AggregatedCompilerOutput> {
    debug_assert!(num_jobs > 1);
    tracing::trace!(
        "compile {} sources in parallel using up to {} solc jobs",
        input.len(),
        num_jobs
    );

    let mut jobs = Vec::with_capacity(input.len());
    for (solc, (version, sources)) in input {
        if sources.is_empty() {
            // nothing to compile
            continue
        }
        for input in CompilerInput::with_sources(sources) {
            let job = input
                .settings(settings.clone())
                .normalize_evm_version(&version)
                .with_remappings(paths.remappings.clone());

            jobs.push((solc.clone(), version.clone(), job))
        }
    }

    // start a rayon threadpool that will execute all `Solc::compile()` processes
    let pool = rayon::ThreadPoolBuilder::new().num_threads(num_jobs).build().unwrap();
    let outputs = pool.install(move || {
        jobs.into_par_iter()
            .map(|(solc, version, input)| {
                tracing::trace!(
                    "calling solc `{}` {:?} with {} sources: {:?}",
                    version,
                    solc.args,
                    input.sources.len(),
                    input.sources.keys()
                );
                report::solc_spawn(&solc, &version, &input);
                solc.compile(&input).map(move |output| {
                    report::solc_success(&solc, &version, &output);
                    (version, output)
                })
            })
            .collect::<Result<Vec<_>>>()
    })?;

    let mut aggregated = AggregatedCompilerOutput::default();
    aggregated.extend_all(outputs);

    Ok(aggregated)
}

#[cfg(test)]
#[cfg(feature = "project-util")]
mod tests {
    use super::*;
    use crate::{project_util::TempProject, MinimalCombinedArtifacts};

    use std::path::PathBuf;

    #[allow(unused)]
    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init()
            .ok();
    }

    #[test]
    fn can_preprocess() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/dapp-sample");
        let project =
            Project::builder().paths(ProjectPathsConfig::dapptools(root).unwrap()).build().unwrap();

        let compiler = ProjectCompiler::new(&project).unwrap();
        let prep = compiler.preprocess().unwrap();
        let cache = prep.cache.as_cached().unwrap();
        // 3 contracts
        assert_eq!(cache.dirty_source_files.len(), 3);
        assert!(cache.filtered.is_empty());
        assert!(cache.cache.is_empty());

        let compiled = prep.compile().unwrap();
        assert_eq!(compiled.output.contracts.files().count(), 3);
    }

    #[test]
    fn can_detect_cached_files() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/dapp-sample");
        let paths = ProjectPathsConfig::builder().sources(root.join("src")).lib(root.join("lib"));
        let project = TempProject::<MinimalCombinedArtifacts>::new(paths).unwrap();

        let compiled = project.compile().unwrap();
        assert!(!compiled.has_compiler_errors());

        let inner = project.project();
        let compiler = ProjectCompiler::new(inner).unwrap();
        let prep = compiler.preprocess().unwrap();
        assert!(prep.cache.as_cached().unwrap().dirty_source_files.is_empty())
    }

    #[test]
    #[ignore]
    fn can_compile_real_project() {
        init_tracing();
        let paths = ProjectPathsConfig::builder()
            .root("../../foundry-integration-tests/testdata/solmate")
            .build()
            .unwrap();
        let project = Project::builder().paths(paths).build().unwrap();
        let compiler = ProjectCompiler::new(&project).unwrap();
        let out = compiler.compile().unwrap();
        println!("{}", out);
    }
}
