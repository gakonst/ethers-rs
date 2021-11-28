//! Support for compiling contracts
use crate::{
    artifacts::Sources,
    config::SolcConfig,
    error::{Result, SolcError},
    utils, ArtifactOutput,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

/// Hardhat format version
const HH_FORMAT_VERSION: &str = "hh-sol-cache-2";

/// The file name of the default cache file
pub const SOLIDITY_FILES_CACHE_FILENAME: &str = "solidity-files-cache.json";

/// A hardhat compatible cache representation
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SolFilesCache {
    #[serde(rename = "_format")]
    pub format: String,
    pub files: BTreeMap<PathBuf, CacheEntry>,
}

impl SolFilesCache {
    /// # Example
    ///
    /// Autodetect solc version and default settings
    ///
    /// ```no_run
    /// use ethers_solc::artifacts::Source;
    /// use ethers_solc::cache::SolFilesCache;
    /// let files = Source::read_all_from("./sources").unwrap();
    /// let config = SolFilesCache::builder().insert_files(files, None).unwrap();
    /// ```
    pub fn builder() -> SolFilesCacheBuilder {
        SolFilesCacheBuilder::default()
    }

    /// Reads the cache json file from the given path
    #[tracing::instrument(skip_all, name = "sol-files-cache::read")]
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        tracing::trace!("reading solfiles cache at {}", path.display());
        let file = fs::File::open(path)?;
        let file = std::io::BufReader::new(file);
        let cache = serde_json::from_reader(file)?;
        tracing::trace!("done");
        Ok(cache)
    }

    /// Write the cache to json file
    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let file = fs::File::create(path)?;
        tracing::trace!("writing cache to json file");
        serde_json::to_writer_pretty(file, self)?;
        tracing::trace!("cache file located: {}", path.display());
        Ok(())
    }

    pub fn remove_missing_files(&mut self) {
        self.files.retain(|file, _| Path::new(file).exists())
    }

    pub fn remove_changed_files(&mut self, changed_files: &Sources) {
        self.files.retain(|file, _| !changed_files.contains_key(file))
    }

    /// Returns only the files that were changed from the provided sources, to save time
    /// when compiling.
    pub fn get_changed_files<'a>(
        &'a self,
        sources: Sources,
        config: Option<&'a SolcConfig>,
    ) -> Sources {
        sources
            .into_iter()
            .filter(move |(file, source)| self.has_changed(file, source.content_hash(), config))
            .collect()
    }

    /// Returns true if the given content hash or config differs from the file's
    /// or the file does not exist
    pub fn has_changed(
        &self,
        file: impl AsRef<Path>,
        hash: impl AsRef<[u8]>,
        config: Option<&SolcConfig>,
    ) -> bool {
        if let Some(entry) = self.files.get(file.as_ref()) {
            if entry.content_hash.as_bytes() != hash.as_ref() {
                return true
            }
            if let Some(config) = config {
                if config != &entry.solc_config {
                    return true
                }
            }
            false
        } else {
            true
        }
    }

    /// Returns only the files that were changed or are missing artifacts compared to previous
    /// compiler execution, to save time when compiling.
    pub fn get_changed_or_missing_artifacts_files<'a, T: ArtifactOutput>(
        &'a self,
        sources: Sources,
        config: Option<&'a SolcConfig>,
        artifacts_root: &Path,
    ) -> Sources {
        sources
            .into_iter()
            .filter(move |(file, source)| {
                self.has_changed_or_missing_artifact::<T>(
                    file,
                    source.content_hash().as_bytes(),
                    config,
                    artifacts_root,
                )
            })
            .collect()
    }

    /// Returns true if the given content hash or config differs from the file's
    /// or the file does not exist or the files' artifacts are missing
    pub fn has_changed_or_missing_artifact<T: ArtifactOutput>(
        &self,
        file: &Path,
        hash: &[u8],
        config: Option<&SolcConfig>,
        artifacts_root: &Path,
    ) -> bool {
        if let Some(entry) = self.files.get(file) {
            if entry.content_hash.as_bytes() != hash {
                return true
            }
            if let Some(config) = config {
                if config != &entry.solc_config {
                    return true
                }
            }

            entry.artifacts.iter().any(|name| !T::output_exists(file, name, artifacts_root))
        } else {
            true
        }
    }

    /// Checks if all artifact files exist
    pub fn all_artifacts_exist<T: ArtifactOutput>(&self, artifacts_root: &Path) -> bool {
        self.files.iter().all(|(file, entry)| {
            entry.artifacts.iter().all(|name| T::output_exists(file, name, artifacts_root))
        })
    }

    /// Reads all cached artifacts from disk
    pub fn read_artifacts<T: ArtifactOutput>(
        &self,
        artifacts_root: &Path,
    ) -> Result<BTreeMap<PathBuf, T::Artifact>> {
        let mut artifacts = BTreeMap::default();
        for (file, entry) in &self.files {
            for artifact in &entry.artifacts {
                let artifact_file = artifacts_root.join(T::output_file(file, artifact));
                let artifact = T::read_cached_artifact(&artifact_file)?;
                artifacts.insert(artifact_file, artifact);
            }
        }
        Ok(artifacts)
    }
}

#[cfg(feature = "async")]
impl SolFilesCache {
    pub async fn async_read(path: impl AsRef<Path>) -> Result<Self> {
        let content = tokio::fs::read_to_string(path.as_ref()).await?;
        Ok(serde_json::from_str(&content)?)
    }

    pub async fn async_write(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = serde_json::to_vec_pretty(self)?;
        Ok(tokio::fs::write(path.as_ref(), content).await?)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SolFilesCacheBuilder {
    format: Option<String>,
    solc_config: Option<SolcConfig>,
    root: Option<PathBuf>,
}

impl SolFilesCacheBuilder {
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    pub fn solc_config(mut self, solc_config: SolcConfig) -> Self {
        self.solc_config = Some(solc_config);
        self
    }

    pub fn root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    pub fn insert_files(self, sources: Sources, dest: Option<PathBuf>) -> Result<SolFilesCache> {
        let format = self.format.unwrap_or_else(|| HH_FORMAT_VERSION.to_string());
        let solc_config =
            self.solc_config.map(Ok).unwrap_or_else(|| SolcConfig::builder().build())?;

        let root = self.root.map(Ok).unwrap_or_else(std::env::current_dir)?;

        let mut files = BTreeMap::new();
        for (file, source) in sources {
            let last_modification_date = fs::metadata(&file)?
                .modified()?
                .duration_since(UNIX_EPOCH)
                .map_err(|err| SolcError::solc(err.to_string()))?
                .as_millis() as u64;
            let imports =
                utils::find_import_paths(source.as_ref()).into_iter().map(str::to_string).collect();

            let version_pragmas = utils::find_version_pragma(source.as_ref())
                .map(|v| vec![v.to_string()])
                .unwrap_or_default();

            let entry = CacheEntry {
                last_modification_date,
                content_hash: source.content_hash(),
                source_name: utils::source_name(&file, &root).into(),
                solc_config: solc_config.clone(),
                imports,
                version_pragmas,
                // TODO detect artifacts
                artifacts: vec![],
            };
            files.insert(file, entry);
        }

        let cache = if let Some(ref dest) = dest {
            if dest.exists() {
                // read the existing cache and extend it by the files that changed
                // (if we just wrote to the cache file, we'd overwrite the existing data)
                let reader = std::io::BufReader::new(File::open(dest)?);
                let mut cache: SolFilesCache = serde_json::from_reader(reader)?;
                assert_eq!(cache.format, format);
                cache.files.extend(files);
                cache
            } else {
                SolFilesCache { format, files }
            }
        } else {
            SolFilesCache { format, files }
        };

        Ok(cache)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheEntry {
    /// the last modification time of this file
    pub last_modification_date: u64,
    pub content_hash: String,
    pub source_name: PathBuf,
    pub solc_config: SolcConfig,
    pub imports: Vec<String>,
    pub version_pragmas: Vec<String>,
    pub artifacts: Vec<String>,
}

impl CacheEntry {
    /// Returns the time
    pub fn last_modified(&self) -> Duration {
        Duration::from_millis(self.last_modification_date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_solidity_files_cache() {
        let input = include_str!("../test-data/solidity-files-cache.json");
        let _ = serde_json::from_str::<SolFilesCache>(input).unwrap();
    }
}
