//! Support for compiling contracts
use crate::{
    artifacts::{Contracts, Sources},
    config::SolcConfig,
    error::{Result, SolcError},
    ArtifactFile, ArtifactOutput, Artifacts, ArtifactsMap, Source,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::BTreeMap, HashMap},
    fs::{self},
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

/// ethers-rs format version
///
/// `ethers-solc` uses a different format version id, but the actual format is consistent with
/// hardhat This allows ethers-solc to detect if the cache file was written by hardhat or
/// `ethers-solc`
const ETHERS_FORMAT_VERSION: &str = "ethers-rs-sol-cache-1";

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
    /// Create a new cache instance with the given files
    pub fn new(files: BTreeMap<PathBuf, CacheEntry>) -> Self {
        Self { format: ETHERS_FORMAT_VERSION.to_string(), files }
    }

    /// Returns the corresponding `CacheEntry` for the file if it exists
    pub fn entry(&self, file: impl AsRef<Path>) -> Option<&CacheEntry> {
        self.files.get(file.as_ref())
    }

    /// Returns the corresponding `CacheEntry` for the file if it exists
    pub fn entry_mut(&mut self, file: impl AsRef<Path>) -> Option<&mut CacheEntry> {
        self.files.get_mut(file.as_ref())
    }

    /// Reads the cache json file from the given path
    /// # Example
    ///
    /// ```
    /// use ethers_solc::cache::SolFilesCache;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let mut cache = SolFilesCache::read(project.cache_path()).unwrap();
    /// cache.join_all(project.artifacts_path());
    /// ```
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

    /// Sets the artifact files location to `base` adjoined to the `CachEntries` artifacts.
    pub fn join_all(&mut self, base: impl AsRef<Path>) -> &mut Self {
        let base = base.as_ref();
        self.files.values_mut().for_each(|entry| entry.join(base));
        self
    }

    /// Removes `base` from all artifact file paths
    pub fn strip_prefix_all(&mut self, base: impl AsRef<Path>) -> &mut Self {
        let base = base.as_ref();
        self.files.values_mut().for_each(|entry| entry.strip_prefix(base));
        self
    }

    /// Removes all `CacheEntry` which source files are missing
    pub fn remove_missing_files(&mut self) {
        tracing::trace!("remove non existing files from cache");
        self.files.retain(|file, _| file.exists())
    }

    /// Checks if all artifact files exist
    pub fn all_artifacts_exist(&self) -> bool {
        self.files.values().all(|entry| entry.all_artifacts_exist())
    }

    /// Reads all cached artifacts from disk using the given ArtifactOutput handler
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::cache::SolFilesCache;
    /// use ethers_solc::{MinimalCombinedArtifacts, Project};
    ///
    /// let project = Project::builder().build().unwrap();
    /// let mut cache = SolFilesCache::read(project.cache_path()).unwrap();
    /// cache.join_all(project.artifacts_path());
    /// let artifacts = cache.read_artifacts::<MinimalCombinedArtifacts>().unwrap();
    /// ```
    pub fn read_artifacts<Artifact: Serialize>(&self) -> Result<Artifacts<Artifact>> {
        let mut artifacts = ArtifactsMap::new();
        for (file, entry) in self.files.iter() {
            // let mut entries = BTreeMap::new();
        }

        // let mut artifacts = BTreeMap::default();
        // for (file, entry) in &self.files {
        //     for artifact in &entry.artifacts {
        //         let artifact_file = artifacts_root.join(T::output_file(file, artifact));
        //         let artifact = T::read_cached_artifact(&artifact_file)?;
        //         artifacts.insert(artifact_file, artifact);
        //     }
        // }
        Ok(Artifacts(artifacts))
    }

    /// Retains only the `CacheEntry` specified by the file + version combination.
    ///
    /// In other words, only keep those cache entries with the paths (keys) that the iterator yields
    /// and only keep the versions in the cache entry that the version iterator yields.
    pub fn retain<'a, I, V>(&mut self, _files: I)
    where
        I: IntoIterator<Item = (&'a Path, V)>,
        V: IntoIterator<Item = &'a Version>,
    {
    }

    /// Inserts the provided cache entries, if there is an existing `CacheEntry` it will be updated
    /// but versions will be merged.
    pub fn extend<I, V>(&mut self, _entries: I)
    where
        I: IntoIterator<Item = (PathBuf, CacheEntry)>,
    {
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheEntry {
    /// the last modification time of this file
    pub last_modification_date: u64,
    /// hash to identify whether the content of the file changed
    pub content_hash: String,
    /// identifier name see [`crate::util::source_name()`]
    pub source_name: PathBuf,
    /// what config was set when compiling this file
    pub solc_config: SolcConfig,
    /// fully resolved imports of the file
    ///
    /// all paths start relative from the project's root: `src/importedFile.sol`
    pub imports: Vec<PathBuf>,
    /// The solidity version pragma
    pub version_requirement: Option<String>,
    /// all artifacts produced for this file
    ///
    /// In theory a file can be compiled by different solc versions:
    /// `A(<=0.8.10) imports C(>0.4.0)` and `B(0.8.11) imports C(>0.4.0)`
    /// file `C` would be compiled twice, with `0.8.10` and `0.8.11`, producing two different
    /// artifacts
    pub artifacts: BTreeMap<Version, Vec<PathBuf>>,
}

impl CacheEntry {
    pub fn new(_file: impl AsRef<Path>, _source: &Source) -> Result<Self> {
        todo!()
    }

    /// Returns the time
    pub fn last_modified(&self) -> Duration {
        Duration::from_millis(self.last_modification_date)
    }

    /// Reads the last modification date from the file's metadata
    pub fn read_last_modification_date(file: impl AsRef<Path>) -> Result<u64> {
        let file = file.as_ref();
        let last_modification_date = fs::metadata(file)
            .map_err(|err| SolcError::io(err, file.to_path_buf()))?
            .modified()
            .map_err(|err| SolcError::io(err, file.to_path_buf()))?
            .duration_since(UNIX_EPOCH)
            .map_err(|err| SolcError::solc(err.to_string()))?
            .as_millis() as u64;
        Ok(last_modification_date)
    }

    fn read_artifact_files<T: ArtifactOutput>(&self) -> Result<Vec<ArtifactFile<T::Artifact>>> {
        for (version, files) in self.artifacts.iter() {
            for file in files {
                // get the contract name based on the number of versions
            }
        }

        todo!()
    }

    /// Iterator that yields all artifact files
    pub fn artifacts(&self) -> impl Iterator<Item = &PathBuf> {
        self.artifacts.values().flat_map(|artifacts| artifacts.into_iter())
    }

    pub fn artifacts_mut(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.artifacts.values_mut().flat_map(|artifacts| artifacts.into_iter())
    }

    /// Checks if all artifact files exist
    pub fn all_artifacts_exist(&self) -> bool {
        self.artifacts().all(|p| p.exists())
    }

    /// Sets the artifact's paths to `base` adjoined to the artifact's `path`.
    pub fn join(&mut self, base: impl AsRef<Path>) {
        let base = base.as_ref();
        self.artifacts_mut().for_each(|p| *p = p.join(base))
    }

    /// Removes `base` from the artifact's path
    pub fn strip_prefix(&mut self, base: impl AsRef<Path>) {
        let base = base.as_ref();
        self.artifacts_mut().for_each(|p| {
            if let Ok(prefix) = p.strip_prefix(base) {
                *p = prefix.to_path_buf();
            }
        })
    }
}

/// A helper type to handle source name/full disk mappings
///
/// The disk path is the actual path where a file can be found on disk.
/// A source name is the internal identifier and is the remaining part of the disk path starting
/// with the configured source directory, (`contracts/contract.sol`)
///
/// See also [Import Path Resolution](https://docs.soliditylang.org/en/develop/path-resolution.html#path-resolution)
#[derive(Debug, Default)]
pub struct SourceUnitNameMap {
    /// all libraries to the source set while keeping track of their actual disk path
    /// (`contracts/contract.sol` -> `/Users/.../contracts.sol`)
    pub source_unit_name_to_path: HashMap<PathBuf, PathBuf>,
    /// inverse of `source_name_to_path` : (`/Users/.../contracts.sol` -> `contracts/contract.sol`)
    pub path_to_source_unit_name: HashMap<PathBuf, PathBuf>,
}

impl SourceUnitNameMap {
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
                let file = self.source_unit_name_to_path.get(&path).cloned().unwrap_or(path);
                (file, contracts.keys().cloned().collect::<Vec<_>>())
            })
            .collect()
    }

    pub fn extend(&mut self, other: SourceUnitNameMap) {
        self.source_unit_name_to_path.extend(other.source_unit_name_to_path);
        self.path_to_source_unit_name.extend(other.path_to_source_unit_name);
    }

    /// Returns a new map with the source names as keys
    pub fn set_source_names(&self, sources: Sources) -> Sources {
        Self::apply_mappings(sources, &self.path_to_source_unit_name)
    }

    /// Returns a new map with the disk paths as keys
    pub fn set_disk_paths(&self, sources: Sources) -> Sources {
        Self::apply_mappings(sources, &self.source_unit_name_to_path)
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
