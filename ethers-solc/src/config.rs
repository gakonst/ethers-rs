use crate::{
    artifacts::{CompactContractRef, Settings},
    cache::SOLIDITY_FILES_CACHE_FILENAME,
    error::Result,
    CompilerOutput, Solc,
};
use serde::{Deserialize, Serialize};
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

    /// Creates a new config with the current directory as the root
    pub fn current() -> io::Result<Self> {
        Self::new(std::env::current_dir()?)
    }
}

/// The config to use when compiling the contracts
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SolcConfig {
    /// Configured solc version
    pub version: String,
    /// How the file was compiled
    pub settings: Settings,
}

impl SolcConfig {
    /// # Example
    ///
    /// Autodetect solc version and default settings
    ///
    /// ```rust
    /// use ethers_solc::SolcConfig;
    /// let config = SolcConfig::builder().build().unwrap();
    /// ```
    pub fn builder() -> SolcConfigBuilder {
        SolcConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct SolcConfigBuilder {
    version: Option<String>,
    settings: Option<Settings>,
}

impl SolcConfigBuilder {
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn settings(mut self, settings: Settings) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Creates the solc config
    ///
    /// If no solc version is configured then it will be determined by calling `solc --version`.
    pub fn build(self) -> Result<SolcConfig> {
        let Self { version, settings } = self;
        let version = version
            .map(Ok)
            .unwrap_or_else(|| Solc::default().version().map(|s| s.to_string()))?;
        let settings = settings.unwrap_or_default();
        Ok(SolcConfig { version, settings })
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

impl Default for ArtifactOutput {
    fn default() -> Self {
        ArtifactOutput::MinimalCombined
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
