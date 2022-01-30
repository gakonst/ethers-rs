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
    artifact_output::Artifacts,
    artifacts::{Settings, VersionedSources},
    cache::CacheEntry,
    error::Result,
    output::AggregatedCompilerOutput,
    resolver::GraphEdges,
    utils, ArtifactOutput, CompilerInput, Graph, Project, ProjectCompileOutput, ProjectPathsConfig,
    SolFilesCache, Solc, SolcConfig, Source, SourceUnitNameMap, Sources,
};
use rayon::prelude::*;
use semver::Version;
use std::{
    collections::{btree_map::BTreeMap, hash_map, hash_map::Entry, HashMap, HashSet},
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
            // if there are multiple different versions and we can use multiple jobs we can compile
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
        let Self { edges, project, mut sources } = self;
        // the map that keeps track of the mapping of resolved solidity file paths -> source unit
        // names
        let mut source_unit_map = SourceUnitNameMap::default();

        let mut cache = ArtifactsCache::new(project, &edges)?;
        // retain and compile only dirty sources
        sources = sources
            .filtered(&mut cache)
            .set_source_unit_names(&project.paths, &mut source_unit_map);

        let output = sources.compile(&project.solc_config.settings, &project.paths)?;

        // TODO reapply the mappings to the contracts

        // write all artifacts
        let written_artifacts = if !project.no_artifacts {
            T::on_output(&output.contracts, &project.paths)?
        } else {
            Default::default()
        };

        // if caching was enabled, this will write to disk and get the artifacts that weren't
        // compiled but reused
        let cached_artifacts = cache.finish(&written_artifacts)?;

        Ok(ProjectCompileOutput {
            compiler_output: output,
            written_artifacts,
            cached_artifacts,
            ignored_error_codes: project.ignored_error_codes.clone(),
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
        fn filterd_sources<T: ArtifactOutput>(
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
                CompilerSources::Sequential(filterd_sources(s, cache))
            }
            CompilerSources::Parallel(s, j) => {
                CompilerSources::Parallel(filterd_sources(s, cache), j)
            }
        }
    }

    /// Sets the correct source unit names for all sources
    fn set_source_unit_names(
        self,
        paths: &ProjectPathsConfig,
        names: &mut SourceUnitNameMap,
    ) -> Self {
        fn set(
            _sources: VersionedSources,
            _paths: &ProjectPathsConfig,
            _cache: &mut SourceUnitNameMap,
        ) -> VersionedSources {
            todo!()
        }

        match self {
            CompilerSources::Sequential(s) => CompilerSources::Sequential(set(s, paths, names)),
            CompilerSources::Parallel(s, j) => CompilerSources::Parallel(set(s, paths, names), j),
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
    input: VersionedSources,
    settings: &Settings,
    paths: &ProjectPathsConfig,
) -> Result<AggregatedCompilerOutput> {
    let mut aggregated = AggregatedCompilerOutput::default();
    for (solc, (version, sources)) in input {
        if sources.is_empty() {
            // nothing to compile
            continue
        }
        tracing::trace!(
            "compiling {} sources with solc \"{}\"",
            sources.len(),
            solc.as_ref().display()
        );

        let input = CompilerInput::with_sources(sources)
            .settings(settings.clone())
            .normalize_evm_version(&version)
            .with_remappings(paths.remappings.clone());

        tracing::trace!("calling solc `{}` with {} sources", version, input.sources.len());
        let output = solc.compile(&input)?;
        tracing::trace!("compiled input, output has error: {}", output.has_error());

        aggregated.extend(version, output);
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
    tracing::trace!("compile sources in parallel using {} solc jobs", num_jobs);

    let mut jobs = Vec::with_capacity(input.len());
    for (solc, (version, sources)) in input {
        if sources.is_empty() {
            // nothing to compile
            continue
        }

        let job = CompilerInput::with_sources(sources)
            .settings(settings.clone())
            .normalize_evm_version(&version)
            .with_remappings(paths.remappings.clone());

        jobs.push((solc, version, job))
    }

    // start a rayon threadpool that will execute all `Solc::compile()` processes
    let pool = rayon::ThreadPoolBuilder::new().num_threads(num_jobs).build().unwrap();
    let outputs = pool.install(move || {
        jobs.into_par_iter()
            .map(|(solc, version, input)| {
                tracing::trace!("calling solc `{}` with {} sources", version, input.sources.len());
                solc.compile(&input).map(|output| (version, output))
            })
            .collect::<Result<Vec<_>>>()
    })?;

    let mut aggregated = AggregatedCompilerOutput::default();
    aggregated.extend_all(outputs);

    Ok(aggregated)
}

/// A helper abstraction over the [`SolFilesCache`] used to determine what files need to compiled
/// and which `Artifacts` can be reused.
struct Cache<'a, T: ArtifactOutput> {
    /// preexisting cache file
    cache: SolFilesCache,
    /// all already existing artifacts
    cached_artifacts: Artifacts<T::Artifact>,
    /// relationship between all the files
    edges: &'a GraphEdges,
    /// how to configure solc
    solc_config: &'a SolcConfig,
    /// project paths
    paths: &'a ProjectPathsConfig,
    /// all files that were filtered because they haven't changed
    filtered: HashMap<PathBuf, (Source, HashSet<Version>)>,
    /// the corresponding cache entries for all sources that were deemed to be dirty
    ///
    /// `CacheEntry` are grouped by their solidity file.
    /// During preprocessing the `artifacts` field of a new `CacheEntry` is left blank, because in
    /// order to determine the artifacts of the solidity file, the file needs to be compiled first.
    /// Only after the `CompilerOutput` is received and all compiled contracts are handled, see
    /// [`crate::ArtifactOutput::on_output()`] all artifacts, their disk paths, are determined and
    /// can be populated before the updated [`crate::SolFilesCache`] is finally written to disk,
    /// see [`Cache::finish()`]
    dirty_entries: HashMap<PathBuf, (CacheEntry, HashSet<Version>)>,
    /// the file hashes
    content_hashes: HashMap<PathBuf, String>,
}

impl<'a, T: ArtifactOutput> Cache<'a, T> {
    /// Creates a new cache entry for the file
    fn create_cache_entry(&self, file: &Path, source: &Source) -> Result<CacheEntry> {
        let imports = self
            .edges
            .imports(file)
            .into_iter()
            .map(|import| utils::source_name(import, &self.paths.root).to_path_buf())
            .collect();

        let entry = CacheEntry {
            last_modification_date: CacheEntry::read_last_modification_date(&file).unwrap(),
            content_hash: source.content_hash(),
            source_name: utils::source_name(file, &self.paths.root).into(),
            solc_config: self.solc_config.clone(),
            imports,
            version_requirement: self.edges.version_requirement(file).map(|v| v.to_string()),
            // artifacts remain empty until we received the compiler output
            artifacts: Default::default(),
        };

        Ok(entry)
    }

    /// inserts a new cache entry for the given file
    ///
    /// If there is already an entry available for the file the given version is added to the set
    fn insert_new_cache_entry(
        &mut self,
        file: &Path,
        source: &Source,
        version: Version,
    ) -> Result<()> {
        if let Some((_, versions)) = self.dirty_entries.get_mut(file) {
            versions.insert(version);
        } else {
            let entry = self.create_cache_entry(file, source)?;
            self.dirty_entries.insert(file.to_path_buf(), (entry, HashSet::from([version])));
        }
        Ok(())
    }

    /// inserts the filtered source with the fiven version
    fn insert_filtered_source(&mut self, file: PathBuf, source: Source, version: Version) {
        match self.filtered.entry(file) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().1.insert(version);
            }
            Entry::Vacant(entry) => {
                entry.insert((source, HashSet::from([version])));
            }
        }
    }

    /// Returns only those sources that
    ///   - are new
    ///   - were changed
    ///   - their imports were changed
    ///   - their artifact is missing
    fn filter(&mut self, sources: Sources, version: &Version) -> Sources {
        self.fill_hashes(&sources);
        sources
            .into_iter()
            .filter_map(|(file, source)| self.requires_solc(file, source, version))
            .collect()
    }

    /// Returns `Some` if the file _needs_ to be compiled and `None` if the artifact can be reu-used
    fn requires_solc(
        &mut self,
        file: PathBuf,
        source: Source,
        version: &Version,
    ) -> Option<(PathBuf, Source)> {
        if !self.is_dirty(&file, version) &&
            self.edges.imports(&file).iter().all(|file| !self.is_dirty(file, version))
        {
            self.insert_filtered_source(file, source, version.clone());
            None
        } else {
            self.insert_new_cache_entry(&file, &source, version.clone()).unwrap();
            Some((file, source))
        }
    }

    /// returns `false` if the corresponding cache entry remained unchanged otherwise `true`
    fn is_dirty(&self, file: &Path, version: &Version) -> bool {
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

                if !entry.contains_version(version) {
                    tracing::trace!("missing linked artifacts for version \"{}\"", version);
                    return true
                }

                if entry.artifacts_for_version(version).any(|artifact_path| {
                    // artifact does not exist
                    !self.cached_artifacts.has_artifact(artifact_path)
                }) {
                    return true
                }
                // all things match, can be reused
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
}

/// Abstraction over configured caching which can be either non-existent or an already loaded cache
#[allow(clippy::large_enum_variant)]
enum ArtifactsCache<'a, T: ArtifactOutput> {
    /// Cache nothing on disk
    Ephemeral,
    Cached(Cache<'a, T>),
}

impl<'a, T: ArtifactOutput> ArtifactsCache<'a, T> {
    fn new(project: &'a Project<T>, edges: &'a GraphEdges) -> Result<Self> {
        let cache = if project.cached {
            // read the cache file if it already exists
            let cache = if project.cache_path().exists() {
                let mut cache = SolFilesCache::read(project.cache_path())?;
                cache.join_all(project.artifacts_path()).remove_missing_files();
                cache
            } else {
                SolFilesCache::default()
            };

            // read all artifacts
            let cached_artifacts = if project.paths.artifacts.exists() {
                tracing::trace!("reading artifacts from cache..");
                // if we failed to read the whole set of artifacts we use an empty set
                let artifacts = cache.read_artifacts::<T::Artifact>().unwrap_or_default();
                tracing::trace!("read {} artifacts from cache", artifacts.artifact_files().count());
                artifacts
            } else {
                Default::default()
            };

            let cache = Cache {
                cache,
                cached_artifacts,
                edges,
                solc_config: &project.solc_config,
                paths: &project.paths,
                filtered: Default::default(),
                dirty_entries: Default::default(),
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
    fn filter(&mut self, sources: Sources, version: &Version) -> Sources {
        match self {
            ArtifactsCache::Ephemeral => sources,
            ArtifactsCache::Cached(cache) => cache.filter(sources, version),
        }
    }

    /// Consumes the `Cache`, rebuilds the [`SolFileCache`] by merging all artifacts that were
    /// filtered out in the previous step (`Cache::filtered`) and the artifacts that were just
    /// written to disk `written_artifacts`.
    ///
    /// Returns all the _cached_ artifacts.
    fn finish(self, written_artifacts: &Artifacts<T::Artifact>) -> Result<Artifacts<T::Artifact>> {
        match self {
            ArtifactsCache::Ephemeral => Ok(Default::default()),
            ArtifactsCache::Cached(cache) => {
                let Cache {
                    mut cache, cached_artifacts, mut dirty_entries, filtered, paths, ..
                } = cache;

                // keep only those files that were previously filtered (not dirty, reused)
                cache.retain(filtered.iter().map(|(p, (_, v))| (p.as_path(), v)));

                // add the artifacts to the cache entries, this way we can keep a mapping from
                // solidity file to its artifacts
                // this step is necessary because the concrete artifacts are only known after solc
                // was invoked and received as output, before that we merely know the the file and
                // the versions so we add the artifacts on a file by file basis
                for (file, artifacts) in written_artifacts.as_ref() {
                    let file_path = Path::new(&file);
                    if let Some((entry, versions)) = dirty_entries.get_mut(file_path) {
                        entry.insert_artifacts(artifacts.into_iter().map(|(name, artifacts)| {
                            let artifacts = artifacts
                                .into_iter()
                                .filter(|artifact| versions.contains(&artifact.version))
                                .collect::<Vec<_>>();
                            (name, artifacts)
                        }));
                    }
                }

                // add the new cache entries to the cache file
                cache.extend(dirty_entries.into_iter().map(|(file, (entry, _))| (file, entry)));

                // write to disk
                cache.write(&paths.cache)?;

                Ok(cached_artifacts)
            }
        }
    }
}
