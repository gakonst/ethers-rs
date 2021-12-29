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
    collections::{HashMap, VecDeque},
    path::{Component, Path, PathBuf},
};

use rayon::prelude::*;
use solang::parser::pt::{Import, SourceUnitPart};

use crate::{error::Result, utils, ProjectPathsConfig, Source, Sources};

/// Represents a fully-resolved solidity dependency graph. Each node in the graph
/// is a file and edges represent dependencies between them.
/// See also https://docs.soliditylang.org/en/latest/layout-of-source-files.html?highlight=import#importing-other-source-files
#[derive(Debug)]
pub struct Graph {
    nodes: Vec<Node>,
    /// The indexes of `edges` correspond to the `nodes`. That is, `edges[0]`
    /// is the set of outgoing edges for `nodes[0]`.
    edges: Vec<Vec<usize>>,
    /// index maps for a solidity file to an index, for fast lookup.
    indices: HashMap<PathBuf, usize>,
    /// with how many input files we started with, corresponds to `let input_files =
    /// nodes[..num_input_files]`.
    num_input_files: usize,
}

impl Graph {
    /// Returns a list of nodes the given node index points to for the given kind.
    pub fn imported_nodes(&self, from: usize) -> &[usize] {
        &self.edges[from]
    }

    /// Gets a node by index.
    pub fn node(&self, index: usize) -> &Node {
        &self.nodes[index]
    }

    /// Returns all source files
    pub fn into_sources(self) -> Sources {
        self.nodes.into_iter().map(|node| (node.path, node.source)).collect()
    }
}

#[derive(Debug)]
pub struct Node {
    path: PathBuf,
    source: Source,
    data: SolData,
}

#[derive(Debug, Clone)]
struct SolData {
    version: Option<String>,
    imports: Vec<PathBuf>,
}

/// Resolves a number of sources within the given config
pub fn resolve_sources(paths: &ProjectPathsConfig, sources: Sources) -> Result<Graph> {
    // we start off by reading all input files, which includes all solidity files from the source
    // and test folder
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
        let node_dir = path.parent();
        if node_dir.is_none() {
            continue
        }
        let node_dir = node_dir.unwrap();

        for import in node.data.imports.iter() {
            let component = import.components().next();
            if component.is_none() {
                continue
            }
            let component = component.unwrap();
            if component == Component::CurDir || component == Component::ParentDir {
                // if the import is relative we assume it's already part of the processed input file
                // set
                match utils::canonicalize(node_dir.join(import)) {
                    Ok(target) => {
                        // the file at least exists,
                        if let Some(idx) = index.get(&target).cloned() {
                            resolved_imports.push(idx);
                        } else {
                            // imported file is not part of the input files
                            let node = read_node(&target)?;
                            unresolved.push_back((target.clone(), node));
                            let idx = index.len();
                            index.insert(target.clone(), idx);
                            resolved_imports.push(idx);
                        }
                    }
                    Err(err) => {
                        tracing::trace!("failed to resolve relative import \"{:?}\"", err);
                    }
                }
            } else {
                // resolve library file
                if let Some(lib) = paths.resolve_library_import(import.as_ref()) {
                    if let Some(idx) = index.get(&lib).cloned() {
                        resolved_imports.push(idx);
                    } else {
                        // imported file is not part of the input files
                        let node = read_node(&lib)?;
                        unresolved.push_back((lib.clone(), node));
                        let idx = index.len();
                        index.insert(lib.clone(), idx);
                        resolved_imports.push(idx);
                    }
                } else {
                    tracing::trace!("failed to resolve library import \"{:?}\"", import.display());
                }
            }
        }
        nodes.push(node);
        edges.push(resolved_imports);
    }

    Ok(Graph { nodes, edges, indices: index, num_input_files })
}

/// Resolves the dependencies of a project's source contracts
pub fn resolve(paths: &ProjectPathsConfig) -> Result<Graph> {
    resolve_sources(paths, paths.read_input_files()?)
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
    let mut imports = Vec::new();
    match solang::parser::parse(content, 0) {
        Ok(units) => {
            for unit in units.0 {
                match unit {
                    SourceUnitPart::PragmaDirective(_, pragma, value) => {
                        if pragma.name == "solidity" {
                            // we're only interested in the solidity version pragma
                            version = Some(value.string);
                        }
                    }
                    SourceUnitPart::ImportDirective(_, import) => {
                        let import = match import {
                            Import::Plain(s) => s,
                            Import::GlobalSymbol(s, _) => s,
                            Import::Rename(s, _) => s,
                        };
                        imports.push(PathBuf::from(import.string));
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
            version = utils::find_version_pragma(content).map(str::to_string);
            imports = utils::find_import_paths(content)
                .into_iter()
                .map(|p| Path::new(p).to_path_buf())
                .collect()
        }
    };
    SolData { version, imports }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_resolve_dependency_graph() {
        let paths =
            ProjectPathsConfig::dapptools("../../foundry-integration-tests/testdata/solmate")
                .unwrap();

        let graph = resolve(&paths).unwrap();

        for (path, idx) in &graph.indices {
            println!("{}", path.display());
            for dep in &graph.edges[*idx] {
                println!("    {}", graph.node(*dep).path.display());
            }
            println!();
        }
    }
}
