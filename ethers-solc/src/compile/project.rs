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

    pub fn with_sources(project: &'a Project<T>, sources: Sources) -> Result<Self> {
        let graph = Graph::resolve_sources(&project.paths, sources)?;
        let (versions, edges) = graph.into_sources_by_version(!project.auto_detect)?;

        let sources_by_version = versions.get(&project.allowed_lib_paths)?;

        let mode = if project.solc_jobs > 1 && sources_by_version.len() > 1 {
            // if there are multiple different versions and we can use multiple jobs we can compile
            // them in parallel
            CompilerSources::Parallel(sources_by_version, project.solc_jobs)
        } else {
            CompilerSources::Sequential(sources_by_version)
        };
        Ok(Self { edges, project, sources: mode })
    }

    /// Compiles all the sources of the `Project`
    pub fn compile(self) {
        let Self { edges: _, project: _, sources: _mode } = self;

        todo!()
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
    fn preprocess<T: ArtifactOutput>(self, _paths: &ProjectPathsConfig) -> Result<Preprocessed<T>> {
        let cached_artifacts = BTreeMap::new();

        todo!()
    }

    /// Compiles all the files with `Solc`
    fn compile(
        self,
        settings: Settings,
        remappings: Vec<Remapping>,
    ) -> Result<AggregatedCompilerOutput> {
        match self {
            CompilerSources::Sequential(input) => compile_sequential(input, settings, remappings),
            CompilerSources::Parallel(input, j) => compile_parallel(input, j, settings, remappings),
        }
    }
}

fn compile_sequential(
    input: BTreeMap<Solc, Sources>,
    settings: Settings,
    remappings: Vec<Remapping>,
) -> Result<AggregatedCompilerOutput> {
    for (solc, sources) in input {
        let version = solc.version()?;

        tracing::trace!(
            "compiling {} sources with solc \"{}\"",
            sources.len(),
            solc.as_ref().display()
        );

        let mut paths = PathMap::default();
        // replace absolute path with source name to make solc happy
        // TODO use correct path
        let sources = paths.set_source_names(sources);

        let input = CompilerInput::with_sources(sources)
            .settings(settings.clone())
            .normalize_evm_version(&version)
            .with_remappings(remappings.clone());

        tracing::trace!("calling solc with {} sources", input.sources.len());
        let output = solc.compile(&input)?;
        tracing::trace!("compiled input, output has error: {}", output.has_error());
    }

    todo!()
}

fn compile_parallel(
    input: BTreeMap<Solc, Sources>,
    jobs: usize,
    settings: Settings,
    remappings: Vec<Remapping>,
) -> Result<AggregatedCompilerOutput> {
    todo!()
}

/// The aggregated output of (multiple) compile jobs
///
/// This is effectively a solc version aware `CompilerOutput`
#[derive(Debug, Default)]
struct AggregatedCompilerOutput {
    /// all errors from all `CompilerOutput`
    ///
    /// this is a set so that the same error from multiple `CompilerOutput`s only appears once
    pub errors: HashSet<Error>,
    /// All source files
    pub sources: BTreeMap<String, SourceFile>,
    /// All compiled contracts combined with the solc version used to compile them
    pub contracts: VersionedContracts,
}

impl AggregatedCompilerOutput {
    /// adds a new `CompilerOutput` to the aggregated output
    fn extend(&mut self, version: Version, output: CompilerOutput) {
        self.errors.extend(compiled.errors);
        self.sources.extend(compiled.sources);

        for (file_name, new_contracts) in output.contracts {
            let contracts = self.contracts.entry(file_name).or_default();
            for (contract_name, contract) in new_contracts {
                let versioned = contracts.entry(contract_name).or_default();
                versioned.push(VersionedContract { contract, version: version.clone() });
            }
        }
    }
}

/// Captures the `CompilerOutput` and the `Solc` version that produced it
#[derive(Debug)]
struct VersionCompilerOutput {
    output: CompilerOutput,
    solc: Solc,
    version: Version,
}

/// Contains a mixture of already compiled/cached artifacts and the input set of sources that still
/// need to be compiled.
#[derive(Debug)]
struct Preprocessed<T: ArtifactOutput> {
    /// all artifacts that don't need to be compiled
    cached_artifacts: BTreeMap<PathBuf, T::Artifact>,

    cache: SolFilesCache,

    sources: CompilerSources,
}

impl<T: ArtifactOutput> Preprocessed<T> {
    /// Drives the compilation process to completion
    pub fn finish(self) {}
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
    /// Filters out those sources that don't need to be compiled
    fn filter(&mut self, sources: Sources) -> Sources {
        match self {
            ArtifactsCache::Ephemeral => sources,
            ArtifactsCache::Cached(cache) => cache.filter(sources),
        }
    }
}
