//! Support for compiling contracts
use crate::{
    artifacts::Sources,
    config::SolcConfig,
    error::{Result, SolcError},
    utils,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
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
    /// let config = SolFilesCache::builder().insert_files(files).unwrap();
    /// ```
    pub fn builder() -> SolFilesCacheBuilder {
        SolFilesCacheBuilder::default()
    }

    /// Reads the cache json file from the given path
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let file = fs::File::open(path.as_ref())?;
        Ok(serde_json::from_reader(file)?)
    }

    /// Write the cache to json file
    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = fs::File::create(path.as_ref())?;
        Ok(serde_json::to_writer_pretty(file, self)?)
    }

    pub fn remove_missing_files(&mut self) {
        self.files.retain(|file, _| Path::new(file).exists())
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

    pub fn insert_files(self, sources: Sources) -> Result<SolFilesCache> {
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

        Ok(SolFilesCache { format, files })
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
