use crate::{
    artifacts::CompactContractRef, cache::SOLIDITY_FILES_CACHE_FILENAME, error::Result,
    CompilerOutput,
};
use std::{fmt, fs, io, path::PathBuf};

/// Where to find all files or where to write them
#[derive(Debug, Clone)]
pub struct ProjectPathsConfig {
    /// Project root
    pub root: PathBuf,
    /// Path to the cache, if any
    pub cache: PathBuf,
    /// Where to store build artifacts
    pub artifacts: PathBuf,
    /// Where to find sources
    pub sources: PathBuf,
    /// Where to find tests
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

/// Determines how to handle compiler output
pub enum ArtifactOutput {
    /// Creates a single json artifact with
    /// ```json
    ///  {
    ///    "abi": [],
    ///    "bin": "...",
    ///    "runtime-bin": "..."
    ///  }
    /// ```
    MinimalCombined,
    /// Hardhat style artifacts
    Hardhat,
    /// Custom output handler
    Custom(Box<dyn Fn(&CompilerOutput, &ProjectPathsConfig) -> Result<()>>),
}

impl ArtifactOutput {
    /// Is expected to handle the output and where to store it
    pub fn on_output(&self, output: &CompilerOutput, layout: &ProjectPathsConfig) -> Result<()> {
        match self {
            ArtifactOutput::MinimalCombined => {
                for contracts in output.contracts.values() {
                    for (name, contract) in contracts {
                        let file = layout.root.join(format!("{}.json", name));
                        let min = CompactContractRef::from(contract);
                        fs::write(file, serde_json::to_vec_pretty(&min)?)?
                    }
                }
                Ok(())
            }
            ArtifactOutput::Hardhat => {
                todo!("Hardhat style artifacts not yet implemented")
            }
            ArtifactOutput::Custom(f) => f(output, layout),
        }
    }
}

impl fmt::Debug for ArtifactOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArtifactOutput::MinimalCombined => {
                write!(f, "MinimalCombined")
            }
            ArtifactOutput::Hardhat => {
                write!(f, "Hardhat")
            }
            ArtifactOutput::Custom(_) => {
                write!(f, "Custom")
            }
        }
    }
}
