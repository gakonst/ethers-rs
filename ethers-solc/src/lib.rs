//! Support for compiling contracts

pub mod artifacts;

pub use artifacts::{CompilerInput, CompilerOutput, EvmVersion};

pub mod cache;

mod compile;
pub use compile::Solc;

mod config;
pub use config::{ArtifactOutput, ProjectPathsConfig, SolcConfig};

use crate::{artifacts::Source, cache::SolFilesCache};

pub mod error;
pub mod utils;
use crate::artifacts::Sources;
use error::Result;

/// Handles contract compiling
#[derive(Debug)]
pub struct Project {
    /// The layout of the
    pub paths: ProjectPathsConfig,
    /// Where to find solc
    pub solc: Solc,
    /// How solc invocation should be configured.
    pub solc_config: SolcConfig,
    /// Whether caching is enabled
    pub cached: bool,
    /// How to handle compiler output
    pub artifacts: ArtifactOutput,
}

impl Project {
    /// Configure the current project
    ///
    /// # Example
    ///
    /// ```rust
    /// use ethers_solc::Project;
    /// let config = Project::builder().build().unwrap();
    /// ```
    pub fn builder() -> ProjectBuilder {
        ProjectBuilder::default()
    }

    fn write_cache_file(&self, sources: Sources) -> Result<()> {
        let cache = SolFilesCache::builder()
            .root(&self.paths.root)
            .solc_config(self.solc_config.clone())
            .insert_files(sources)?;
        cache.write(&self.paths.cache)
    }

    /// Attempts to compile the contracts found at the configured location.
    ///
    /// Returns the `Some(CompilerOutput)` of solc.
    /// NOTE: this does not check if the contracts were successfully compiled.
    ///
    /// Returns `None` if caching is enabled and there was nothing to compile.
    pub fn compile(&self) -> Result<Option<CompilerOutput>> {
        let sources = Source::read_all_from(self.paths.sources.as_path())?;
        if self.cached {
            if self.paths.cache.exists() {
                // check anything changed
                let cache = SolFilesCache::read(&self.paths.cache)?;
                if !cache.is_changed(&sources, Some(&self.solc_config)) {
                    return Ok(None);
                }
            }
            // create cache file
            self.write_cache_file(sources.clone())?;
        }

        // TODO handle special imports
        let input = CompilerInput::with_sources(sources);
        let output = self.solc.compile(&input)?;
        self.artifacts.on_output(&output, &self.paths).unwrap();
        Ok(Some(output))
    }
}

pub struct ProjectBuilder {
    /// The layout of the
    paths: Option<ProjectPathsConfig>,
    /// Where to find solc
    solc: Option<Solc>,
    /// How solc invocation should be configured.
    solc_config: Option<SolcConfig>,
    /// Whether caching is enabled, default is true.
    cached: bool,
    /// How to handle compiler output
    artifacts: Option<ArtifactOutput>,
}

impl ProjectBuilder {
    pub fn paths(mut self, paths: ProjectPathsConfig) -> Self {
        self.paths = Some(paths);
        self
    }

    pub fn solc(mut self, solc: impl Into<Solc>) -> Self {
        self.solc = Some(solc.into());
        self
    }

    pub fn solc_config(mut self, solc_config: SolcConfig) -> Self {
        self.solc_config = Some(solc_config);
        self
    }

    pub fn artifacts(mut self, artifacts: ArtifactOutput) -> Self {
        self.artifacts = Some(artifacts);
        self
    }

    /// Disables cached builds
    pub fn ephemeral(mut self) -> Self {
        self.cached = false;
        self
    }

    pub fn build(self) -> Result<Project> {
        let Self {
            paths,
            solc,
            solc_config,
            cached,
            artifacts,
        } = self;

        let solc = solc.unwrap_or_default();
        let solc_config = solc_config.map(Ok).unwrap_or_else(|| {
            let version = solc.version()?;
            SolcConfig::builder().version(version.to_string()).build()
        })?;

        Ok(Project {
            paths: paths.map(Ok).unwrap_or_else(ProjectPathsConfig::current)?,
            solc,
            solc_config,
            cached,
            artifacts: artifacts.unwrap_or_default(),
        })
    }
}

impl Default for ProjectBuilder {
    fn default() -> Self {
        Self {
            paths: None,
            solc: None,
            solc_config: None,
            cached: true,
            artifacts: None,
        }
    }
}
