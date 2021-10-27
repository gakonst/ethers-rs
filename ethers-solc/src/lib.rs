//! Support for compiling contracts

pub mod artifacts;

pub use artifacts::{CompilerInput, CompilerOutput, EvmVersion};

pub mod cache;

mod compile;
pub use compile::Solc;

mod config;
use crate::{artifacts::Source, cache::SolFilesCache, config::ArtifactOutput};
pub use config::ProjectPathsConfig;

pub mod error;
pub mod utils;
use error::Result;

/// Handles contract compiling
#[derive(Debug)]
pub struct Project {
    /// The layout of the
    pub config: ProjectPathsConfig,
    /// Where to find solc
    pub solc: Solc,
    /// Whether caching is enabled
    pub cached: bool,
    /// How to handle compiler output
    pub artifacts: ArtifactOutput,
}

impl Project {
    /// New compile project without cache support.
    pub fn new(config: ProjectPathsConfig, solc: Solc, artifacts: ArtifactOutput) -> Self {
        Self { config, solc, cached: false, artifacts }
    }

    /// Enable cache.
    pub fn cached(mut self) -> Self {
        self.cached = true;
        self
    }

    pub fn compile(&self) -> Result<()> {
        let _sources = Source::read_all_from(self.config.sources.as_path())?;
        if self.cached {
            let _cache = if self.config.cache.exists() {
                SolFilesCache::read(&self.config.cache)?
            } else {
                SolFilesCache::default()
            };
        }

        unimplemented!()
    }
}
