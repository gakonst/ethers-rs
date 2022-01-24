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
//! The compiler will look for the file in the VFS under `dapp-bin/library/math.sol`. If the file is
//! not available there, the source unit name will be passed to the Host Filesystem Loader, which
//! will then look in `/project/dapp-bin/library/iterable_mapping.sol`

use crate::{
    artifacts::{Error, Settings, SourceFile, VersionedContract, VersionedContracts},
    error::Result,
    remappings::Remapping,
    resolver::GraphEdges,
    utils, ArtifactOutput, CompilerInput, CompilerOutput, Graph, PathMap, Project,
    ProjectPathsConfig, SolFilesCache, Solc, SolcConfig, Source, Sources,
};
use semver::Version;
use std::{
    collections::{hash_map, BTreeMap, HashMap, HashSet},
    path::{Path, PathBuf},
};

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
    /// ```
    pub fn new(project: &'a Project<T>) -> Result<Self> {
        Self::with_sources(project, project.paths.read_input_files()?)
    }

    /// Bootstraps the compilation process by resolving the dependency graph of all sources and the
    /// appropriate `Solc` -> `Sources` set as well as the compile mode to use (parallel,
    /// sequential)
    ///
    /// Multiple (`Solc` -> `Sources`) pairs can be compiled in parallel if the `Project` allows
    /// multiple `jobs`, see [`crate::Project::set_solc_jobs()`].
    pub fn with_sources(project: &'a Project<T>, sources: Sources) -> Result<Self> {
        let graph = Graph::resolve_sources(&project.paths, sources)?;
        let (versions, edges) = graph.into_sources_by_version(!project.auto_detect)?;

        let sources_by_version = versions.get(&project.allowed_lib_paths)?;

        let sources = if project.solc_jobs > 1 && sources_by_version.len() > 1 {
            // if there are multiple different versions and we can use multiple jobs we can compile
            // them in parallel
            CompilerSources::Parallel(sources_by_version, project.solc_jobs)
        } else {
            CompilerSources::Sequential(sources_by_version)
        };

        Ok(Self { edges, project, sources })
    }

    /// Compiles all the sources of the `Project` in the appropriate mode
    ///
    /// If caching is enabled, the sources are filtered and only _dirty_ sources are recompiled.
    ///
    /// The output of the compile process can be a mix of reused artifacts and freshly compiled
    /// `Contract`s
    pub fn compile(self) -> Result<ProjectCompileOutput2<T>> {
        let Self { edges, project, mut sources } = self;

        let mut cache = ArtifactsCache::new(&project, &edges)?;

        // retain only dirty sources
        sources = sources.filtered(&mut cache);

        let output = sources.compile(&project.solc_config.settings, &project.paths)?;

        // TODO rebuild cache
        // TODO write artifacts

        Ok(ProjectCompileOutput2 {
            output,
            cached_artifacts: Default::default(),
            ignored_error_codes: vec![],
        })
    }
}

/// Determines how the `solc <-> sources` pairs are executed
#[derive(Debug)]
enum CompilerSources {
    /// Compile all these sequentially
    Sequential(BTreeMap<Solc, Sources>),
    /// Compile all these in parallel using a certain amount of jobs
    Parallel(BTreeMap<Solc, Sources>, usize),
}

impl CompilerSources {
    /// Filters out all sources that don't need to be compiled, see [`ArtifactsCache::filter`]
    fn filtered<T: ArtifactOutput>(mut self, cache: &mut ArtifactsCache<T>) -> Self {
        fn filterd_sources<T: ArtifactOutput>(
            sources: BTreeMap<Solc, Sources>,
            cache: &mut ArtifactsCache<T>,
        ) -> BTreeMap<Solc, Sources> {
            sources.into_iter().map(|(solc, sources)| (solc, cache.filter(sources))).collect()
        }

        match self {
            CompilerSources::Sequential(s) => {
                CompilerSources::Sequential(filterd_sources(s, cache))
            }
            CompilerSources::Parallel(s, j) => {
                CompilerSources::Parallel(filterd_sources(s, cache), j)
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
}

/// Compiles the input set sequentially and returns an aggregated set of the solc `CompilerOutput`s
fn compile_sequential(
    input: BTreeMap<Solc, Sources>,
    settings: &Settings,
    paths: &ProjectPathsConfig,
) -> Result<AggregatedCompilerOutput> {
    let mut aggregated = AggregatedCompilerOutput::default();
    for (solc, sources) in input {
        let version = solc.version()?;

        tracing::trace!(
            "compiling {} sources with solc \"{}\"",
            sources.len(),
            solc.as_ref().display()
        );

        let mut source_unit_map = PathMap::default();
        // replace absolute path with source name to make solc happy
        // TODO use correct path
        let sources = source_unit_map.set_source_names(sources);

        let input = CompilerInput::with_sources(sources)
            .settings(settings.clone())
            .normalize_evm_version(&version)
            .with_remappings(paths.remappings.clone());

        tracing::trace!("calling solc `{}` with {} sources", version, input.sources.len());
        let mut output = solc.compile(&input)?;
        tracing::trace!("compiled input, output has error: {}", output.has_error());

        // TODO reapply the paths?

        aggregated.extend(version, output);
    }
    Ok(aggregated)
}

fn compile_parallel(
    input: BTreeMap<Solc, Sources>,
    jobs: usize,
    settings: &Settings,
    paths: &ProjectPathsConfig,
) -> Result<AggregatedCompilerOutput> {
    todo!()
}

/// Contains a mixture of already compiled/cached artifacts and the input set of sources that still
/// need to be compiled.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProjectCompileOutput2<T: ArtifactOutput> {
    /// contains the aggregated `CompilerOutput`
    ///
    /// See [`CompilerSources::compile`]
    output: AggregatedCompilerOutput,
    /// All artifacts that were read from cache
    cached_artifacts: BTreeMap<PathBuf, T::Artifact>,
    ignored_error_codes: Vec<u64>,
}

/// The aggregated output of (multiple) compile jobs
///
/// This is effectively a solc version aware `CompilerOutput`
#[derive(Debug, Default)]
struct AggregatedCompilerOutput {
    /// all errors from all `CompilerOutput`
    pub errors: Vec<Error>,
    /// All source files
    pub sources: BTreeMap<String, SourceFile>,
    /// All compiled contracts combined with the solc version used to compile them
    pub contracts: VersionedContracts,
}

impl AggregatedCompilerOutput {
    pub fn is_empty(&self) -> bool {
        self.contracts.is_empty()
    }

    /// adds a new `CompilerOutput` to the aggregated output
    fn extend(&mut self, version: Version, output: CompilerOutput) {
        self.errors.extend(output.errors);
        self.sources.extend(output.sources);

        for (file_name, new_contracts) in output.contracts {
            let contracts = self.contracts.entry(file_name).or_default();
            for (contract_name, contract) in new_contracts {
                let versioned = contracts.entry(contract_name).or_default();
                versioned.push(VersionedContract { contract, version: version.clone() });
            }
        }
    }
}

/// A helper abstraction over the [`SolFilesCache`] used to determine what files need to compiled
/// and which `Artifacts` can be reused.
struct Cache<'a, T: ArtifactOutput> {
    /// cache file
    cache: SolFilesCache,
    /// all already existing artifacts
    cached_artifacts: BTreeMap<PathBuf, T::Artifact>,
    /// relationship between all the files
    edges: &'a GraphEdges,
    /// how to configure solc
    solc_config: &'a SolcConfig,
    /// project paths
    paths: &'a ProjectPathsConfig,
    /// all files that were filtered because they haven't changed
    filtered: Sources,
    /// the file hashes
    content_hashes: HashMap<PathBuf, String>,
}

impl<'a, T: ArtifactOutput> Cache<'a, T> {
    /// Returns only those sources that
    ///   - are new
    ///   - were changed
    ///   - their imports were changed
    ///   - their artifact is missing
    fn filter(&mut self, sources: Sources) -> Sources {
        self.fill_hashes(&sources);
        sources.into_iter().filter_map(|(file, source)| self.needs_solc(file, source)).collect()
    }

    /// Returns `Some` if the file needs to be compiled and `None` if the artifact can be reu-used
    fn needs_solc(&mut self, file: PathBuf, source: Source) -> Option<(PathBuf, Source)> {
        if !self.is_dirty(&file) &&
            self.edges.imports(&file).iter().all(|file| !self.is_dirty(file))
        {
            self.filtered.insert(file, source);
            None
        } else {
            Some((file, source))
        }
    }

    /// returns `false` if the corresponding cache entry remained unchanged otherwise `true`
    fn is_dirty(&self, file: &Path) -> bool {
        if let Some(hash) = self.content_hashes.get(file) {
            let cache_path = utils::source_name(file, &self.paths.root);
            if let Some(entry) = self.cache.entry(&cache_path) {
                if entry.content_hash.as_bytes() != hash.as_bytes() {
                    tracing::trace!(
                        "changed content hash for cached artifact \"{}\"",
                        file.display()
                    );
                    return true
                }
                if self.solc_config != &entry.solc_config {
                    tracing::trace!(
                        "changed solc config for cached artifact \"{}\"",
                        file.display()
                    );
                    return true
                }
                // checks whether an artifact this file depends on was removed
                if entry.artifacts.iter().any(|name| !self.has_artifact(file, name)) {
                    tracing::trace!(
                        "missing linked artifacts for cached artifact \"{}\"",
                        file.display()
                    );
                    return true
                }
                return false
            }
        }
        true
    }

    /// Adds the file's hashes to the set if not set yet
    fn fill_hashes(&mut self, sources: &Sources) {
        for (file, source) in sources {
            if let hash_map::Entry::Vacant(entry) = self.content_hashes.entry(file.clone()) {
                entry.insert(source.content_hash());
            }
        }
    }

    /// Returns true if the artifact for the exists
    fn has_artifact(&self, file: &Path, name: &str) -> bool {
        let artifact_path = self.paths.artifacts.join(T::output_file(file, name));
        self.cached_artifacts.contains_key(&artifact_path)
    }
}

/// Abstraction over configured caching which can be either non-existent or an already loaded cache
enum ArtifactsCache<'a, T: ArtifactOutput> {
    Ephemeral,
    Cached(Cache<'a, T>),
}

impl<'a, T: ArtifactOutput> ArtifactsCache<'a, T> {
    fn new(project: &'a Project<T>, edges: &'a GraphEdges) -> Result<Self> {
        let cache = if project.cached {
            // read the cache file if it already exists
            let cache = if project.cache_path().exists() {
                let mut cache = SolFilesCache::read(project.cache_path())?;
                // TODO this should take the project dir, since we're storing surce unit ids
                // starting at the project dir?
                cache.remove_missing_files();
                cache
            } else {
                SolFilesCache::builder()
                    .root(project.root())
                    .solc_config(project.solc_config.clone())
            };

            // read all artifacts
            let cached_artifacts = if project.paths.artifacts.exists() {
                tracing::trace!("reading artifacts from cache..");
                let artifacts = cache.read_artifacts::<T>(&project.paths.artifacts)?;
                tracing::trace!("read {} artifacts from cache", artifacts.len());
                artifacts
            } else {
                BTreeMap::default()
            };

            let cache = Cache {
                cache,
                cached_artifacts,
                edges: &edges,
                solc_config: &project.solc_config,
                paths: &project.paths,
                filtered: Default::default(),
                content_hashes: Default::default(),
            };

            ArtifactsCache::Cached(cache)
        } else {
            // nothing to cache
            ArtifactsCache::Ephemeral
        };

        Ok(cache)
    }

    /// Filters out those sources that don't need to be compiled
    fn filter(&mut self, sources: Sources) -> Sources {
        match self {
            ArtifactsCache::Ephemeral => sources,
            ArtifactsCache::Cached(cache) => cache.filter(sources),
        }
    }
}
