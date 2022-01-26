//! Support for compiling contracts
use crate::{
    artifacts::{Contracts, Sources},
    config::SolcConfig,
    error::{Result, SolcError},
    utils, ArtifactOutput, ProjectPathsConfig,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::{self, File},
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

/// Hardhat format version
const HH_FORMAT_VERSION: &str = "hh-sol-cache-2";

/// ethers-rs format version
///
/// `ethers-solc` uses a different format version id, but the actual format is consistent with
/// hardhat This allows ethers-solc to detect if the cache file was written by hardhat or
/// `ethers-solc`
const ETHERS_FORMAT_VERSION: &str = "ethers-rs-sol-cache-2";

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

    /// This returns a `SolFilesCache` that has read the cache file located
    /// at the default cache location as provided by `ProjectPathsConfig`
    pub fn cache_populated_from_defaults() -> Result<Self> {
        let paths = ProjectPathsConfig::builder().build()?;
        SolFilesCache::read(paths.cache)
    }

    /// Whether this cache's format is the hardhat format identifier
    pub fn is_hardhat_format(&self) -> bool {
        self.format == HH_FORMAT_VERSION
    }

    /// Whether this cache's format is our custom format identifier
    pub fn is_ethers_format(&self) -> bool {
        // TODO: It may be preferable to test prefix here since we've bumped ETHERS_FORMAT_VERSION
        self.format == ETHERS_FORMAT_VERSION
    }

    /// Reads the cache json file from the given path
    #[tracing::instrument(skip_all, name = "sol-files-cache::read")]
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        tracing::trace!("reading solfiles cache at {}", path.display());
        let file = fs::File::open(path).map_err(|err| SolcError::io(err, path))?;
        let file = std::io::BufReader::new(file);
        let cache: Self = serde_json::from_reader(file)?;
        tracing::trace!("read cache \"{}\" with {} entries", cache.format, cache.files.len());
        Ok(cache)
    }

    /// Write the cache to json file
    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let file = fs::File::create(path).map_err(|err| SolcError::io(err, path))?;
        tracing::trace!("writing cache to json file: \"{}\"", path.display());
        serde_json::to_writer_pretty(file, self)?;
        tracing::trace!("cache file located: \"{}\"", path.display());
        Ok(())
    }

    pub fn remove_missing_files(&mut self) {
        tracing::trace!("remove non existing files from cache");
        self.files.retain(|file, _| Path::new(file).exists())
    }

    pub fn remove_changed_files(&mut self, changed_files: &Sources) {
        tracing::trace!("remove changed files from cache");
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
        paths: &ProjectPathsConfig,
        content_hashes: HashMap<PathBuf, String>,
    ) -> Sources {
        sources
            .into_iter()
            .filter(move |(file, _)| {
                self.has_changed_or_missing_artifact::<T>(file, &content_hashes, config, paths)
            })
            .collect()
    }

    /// Returns true if the given content hash or config differs from the file's
    /// or the file does not exist or the files' artifacts are missing
    pub fn has_changed_or_missing_artifact<T: ArtifactOutput>(
        &self,
        file: &Path,
        hashes: &HashMap<PathBuf, String>,
        config: Option<&SolcConfig>,
        paths: &ProjectPathsConfig,
    ) -> bool {
        let hash = hashes.get(file).unwrap().as_bytes();
        if let Some(entry) = self.files.get(file) {
            if entry.content_hash.as_bytes() != hash {
                tracing::trace!("changed content hash for cached artifact \"{}\"", file.display());
                return true
            }
            if let Some(config) = config {
                if config != &entry.solc_config {
                    tracing::trace!(
                        "changed solc config for cached artifact \"{}\"",
                        file.display()
                    );
                    return true
                }
            }

            // checks whether an artifact this file depends on was removed
            if entry
                .artifact_paths
                .keys()
                .any(|artifact| !T::output_exists(artifact, &paths.artifacts))
            {
                tracing::trace!(
                    "missing linked artifacts for cached artifact \"{}\"",
                    file.display()
                );
                return true
            }

            // check if any of the file's imported files changed
            self.has_changed_imports(file, entry, hashes, paths, &mut HashSet::new())
        } else {
            tracing::trace!("missing cached artifact for \"{}\"", file.display());
            true
        }
    }

    /// Returns true if the entry has any imports that were changed
    fn has_changed_imports(
        &self,
        path: &Path,
        entry: &CacheEntry,
        hashes: &HashMap<PathBuf, String>,
        paths: &ProjectPathsConfig,
        traversed: &mut HashSet<PathBuf>,
    ) -> bool {
        let cwd = match path.parent() {
            Some(inner) => inner,
            None => return true,
        };
        if !traversed.insert(path.to_path_buf()) {
            // skip already traversed files, this prevents SO for circular imports
            return false
        }

        for import in entry.imports.iter() {
            if let Some((import, import_path)) = paths
                .resolve_import(cwd, Path::new(import.as_str()))
                .ok()
                .and_then(|import| self.files.get(&import).map(|e| (e, import)))
            {
                if let Some(hash) = hashes.get(&import_path) {
                    if import.content_hash == hash.as_str() &&
                        !self.has_changed_imports(&import_path, import, hashes, paths, traversed)
                    {
                        return false
                    }
                }
            }
        }

        !entry.imports.is_empty()
    }

    /// Checks if all artifact files exist
    pub fn all_artifacts_exist<T: ArtifactOutput>(&self, artifacts: &Path) -> bool {
        self.files.iter().all(|(_, entry)| {
            entry.artifact_paths.keys().all(|artifact| T::output_exists(artifact, artifacts))
        })
    }

    /// Reads all cached artifacts from disk using the given ArtifactOutput handler
    pub fn read_artifacts<T: ArtifactOutput>(
        &self,
        artifacts_root: &Path,
    ) -> Result<BTreeMap<PathBuf, T::Artifact>> {
        let mut artifacts = BTreeMap::default();
        for entry in self.files.values() {
            for artifact in entry.artifact_paths.keys() {
                let artifact_file = artifacts_root.join(artifact);
                let artifact = T::read_cached_artifact(&artifact_file)?;
                artifacts.insert(artifact_file, artifact);
            }
        }
        Ok(artifacts)
    }

    /// Get source name for cache entry matching artifact name
    pub fn source_name_for_artifact(&self, name: &str) -> Option<PathBuf> {
        for entry in self.files.values() {
            if entry.artifacts.iter().any(|art_name| art_name == name) {
                return Some(entry.source_name.clone())
            }
        }
        None
    }

    /// Get source name for cache entry matching source path
    pub fn source_name_for_source(&self, src_path: &str) -> Option<PathBuf> {
        self.files.get(&PathBuf::from(src_path)).map(|entry| entry.source_name.clone())
    }

    /// Get metadata for artifacts related to source path
    pub fn metadata_for_source(
        &self,
        path: impl AsRef<Path>,
    ) -> Option<(SolcConfig, BTreeMap<PathBuf, String>)> {
        self.files
            .get(path.as_ref())
            .map(|entry| (entry.solc_config.clone(), entry.artifact_paths.clone()))
    }
}

// async variants for read and write
#[cfg(feature = "async")]
impl SolFilesCache {
    pub async fn async_read(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content =
            tokio::fs::read_to_string(path).await.map_err(|err| SolcError::io(err, path))?;
        Ok(serde_json::from_str(&content)?)
    }

    pub async fn async_write(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let content = serde_json::to_vec_pretty(self)?;
        Ok(tokio::fs::write(path, content).await.map_err(|err| SolcError::io(err, path))?)
    }
}

impl Default for SolFilesCache {
    fn default() -> Self {
        SolFilesCache { format: ETHERS_FORMAT_VERSION.to_string(), files: Default::default() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SolFilesCacheBuilder {
    format: Option<String>,
    solc_config: Option<SolcConfig>,
    root: Option<PathBuf>,
}

impl SolFilesCacheBuilder {
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    #[must_use]
    pub fn solc_config(mut self, solc_config: SolcConfig) -> Self {
        self.solc_config = Some(solc_config);
        self
    }

    #[must_use]
    pub fn root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    /// Creates a new `SolFilesCache` instance
    ///
    /// If a `cache_file` path was provided it's used as base.
    pub fn insert_files(
        self,
        sources: Sources,
        cache_file: Option<PathBuf>,
    ) -> Result<SolFilesCache> {
        let format = self.format.unwrap_or_else(|| ETHERS_FORMAT_VERSION.to_string());
        let solc_config =
            self.solc_config.map(Ok).unwrap_or_else(|| SolcConfig::builder().build())?;

        let root = self
            .root
            .map(Ok)
            .unwrap_or_else(std::env::current_dir)
            .map_err(|err| SolcError::io(err, "."))?;

        let mut files = BTreeMap::new();
        for (file, source) in sources {
            let last_modification_date = fs::metadata(&file)
                .map_err(|err| SolcError::io(err, file.clone()))?
                .modified()
                .map_err(|err| SolcError::io(err, file.clone()))?
                .duration_since(UNIX_EPOCH)
                .map_err(|err| SolcError::solc(err.to_string()))?
                .as_millis() as u64;
            let imports =
                utils::find_import_paths(source.as_ref()).map(|m| m.as_str().to_owned()).collect();

            let version_pragmas = utils::find_version_pragma(source.as_ref())
                .map(|v| vec![v.as_str().to_string()])
                .unwrap_or_default();

            let entry = CacheEntry {
                last_modification_date,
                content_hash: source.content_hash(),
                source_name: utils::source_name(&file, &root).into(),
                solc_config: solc_config.clone(),
                imports,
                version_pragmas,
                artifacts: vec![],
                artifact_paths: BTreeMap::new(),
            };
            files.insert(file, entry);
        }

        let cache = if let Some(dest) = cache_file.as_ref().filter(|dest| dest.exists()) {
            // read the existing cache and extend it by the files that changed
            // (if we just wrote to the cache file, we'd overwrite the existing data)
            let reader =
                std::io::BufReader::new(File::open(dest).map_err(|err| SolcError::io(err, dest))?);
            if let Ok(mut cache) = serde_json::from_reader::<_, SolFilesCache>(reader) {
                cache.files.extend(files);
                cache
            } else {
                tracing::error!("Failed to read existing cache file {}", dest.display());
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
    pub artifact_paths: BTreeMap<PathBuf, String>,
}

impl CacheEntry {
    /// Returns the time
    pub fn last_modified(&self) -> Duration {
        Duration::from_millis(self.last_modification_date)
    }
}

/// A helper type to handle source name/full disk mappings
///
/// The disk path is the actual path where a file can be found on disk.
/// A source name is the internal identifier and is the remaining part of the disk path starting
/// with the configured source directory, (`contracts/contract.sol`)
#[derive(Debug, Default)]
pub struct PathMap {
    /// all libraries to the source set while keeping track of their actual disk path
    /// (`contracts/contract.sol` -> `/Users/.../contracts.sol`)
    pub source_name_to_path: HashMap<PathBuf, PathBuf>,
    /// inverse of `source_name_to_path` : (`/Users/.../contracts.sol` -> `contracts/contract.sol`)
    pub path_to_source_name: HashMap<PathBuf, PathBuf>,
    /* /// All paths, source names and actual file paths
     * paths: Vec<PathBuf> */
}

impl PathMap {
    fn apply_mappings(sources: Sources, mappings: &HashMap<PathBuf, PathBuf>) -> Sources {
        sources
            .into_iter()
            .map(|(import, source)| {
                if let Some(path) = mappings.get(&import).cloned() {
                    (path, source)
                } else {
                    (import, source)
                }
            })
            .collect()
    }

    /// Returns all contract names of the files mapped with the disk path
    pub fn get_artifacts(&self, contracts: &Contracts) -> Vec<(PathBuf, Vec<String>)> {
        contracts
            .iter()
            .map(|(path, contracts)| {
                let path = PathBuf::from(path);
                let file = self.source_name_to_path.get(&path).cloned().unwrap_or(path);
                (file, contracts.keys().cloned().collect::<Vec<_>>())
            })
            .collect()
    }

    pub fn extend(&mut self, other: PathMap) {
        self.source_name_to_path.extend(other.source_name_to_path);
        self.path_to_source_name.extend(other.path_to_source_name);
    }

    /// Returns a new map with the source names as keys
    pub fn set_source_names(&self, sources: Sources) -> Sources {
        Self::apply_mappings(sources, &self.path_to_source_name)
    }

    /// Returns a new map with the disk paths as keys
    pub fn set_disk_paths(&self, sources: Sources) -> Sources {
        Self::apply_mappings(sources, &self.source_name_to_path)
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
