//! Dependency graph for a solidity project

use std::path::{PathBuf};

/// Represents a set of files and their dependencies on each other
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub root: PathBuf,
}

impl DependencyGraph {
    pub fn new(root: impl Into<PathBuf>) -> eyre::Result<Self> {
        let _root = root.into();

        todo!()
    }
}
