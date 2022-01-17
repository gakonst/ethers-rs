//! Manages compiling of a `Project`

use crate::{
    error::Result, resolver::GraphEdges, utils, ArtifactOutput, Graph, Project, ProjectPathsConfig,
    SolFilesCache, Solc, SolcConfig, Source, Sources,
};
use std::{
    collections::{hash_map, BTreeMap, HashMap},
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
    pub fn new(project: &'a Project<T>) -> Result<Self> {
        Self::with_sources(project, project.paths.read_input_files()?)
    }

    pub fn with_sources(project: &'a Project<T>, sources: Sources) -> Result<Self> {
        let graph = Graph::resolve_sources(&project.paths, sources)?;
        // TODO this should return a type that still knows the relationships edges and nodes
        let (versions, edges) = graph.into_sources_by_version(!project.auto_detect)?;

        let sources_by_version = versions.get(&project.allowed_lib_paths)?;

        let mode = if project.solc_jobs > 1 && sources_by_version.len() > 1 {
            // if there are multiple different versions and we can use multiple jobs we can compile
            // them in parallel
            CompilerSources::Para(sources_by_version, project.solc_jobs)
        } else {
            CompilerSources::Sequ(sources_by_version)
        };
        Ok(Self { edges, project, sources: mode })
    }

    /// Compiles all the sources
    pub fn compile(self) {
        let Self { edges: _, project: _, sources: _mode } = self;

        todo!()
    }
}

/// Determines how the `solc <-> sources` pairs are executed
#[derive(Debug)]
enum CompilerSources {
    /// Compile all these sequentially
    Sequ(BTreeMap<Solc, Sources>),
    /// Compile all these in parallel using a certain amount of jobs
    Para(BTreeMap<Solc, Sources>, usize),
}

impl CompilerSources {
    fn preprocess<T: ArtifactOutput>(self, _paths: &ProjectPathsConfig) -> Result<Preprocessed<T>> {
        let cached_artifacts = BTreeMap::new();

        Ok(Preprocessed { cached_artifacts, sources: self })
    }
}

/// Contains a mixture of already compiled/cached artifacts and the input set of sources that still
/// need to be compiled.
#[derive(Debug)]
struct Preprocessed<T: ArtifactOutput> {
    /// all artifacts that don't need to be compiled
    cached_artifacts: BTreeMap<PathBuf, T::Artifact>,

    sources: CompilerSources,
}

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
