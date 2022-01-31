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
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use regex::Match;
use semver::VersionReq;
use solang_parser::pt::{Import, Loc, SourceUnitPart};

use crate::{error::Result, utils, ProjectPathsConfig, Solc, Source, Sources};

/// Represents a fully-resolved solidity dependency graph. Each node in the graph
/// is a file and edges represent dependencies between them.
/// See also https://docs.soliditylang.org/en/latest/layout-of-source-files.html?highlight=import#importing-other-source-files
#[derive(Debug)]
pub struct Graph {
    nodes: Vec<Node>,
    /// The indices of `edges` correspond to the `nodes`. That is, `edges[0]`
    /// is the set of outgoing edges for `nodes[0]`.
    edges: Vec<Vec<usize>>,
    /// index maps for a solidity file to an index, for fast lookup.
    indices: HashMap<PathBuf, usize>,
    /// with how many input files we started with, corresponds to `let input_files =
    /// nodes[..num_input_files]`.
    num_input_files: usize,
    /// the root of the project this graph represents
    #[allow(unused)]
    root: PathBuf,
}

impl Graph {
    /// Returns a list of nodes the given node index points to for the given kind.
    pub fn imported_nodes(&self, from: usize) -> &[usize] {
        &self.edges[from]
    }

    /// Returns all the resolved files and their index in the graph
    pub fn files(&self) -> &HashMap<PathBuf, usize> {
        &self.indices
    }

    /// Gets a node by index.
    pub fn node(&self, index: usize) -> &Node {
        &self.nodes[index]
    }

    /// Returns an iterator that yields all nodes of the dependency tree that the given node id
    /// spans, starting with the node itself.
    ///
    /// # Panics
    ///
    /// if the `start` node id is not included in the graph
    pub fn node_ids(&self, start: usize) -> impl Iterator<Item = usize> + '_ {
        NodesIter::new(start, self)
    }

    /// Same as `Self::node_ids` but returns the actual `Node`
    pub fn nodes(&self, start: usize) -> impl Iterator<Item = &Node> + '_ {
        self.node_ids(start).map(move |idx| self.node(idx))
    }

    /// Returns all files together with their paths
    pub fn into_sources(self) -> Sources {
        self.nodes.into_iter().map(|node| (node.path, node.source)).collect()
    }

    /// Returns an iterator that yields only those nodes that represent input files.
    /// See `Self::resolve_sources`
    /// This won't yield any resolved library nodes
    pub fn input_nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().take(self.num_input_files)
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

        // we start off by reading all input files, which includes all solidity files from the
        // source and test folder
        let mut unresolved: VecDeque<(PathBuf, Node)> = sources
            .into_par_iter()
            .map(|(path, source)| {
                let data = parse_data(source.as_ref());
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
                    Err(err) => tracing::trace!("failed to resolve import component \"{:?}\"", err),
                };
            }
            nodes.push(node);
            edges.push(resolved_imports);
        }

        Ok(Graph { nodes, edges, indices: index, num_input_files, root: paths.root.clone() })
    }

    /// Resolves the dependencies of a project's source contracts
    pub fn resolve(paths: &ProjectPathsConfig) -> Result<Graph> {
        Self::resolve_sources(paths, paths.read_input_files()?)
    }
}

#[cfg(all(feature = "svm", feature = "async"))]
impl Graph {
    /// Returns all input files together with their appropriate version.
    ///
    /// First we determine the compatible version for each input file (from sources and test folder,
    /// see `Self::resolve`) and then we add all resolved library imports.
    pub fn into_sources_by_version(self, offline: bool) -> Result<VersionedSources> {
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
        let Self { nodes, edges, num_input_files, .. } = self;
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
                insert_imports(idx, &mut all_nodes, &mut sources, &edges, num_input_files);
            }
            versioned_sources.insert(version, sources);
        }
        Ok(VersionedSources { inner: versioned_sources, offline })
    }

    /// Writes the list of imported files into the given formatter:
    /// `A (version) imports B (version)`
    fn format_imports_list<W: std::fmt::Write>(
        &self,
        idx: usize,
        f: &mut W,
    ) -> std::result::Result<(), std::fmt::Error> {
        let node = self.node(idx);
        for dep in self.imported_nodes(idx) {
            let dep = self.node(*dep);
            writeln!(
                f,
                "  {} ({:?}) imports {} ({:?})",
                utils::source_name(&node.path, &self.root).display(),
                node.data.version,
                utils::source_name(&dep.path, &self.root).display(),
                dep.data.version
            )?;
        }
        for dep in self.imported_nodes(idx) {
            self.format_imports_list(*dep, f)?;
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
        let mut erroneous_nodes = std::collections::HashSet::with_capacity(self.num_input_files);

        let all_versions = if offline { Solc::installed_versions() } else { Solc::all_versions() };

        // stores all versions and their nodes
        let mut versioned_nodes = HashMap::new();

        // walking through the node's dep tree and filtering the versions along the way
        for idx in 0..self.num_input_files {
            let mut candidates = all_versions.iter().collect::<Vec<_>>();
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
                let candidate = (*candidates
                    .iter()
                    .rev()
                    .find(|v| v.is_installed())
                    .or_else(|| candidates.iter().last())
                    .unwrap())
                .clone();
                versioned_nodes.entry(candidate).or_insert_with(|| Vec::with_capacity(1)).push(idx);
            }
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
            Err(crate::error::SolcError::msg(errors.join("\n")))
        }
    }
}

/// An iterator over a node and its dependencies
#[derive(Debug)]
pub struct NodesIter<'a> {
    /// stack of nodes
    stack: VecDeque<usize>,
    visited: HashSet<usize>,
    graph: &'a Graph,
}

impl<'a> NodesIter<'a> {
    fn new(start: usize, graph: &'a Graph) -> Self {
        Self { stack: VecDeque::from([start]), visited: Default::default(), graph }
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
    ) -> Result<std::collections::BTreeMap<Solc, Sources>> {
        use crate::SolcError;

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
            sources_by_version
                .insert(solc.arg("--allow-paths").arg(allowed_lib_paths.to_string()), sources);
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

#[derive(Debug, Clone)]
#[allow(unused)]
struct SolData {
    license: Option<SolDataUnit<String>>,
    version: Option<SolDataUnit<String>>,
    imports: Vec<SolDataUnit<PathBuf>>,
    version_req: Option<VersionReq>,
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
    let source = Source::read(file)?;
    let data = parse_data(source.as_ref());
    Ok(Node { path: file.to_path_buf(), source, data })
}

/// Extracts the useful data from a solidity source
///
/// This will attempt to parse the solidity AST and extract the imports and version pragma. If
/// parsing fails, we'll fall back to extract that info via regex
fn parse_data(content: &str) -> SolData {
    let mut version = None;
    let mut imports = Vec::<SolDataUnit<PathBuf>>::new();
    match solang_parser::parse(content, 0) {
        Ok(units) => {
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
                        imports.push(SolDataUnit::new(PathBuf::from(import.string), loc.into()));
                    }
                    _ => {}
                }
            }
        }
        Err(err) => {
            tracing::trace!(
                "failed to parse solidity ast: \"{:?}\". Falling back to regex to extract data",
                err
            );
            version = capture_outer_and_inner(content, &utils::RE_SOL_PRAGMA_VERSION, &["version"])
                .first()
                .map(|(cap, name)| {
                    SolDataUnit::new(name.as_str().to_owned(), cap.to_owned().into())
                });
            imports = capture_outer_and_inner(content, &utils::RE_SOL_IMPORT, &["p1", "p2", "p3"])
                .iter()
                .map(|(cap, m)| SolDataUnit::new(PathBuf::from(m.as_str()), cap.to_owned().into()))
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

        assert_eq!(graph.num_input_files, 1);
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

        assert_eq!(graph.num_input_files, 2);
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
}
