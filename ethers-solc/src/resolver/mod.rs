//! Resolution of the entire dependency graph for a project.
//!
//! This module implements the core logic in taking all contracts of a project and creating a
//! resolved graph with applied remappings for all source contracts.
//!
//! Some constraints we're working with when resolving contracts
//!
//!   1. Each file can contain several source units and can have any number of imports/dependencies
//! (using the term interchangeably). Each dependency can declare a version range that it is
//! compatible with, solidity version pragma.
//!   2. A dependency can be imported from any directory,
//! see `Remappings`
//!
//! Finding all dependencies is fairly simple, we're simply doing a DFS, starting the source
//! contracts
//!
//! ## Solc version auto-detection
//!
//! Solving a constraint graph is an NP-hard problem. The algorithm for finding the "best" solution
//! makes several assumptions and tries to find a version of "Solc" that is compatible with all
//! source files.
//!
//! The algorithm employed here is fairly simple, we simply do a DFS over all the source files and
//! find the set of Solc versions that the file and all its imports are compatible with, and then we
//! try to find a single Solc version that is compatible with all the files. This is effectively the
//! intersection of all version sets.
//!
//! We always try to activate the highest (installed) solc version first. Uninstalled solc is only
//! used if this version is the only compatible version for a single file or in the intersection of
//! all version sets.
//!
//! This leads to finding the optimal version, if there is one. If there is no single Solc version
//! that is compatible with all sources and their imports, then suddenly this becomes a very
//! difficult problem, because what would be the "best" solution. In this case, just choose the
//! latest (installed) Solc version and try to minimize the number of Solc versions used.
//!
//! ## Performance
//!
//! Note that this is a relatively performance-critical portion of the ethers-solc preprocessing.
//! The data that needs to be processed is proportional to the size of the dependency
//! graph, which can, depending on the project, often be quite large.
//!
//! Note that, unlike the solidity compiler, we work with the filesystem, where we have to resolve
//! remappings and follow relative paths. We're also limiting the nodes in the graph to solidity
//! files, since we're only interested in their
//! [version pragma](https://docs.soliditylang.org/en/develop/layout-of-source-files.html#version-pragma),
//! which is defined on a per source file basis.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt, io,
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use regex::Match;
use semver::VersionReq;
use solang_parser::pt::{Import, Loc, SourceUnitPart};

use crate::{error::Result, utils, ProjectPathsConfig, Solc, SolcError, Source, Sources};

mod tree;
pub use tree::{print, Charset, TreeOptions};

/// The underlying edges of the graph which only contains the raw relationship data.
///
/// This is kept separate from the `Graph` as the `Node`s get consumed when the `Solc` to `Sources`
/// set is determined.
#[derive(Debug, Clone)]
pub struct GraphEdges {
    /// The indices of `edges` correspond to the `nodes`. That is, `edges[0]`
    /// is the set of outgoing edges for `nodes[0]`.
    edges: Vec<Vec<usize>>,
    /// index maps for a solidity file to an index, for fast lookup.
    indices: HashMap<PathBuf, usize>,
    /// reverse of `indices` for reverse lookup
    rev_indices: HashMap<usize, PathBuf>,
    /// the identified version requirement of a file
    versions: HashMap<usize, Option<VersionReq>>,
    /// with how many input files we started with, corresponds to `let input_files =
    /// nodes[..num_input_files]`.
    ///
    /// Combined with the `indices` this way we can determine if a file was original added to the
    /// graph as input or was added as resolved import, see [`Self::is_input_file()`]
    num_input_files: usize,
}

impl GraphEdges {
    /// Returns a list of nodes the given node index points to for the given kind.
    pub fn imported_nodes(&self, from: usize) -> &[usize] {
        &self.edges[from]
    }

    /// Returns all files imported by the given file
    pub fn imports(&self, file: impl AsRef<Path>) -> HashSet<&PathBuf> {
        if let Some(start) = self.indices.get(file.as_ref()).copied() {
            NodesIter::new(start, self).skip(1).map(move |idx| &self.rev_indices[&idx]).collect()
        } else {
            HashSet::new()
        }
    }

    /// Returns true if the `file` was originally included when the graph was first created and not
    /// added when all `imports` were resolved
    pub fn is_input_file(&self, file: impl AsRef<Path>) -> bool {
        if let Some(idx) = self.indices.get(file.as_ref()).copied() {
            idx < self.num_input_files
        } else {
            false
        }
    }

    /// Returns the `VersionReq` for the given file
    pub fn version_requirement(&self, file: impl AsRef<Path>) -> Option<&VersionReq> {
        self.indices
            .get(file.as_ref())
            .and_then(|idx| self.versions.get(idx))
            .and_then(|v| v.as_ref())
    }
}

/// Represents a fully-resolved solidity dependency graph. Each node in the graph
/// is a file and edges represent dependencies between them.
/// See also https://docs.soliditylang.org/en/latest/layout-of-source-files.html?highlight=import#importing-other-source-files
#[derive(Debug)]
pub struct Graph {
    nodes: Vec<Node>,
    /// relationship of the nodes
    edges: GraphEdges,
    /// the root of the project this graph represents
    #[allow(unused)]
    root: PathBuf,
}

impl Graph {
    /// Print the graph to `StdOut`
    pub fn print(&self) {
        self.print_with_options(Default::default())
    }

    /// Print the graph to `StdOut` using the provided `TreeOptions`
    pub fn print_with_options(&self, opts: TreeOptions) {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        tree::print(self, &opts, &mut out).expect("failed to write to stdout.")
    }

    /// Returns a list of nodes the given node index points to for the given kind.
    pub fn imported_nodes(&self, from: usize) -> &[usize] {
        self.edges.imported_nodes(from)
    }

    /// Returns `true` if the given node has any outgoing edges.
    pub(crate) fn has_outgoing_edges(&self, index: usize) -> bool {
        !self.edges.edges[index].is_empty()
    }

    /// Returns all the resolved files and their index in the graph
    pub fn files(&self) -> &HashMap<PathBuf, usize> {
        &self.edges.indices
    }

    /// Gets a node by index.
    ///
    /// # Panics
    ///
    /// if the `index` node id is not included in the graph
    pub fn node(&self, index: usize) -> &Node {
        &self.nodes[index]
    }

    pub(crate) fn display_node(&self, index: usize) -> DisplayNode {
        DisplayNode { node: self.node(index), root: &self.root }
    }
    /// Returns an iterator that yields all nodes of the dependency tree that the given node id
    /// spans, starting with the node itself.
    ///
    /// # Panics
    ///
    /// if the `start` node id is not included in the graph
    pub fn node_ids(&self, start: usize) -> impl Iterator<Item = usize> + '_ {
        NodesIter::new(start, &self.edges)
    }

    /// Same as `Self::node_ids` but returns the actual `Node`
    pub fn nodes(&self, start: usize) -> impl Iterator<Item = &Node> + '_ {
        self.node_ids(start).map(move |idx| self.node(idx))
    }

    /// Consumes the `Graph`, effectively splitting the `nodes` and the `GraphEdges` off and
    /// returning the `nodes` converted to `Sources`
    pub fn into_sources(self) -> (Sources, GraphEdges) {
        let Graph { nodes, edges, .. } = self;
        (nodes.into_iter().map(|node| (node.path, node.source)).collect(), edges)
    }

    /// Returns an iterator that yields only those nodes that represent input files.
    /// See `Self::resolve_sources`
    /// This won't yield any resolved library nodes
    pub fn input_nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().take(self.edges.num_input_files)
    }

    pub fn imports(&self, path: impl AsRef<Path>) -> HashSet<&PathBuf> {
        self.edges.imports(path)
    }

    /// Resolves a number of sources within the given config
    pub fn resolve_sources(paths: &ProjectPathsConfig, sources: Sources) -> Result<Graph> {
        /// checks if the given target path was already resolved, if so it adds its id to the list
        /// of resolved imports. If it hasn't been resolved yet, it queues in the file for
        /// processing
        fn add_node(
            unresolved: &mut VecDeque<(PathBuf, Node)>,
            index: &mut HashMap<PathBuf, usize>,
            resolved_imports: &mut Vec<usize>,
            target: PathBuf,
        ) -> Result<()> {
            if let Some(idx) = index.get(&target).copied() {
                resolved_imports.push(idx);
            } else {
                // imported file is not part of the input files
                let node = read_node(&target)?;
                unresolved.push_back((target.clone(), node));
                let idx = index.len();
                index.insert(target, idx);
                resolved_imports.push(idx);
            }
            Ok(())
        }

        println!("paths {:?}", paths);

        // we start off by reading all input files, which includes all solidity files from the
        // source and test folder
        let mut unresolved: VecDeque<(PathBuf, Node)> = sources
            .into_par_iter()
            .map(|(path, source)| {
                let data = parse_data(source.as_ref(), &path);
                (path.clone(), Node { path, source, data })
            })
            .collect();

        // identifiers of all resolved files
        let mut index: HashMap<_, _> =
            unresolved.iter().enumerate().map(|(idx, (p, _))| (p.clone(), idx)).collect();

        let num_input_files = unresolved.len();

        // contains the files and their dependencies
        let mut nodes = Vec::with_capacity(unresolved.len());
        let mut edges = Vec::with_capacity(unresolved.len());
        // now we need to resolve all imports for the source file and those imported from other
        // locations
        while let Some((path, node)) = unresolved.pop_front() {
            let mut resolved_imports = Vec::with_capacity(node.data.imports.len());
            // parent directory of the current file
            let cwd = match path.parent() {
                Some(inner) => inner,
                None => continue,
            };

            for import in node.data.imports.iter() {
                match paths.resolve_import(cwd, import.data()) {
                    Ok(import) => {
                        add_node(&mut unresolved, &mut index, &mut resolved_imports, import)?;
                    }
                    Err(err) => {
                        crate::report::unresolved_import(import.data());
                        tracing::trace!("failed to resolve import component \"{:?}\"", err)
                    }
                };
            }
            nodes.push(node);
            edges.push(resolved_imports);
        }
        let edges = GraphEdges {
            edges,
            rev_indices: index.iter().map(|(k, v)| (*v, k.clone())).collect(),
            indices: index,
            num_input_files,
            versions: nodes
                .iter()
                .enumerate()
                .map(|(idx, node)| (idx, node.data.version_req.clone()))
                .collect(),
        };
        Ok(Graph { nodes, edges, root: paths.root.clone() })
    }

    /// Resolves the dependencies of a project's source contracts
    pub fn resolve(paths: &ProjectPathsConfig) -> Result<Graph> {
        Self::resolve_sources(paths, paths.read_input_files()?)
    }
}

#[cfg(all(feature = "svm", feature = "async"))]
impl Graph {
    /// Consumes the nodes of the graph and returns all input files together with their appropriate
    /// version and the edges of the graph
    ///
    /// First we determine the compatible version for each input file (from sources and test folder,
    /// see `Self::resolve`) and then we add all resolved library imports.
    pub fn into_sources_by_version(self, offline: bool) -> Result<(VersionedSources, GraphEdges)> {
        /// insert the imports of the given node into the sources map
        /// There can be following graph:
        /// `A(<=0.8.10) imports C(>0.4.0)` and `B(0.8.11) imports C(>0.4.0)`
        /// where `C` is a library import, in which case we assign `C` only to the first input file.
        /// However, it's not required to include them in the solc `CompilerInput` as they would get
        /// picked up by solc otherwise, but we add them, so we can create a corresponding
        /// cache entry for them as well. This can be optimized however
        fn insert_imports(
            idx: usize,
            all_nodes: &mut HashMap<usize, Node>,
            sources: &mut Sources,
            edges: &[Vec<usize>],
            num_input_files: usize,
        ) {
            for dep in edges[idx].iter().copied() {
                // we only process nodes that were added as part of the resolve step because input
                // nodes are handled separately
                if dep >= num_input_files {
                    // library import
                    if let Some(node) = all_nodes.remove(&dep) {
                        sources.insert(node.path, node.source);
                        insert_imports(dep, all_nodes, sources, edges, num_input_files);
                    }
                }
            }
        }

        let versioned_nodes = self.get_input_node_versions(offline)?;
        let Self { nodes, edges, .. } = self;
        let mut versioned_sources = HashMap::with_capacity(versioned_nodes.len());
        let mut all_nodes = nodes.into_iter().enumerate().collect::<HashMap<_, _>>();

        // determine the `Sources` set for each solc version
        for (version, input_node_indices) in versioned_nodes {
            let mut sources = Sources::new();
            // we only process input nodes (from sources, tests for example)
            for idx in input_node_indices {
                // insert the input node in the sources set and remove it from the available set
                let node = all_nodes.remove(&idx).expect("node is preset. qed");
                sources.insert(node.path, node.source);
                insert_imports(
                    idx,
                    &mut all_nodes,
                    &mut sources,
                    &edges.edges,
                    edges.num_input_files,
                );
            }
            versioned_sources.insert(version, sources);
        }
        Ok((VersionedSources { inner: versioned_sources, offline }, edges))
    }

    /// Writes the list of imported files into the given formatter:
    /// `A (version) imports B (version)`
    fn format_imports_list<W: std::fmt::Write>(
        &self,
        idx: usize,
        f: &mut W,
    ) -> std::result::Result<(), std::fmt::Error> {
        let node = self.node(idx);
        write!(f, "{} ", utils::source_name(&node.path, &self.root).display(),)?;
        node.data.fmt_version(f)?;
        write!(f, " imports:",)?;
        for dep in self.node_ids(idx).skip(1) {
            writeln!(f)?;
            let dep = self.node(dep);
            write!(f, "    {} ", utils::source_name(&dep.path, &self.root).display())?;
            dep.data.fmt_version(f)?;
        }

        Ok(())
    }

    /// Filters incompatible versions from the `candidates`.
    fn retain_compatible_versions(&self, idx: usize, candidates: &mut Vec<&crate::SolcVersion>) {
        let nodes: HashSet<_> = self.node_ids(idx).collect();
        for node in nodes {
            let node = self.node(node);
            if let Some(ref req) = node.data.version_req {
                candidates.retain(|v| req.matches(v.as_ref()));
            }
            if candidates.is_empty() {
                // nothing to filter anymore
                return
            }
        }
    }

    /// Ensures that all files are compatible with all of their imports.
    pub fn ensure_compatible_imports(&self, offline: bool) -> Result<()> {
        self.get_input_node_versions(offline)?;
        Ok(())
    }

    /// Returns a map of versions together with the input nodes that are compatible with that
    /// version.
    ///
    /// This will essentially do a DFS on all input sources and their transitive imports and
    /// checking that all can compiled with the version stated in the input file.
    ///
    /// Returns an error message with __all__ input files that don't have compatible imports.
    ///
    /// This also attempts to prefer local installations over remote available.
    /// If `offline` is set to `true` then only already installed.
    fn get_input_node_versions(
        &self,
        offline: bool,
    ) -> Result<HashMap<crate::SolcVersion, Vec<usize>>> {
        tracing::trace!("resolving input node versions");
        // this is likely called by an application and will be eventually printed so we don't exit
        // on first error, instead gather all the errors and return a bundled error message instead
        let mut errors = Vec::new();
        // we also  don't want duplicate error diagnostic
        let mut erroneous_nodes =
            std::collections::HashSet::with_capacity(self.edges.num_input_files);

        // the sorted list of all versions
        let all_versions = if offline { Solc::installed_versions() } else { Solc::all_versions() };

        // stores all versions and their nodes that can be compiled
        let mut versioned_nodes = HashMap::new();

        // stores all files and the versions they're compatible with
        let mut all_candidates = Vec::with_capacity(self.edges.num_input_files);
        // walking through the node's dep tree and filtering the versions along the way
        for idx in 0..self.edges.num_input_files {
            let mut candidates = all_versions.iter().collect::<Vec<_>>();
            // remove all incompatible versions from the candidates list by checking the node and
            // all its imports
            self.retain_compatible_versions(idx, &mut candidates);

            if candidates.is_empty() && !erroneous_nodes.contains(&idx) {
                let mut msg = String::new();
                self.format_imports_list(idx, &mut msg).unwrap();
                errors.push(format!(
                    "Discovered incompatible solidity versions in following\n: {}",
                    msg
                ));
                erroneous_nodes.insert(idx);
            } else {
                // found viable candidates, pick the most recent version that's already installed
                let candidate =
                    if let Some(pos) = candidates.iter().rposition(|v| v.is_installed()) {
                        candidates[pos]
                    } else {
                        candidates.last().expect("not empty; qed.")
                    }
                    .clone();

                // also store all possible candidates to optimize the set
                all_candidates.push((idx, candidates.into_iter().collect::<HashSet<_>>()));

                versioned_nodes.entry(candidate).or_insert_with(|| Vec::with_capacity(1)).push(idx);
            }
        }

        // detected multiple versions but there might still exist a single version that satisfies
        // all sources
        if versioned_nodes.len() > 1 {
            versioned_nodes = Self::resolve_multiple_versions(all_candidates);
        }

        if versioned_nodes.len() == 1 {
            tracing::trace!(
                "found exact solc version for all sources  \"{}\"",
                versioned_nodes.keys().next().unwrap()
            );
        }

        if errors.is_empty() {
            tracing::trace!(
                "resolved {} versions {:?}",
                versioned_nodes.len(),
                versioned_nodes.keys()
            );
            Ok(versioned_nodes)
        } else {
            tracing::error!("failed to resolve versions");
            Err(SolcError::msg(errors.join("\n")))
        }
    }

    /// Tries to find the "best" set of versions to nodes, See [Solc version
    /// auto-detection](#solc-version-auto-detection)
    ///
    /// This is a bit inefficient but is fine, the max. number of versions is ~80 and there's
    /// a high chance that the number of source files is <50, even for larger projects.
    fn resolve_multiple_versions(
        all_candidates: Vec<(usize, HashSet<&crate::SolcVersion>)>,
    ) -> HashMap<crate::SolcVersion, Vec<usize>> {
        // returns the intersection as sorted set of nodes
        fn intersection<'a>(
            mut sets: Vec<&HashSet<&'a crate::SolcVersion>>,
        ) -> Vec<&'a crate::SolcVersion> {
            if sets.is_empty() {
                return Vec::new()
            }

            let mut result = sets.pop().cloned().expect("not empty; qed.").clone();
            if !sets.is_empty() {
                result.retain(|item| sets.iter().all(|set| set.contains(item)));
            }

            let mut v = result.into_iter().collect::<Vec<_>>();
            v.sort_unstable();
            v
        }

        /// returns the highest version that is installed
        /// if the candidates set only contains uninstalled versions then this returns the highest
        /// uninstalled version
        fn remove_candidate(candidates: &mut Vec<&crate::SolcVersion>) -> crate::SolcVersion {
            debug_assert!(!candidates.is_empty());

            if let Some(pos) = candidates.iter().rposition(|v| v.is_installed()) {
                candidates.remove(pos)
            } else {
                candidates.pop().expect("not empty; qed.")
            }
            .clone()
        }

        let all_sets = all_candidates.iter().map(|(_, versions)| versions).collect();

        // find all versions that satisfy all nodes
        let mut intersection = intersection(all_sets);
        if !intersection.is_empty() {
            let exact_version = remove_candidate(&mut intersection);
            let all_nodes = all_candidates.into_iter().map(|(node, _)| node).collect();
            tracing::trace!(
                "resolved solc version compatible with all sources  \"{}\"",
                exact_version
            );
            return HashMap::from([(exact_version, all_nodes)])
        }

        // no version satisfies all nodes
        let mut versioned_nodes: HashMap<crate::SolcVersion, Vec<usize>> = HashMap::new();

        // try to minimize the set of versions, this is guaranteed to lead to `versioned_nodes.len()
        // > 1` as no solc version exists that can satisfy all sources
        for (node, versions) in all_candidates {
            // need to sort them again
            let mut versions = versions.into_iter().collect::<Vec<_>>();
            versions.sort_unstable();

            let candidate =
                if let Some(idx) = versions.iter().rposition(|v| versioned_nodes.contains_key(v)) {
                    // use a version that's already in the set
                    versions.remove(idx).clone()
                } else {
                    // use the highest version otherwise
                    remove_candidate(&mut versions)
                };

            versioned_nodes.entry(candidate).or_insert_with(|| Vec::with_capacity(1)).push(node);
        }

        tracing::trace!(
            "no solc version can satisfy all source files, resolved multiple versions  \"{:?}\"",
            versioned_nodes.keys()
        );

        versioned_nodes
    }
}

/// An iterator over a node and its dependencies
#[derive(Debug)]
pub struct NodesIter<'a> {
    /// stack of nodes
    stack: VecDeque<usize>,
    visited: HashSet<usize>,
    graph: &'a GraphEdges,
}

impl<'a> NodesIter<'a> {
    fn new(start: usize, graph: &'a GraphEdges) -> Self {
        Self { stack: VecDeque::from([start]), visited: HashSet::new(), graph }
    }
}

impl<'a> Iterator for NodesIter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop_front()?;

        if self.visited.insert(node) {
            // push the node's direct dependencies to the stack if we haven't visited it already
            self.stack.extend(self.graph.imported_nodes(node).iter().copied());
        }
        Some(node)
    }
}

/// Container type for solc versions and their compatible sources
#[cfg(all(feature = "svm", feature = "async"))]
#[derive(Debug)]
pub struct VersionedSources {
    inner: HashMap<crate::SolcVersion, Sources>,
    offline: bool,
}

#[cfg(all(feature = "svm", feature = "async"))]
impl VersionedSources {
    /// Resolves or installs the corresponding `Solc` installation.
    pub fn get(
        self,
        allowed_lib_paths: &crate::AllowedLibPaths,
    ) -> Result<std::collections::BTreeMap<Solc, (semver::Version, Sources)>> {
        // we take the installer lock here to ensure installation checking is done in sync
        #[cfg(any(test, feature = "tests"))]
        let _lock = crate::compile::take_solc_installer_lock();

        let mut sources_by_version = std::collections::BTreeMap::new();
        for (version, sources) in self.inner {
            if !version.is_installed() {
                if self.offline {
                    return Err(SolcError::msg(format!(
                        "missing solc \"{}\" installation in offline mode",
                        version
                    )))
                } else {
                    Solc::blocking_install(version.as_ref())?;
                }
            }
            let solc = Solc::find_svm_installed_version(version.to_string())?.ok_or_else(|| {
                SolcError::msg(format!("solc \"{}\" should have been installed", version))
            })?;

            tracing::trace!("verifying solc checksum for {}", solc.solc.display());
            if solc.verify_checksum().is_err() {
                tracing::trace!("corrupted solc version, redownloading  \"{}\"", version);
                Solc::blocking_install(version.as_ref())?;
                tracing::trace!("reinstalled solc: \"{}\"", version);
            }
            let solc = solc.arg("--allow-paths").arg(allowed_lib_paths.to_string());
            let version = solc.version()?;
            sources_by_version.insert(solc, (version, sources));
        }
        Ok(sources_by_version)
    }
}

#[derive(Debug)]
pub struct Node {
    path: PathBuf,
    source: Source,
    data: SolData,
}

impl Node {
    pub fn content(&self) -> &str {
        &self.source.content
    }

    pub fn imports(&self) -> &Vec<SolDataUnit<PathBuf>> {
        &self.data.imports
    }

    pub fn version(&self) -> &Option<SolDataUnit<String>> {
        &self.data.version
    }

    pub fn license(&self) -> &Option<SolDataUnit<String>> {
        &self.data.license
    }
}

/// Helper type for formatting a node
pub(crate) struct DisplayNode<'a> {
    node: &'a Node,
    root: &'a PathBuf,
}

impl<'a> fmt::Display for DisplayNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = utils::source_name(&self.node.path, self.root);
        write!(f, "{}", path.display())?;
        if let Some(ref v) = self.node.data.version {
            write!(f, " {}", v.data())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
#[allow(unused)]
struct SolData {
    license: Option<SolDataUnit<String>>,
    version: Option<SolDataUnit<String>>,
    imports: Vec<SolDataUnit<PathBuf>>,
    version_req: Option<VersionReq>,
}

impl SolData {
    #[allow(unused)]
    fn fmt_version<W: std::fmt::Write>(
        &self,
        f: &mut W,
    ) -> std::result::Result<(), std::fmt::Error> {
        if let Some(ref version) = self.version {
            write!(f, "({})", version.data)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SolDataUnit<T> {
    loc: Location,
    data: T,
}
#[derive(Debug, Clone)]
pub struct Location {
    pub start: usize,
    pub end: usize,
}

/// Solidity Data Unit decorated with its location within the file
impl<T> SolDataUnit<T> {
    pub fn new(data: T, loc: Location) -> Self {
        Self { data, loc }
    }

    /// Returns the underlying data for the unit
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns the location of the given data unit
    pub fn loc(&self) -> (usize, usize) {
        (self.loc.start, self.loc.end)
    }

    /// Returns the location of the given data unit adjusted by an offset.
    /// Used to determine new position of the unit within the file after
    /// content manipulation.
    pub fn loc_by_offset(&self, offset: isize) -> (usize, usize) {
        (
            offset.saturating_add(self.loc.start as isize) as usize,
            // make the end location exclusive
            offset.saturating_add(self.loc.end as isize + 1) as usize,
        )
    }
}

impl From<Match<'_>> for Location {
    fn from(src: Match) -> Self {
        Location { start: src.start(), end: src.end() }
    }
}

impl From<Loc> for Location {
    fn from(src: Loc) -> Self {
        Location { start: src.1, end: src.2 }
    }
}

fn read_node(file: impl AsRef<Path>) -> Result<Node> {
    let file = file.as_ref();
    let source = Source::read(file).map_err(SolcError::Resolve)?;
    let data = parse_data(source.as_ref(), file);
    Ok(Node { path: file.to_path_buf(), source, data })
}

/// Extracts the useful data from a solidity source
///
/// This will attempt to parse the solidity AST and extract the imports and version pragma. If
/// parsing fails, we'll fall back to extract that info via regex
fn parse_data(content: &str, file: &Path) -> SolData {
    println!("parsing {}", file.display());
    let mut version = None;
    let mut imports = Vec::<SolDataUnit<PathBuf>>::new();
    match solang_parser::parse(content, 0) {
        Ok((units, _)) => {
            // println!("{:?}", units.0);
            for unit in units.0 {
                match unit {
                    SourceUnitPart::PragmaDirective(loc, _, pragma, value) => {
                        if pragma.name == "solidity" {
                            // we're only interested in the solidity version pragma
                            version = Some(SolDataUnit::new(value.string, loc.into()));
                        }
                    }
                    SourceUnitPart::ImportDirective(_, import) => {
                        let (import, loc) = match import {
                            Import::Plain(s, l) => (s, l),
                            Import::GlobalSymbol(s, _, l) => (s, l),
                            Import::Rename(s, _, l) => (s, l),
                        };
                        println!("solang {:?} ... {:?}", import, loc);
                        imports.push(SolDataUnit::new(PathBuf::from(import.string), loc.into()));
                    }
                    _ => {}
                }
            }
        }
        Err(err) => {
            println!("solang failed!!!");
            tracing::trace!(
                "failed to parse \"{}\" ast: \"{:?}\". Falling back to regex to extract data",
                file.display(),
                err
            );
            version = capture_outer_and_inner(content, &utils::RE_SOL_PRAGMA_VERSION, &["version"])
                .first()
                .map(|(cap, name)| {
                    SolDataUnit::new(name.as_str().to_owned(), cap.to_owned().into())
                });
            imports = utils::find_import_paths(content)
                .map(|m| SolDataUnit::new(PathBuf::from(m.as_str()), m.to_owned().into()))
                .collect();
        }
    };
    let license = content.lines().next().and_then(|line| {
        capture_outer_and_inner(line, &utils::RE_SOL_SDPX_LICENSE_IDENTIFIER, &["license"])
            .first()
            .map(|(cap, l)| SolDataUnit::new(l.as_str().to_owned(), cap.to_owned().into()))
    });
    let version_req = version.as_ref().and_then(|v| Solc::version_req(v.data()).ok());
    SolData { version_req, version, imports, license }
}

/// Given the regex and the target string, find all occurrences
/// of named groups within the string. This method returns
/// the tuple of matches `(a, b)` where `a` is the match for the
/// entire regex and `b` is the match for the first named group.
///
/// NOTE: This method will return the match for the first named
/// group, so the order of passed named groups matters.
fn capture_outer_and_inner<'a>(
    content: &'a str,
    regex: &regex::Regex,
    names: &[&str],
) -> Vec<(regex::Match<'a>, regex::Match<'a>)> {
    regex
        .captures_iter(content)
        .filter_map(|cap| {
            let cap_match = names.iter().find_map(|name| cap.name(name));
            cap_match.and_then(|m| cap.get(0).map(|outer| (outer.to_owned(), m)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_resolve_hardhat_dependency_graph() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/hardhat-sample");
        let paths = ProjectPathsConfig::hardhat(root).unwrap();

        let graph = Graph::resolve(&paths).unwrap();

        assert_eq!(graph.edges.num_input_files, 1);
        assert_eq!(graph.files().len(), 2);

        assert_eq!(
            graph.files().clone(),
            HashMap::from([
                (paths.sources.join("Greeter.sol"), 0),
                (paths.root.join("node_modules/hardhat/console.sol"), 1),
            ])
        );
    }

    #[test]
    fn can_resolve_dapp_dependency_graph() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/dapp-sample");
        let paths = ProjectPathsConfig::dapptools(root).unwrap();

        let graph = Graph::resolve(&paths).unwrap();

        assert_eq!(graph.edges.num_input_files, 2);
        assert_eq!(graph.files().len(), 3);
        assert_eq!(
            graph.files().clone(),
            HashMap::from([
                (paths.sources.join("Dapp.sol"), 0),
                (paths.sources.join("Dapp.t.sol"), 1),
                (paths.root.join("lib/ds-test/src/test.sol"), 2),
            ])
        );

        let dapp_test = graph.node(1);
        assert_eq!(dapp_test.path, paths.sources.join("Dapp.t.sol"));
        assert_eq!(
            dapp_test.data.imports.iter().map(|i| i.data()).collect::<Vec<&PathBuf>>(),
            vec![&PathBuf::from("ds-test/test.sol"), &PathBuf::from("./Dapp.sol")]
        );
        assert_eq!(graph.imported_nodes(1).to_vec(), vec![2, 0]);
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn can_print_dapp_sample_graph() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/dapp-sample");
        let paths = ProjectPathsConfig::dapptools(root).unwrap();
        let graph = Graph::resolve(&paths).unwrap();
        let mut out = Vec::<u8>::new();
        tree::print(&graph, &Default::default(), &mut out).unwrap();

        assert_eq!(
            "
src/Dapp.sol >=0.6.6
src/Dapp.t.sol >=0.6.6
├── lib/ds-test/src/test.sol >=0.4.23
└── src/Dapp.sol >=0.6.6
"
            .trim_start()
            .as_bytes()
            .to_vec(),
            out
        );
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn can_print_hardhat_sample_graph() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/hardhat-sample");
        let paths = ProjectPathsConfig::hardhat(root).unwrap();
        let graph = Graph::resolve(&paths).unwrap();
        let mut out = Vec::<u8>::new();
        tree::print(&graph, &Default::default(), &mut out).unwrap();
        assert_eq!(
            "
contracts/Greeter.sol >=0.6.0
└── node_modules/hardhat/console.sol >= 0.4.22 <0.9.0
"
            .trim_start()
            .as_bytes()
            .to_vec(),
            out
        );
    }
}
