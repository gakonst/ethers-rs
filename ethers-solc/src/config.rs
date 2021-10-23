use crate::cache::SOLIDITY_FILES_CACHE_FILENAME;
use std::{io, path::PathBuf};

/// Where to find all files or where to write them
#[derive(Debug, Clone)]
pub struct ProjectPathsConfig {
    pub root: PathBuf,
    pub cache: PathBuf,
    pub artifacts: PathBuf,
    pub sources: PathBuf,
    pub tests: PathBuf,
}

impl ProjectPathsConfig {
    /// Creates a new config instance which points to the canonicalized root
    /// path
    pub fn new(root: impl Into<PathBuf>) -> io::Result<Self> {
        let root = std::fs::canonicalize(root.into())?;
        Ok(Self {
            cache: root.join("cache").join(SOLIDITY_FILES_CACHE_FILENAME),
            artifacts: root.join("artifacts"),
            sources: root.join("contracts"),
            tests: root.join("tests"),
            root,
        })
    }
}
