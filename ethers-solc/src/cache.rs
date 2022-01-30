//! Support for compiling contracts
use crate::{
    artifacts::{Contracts, Sources},
    config::SolcConfig,
    error::{Result, SolcError},
    utils, ArtifactFile, Artifacts, ArtifactsMap,
};
use semver::Version;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{
        btree_map::{BTreeMap, Entry},
        HashMap, HashSet,
    },
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
        let cache: SolFilesCache = utils::read_json_file(path)?;
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
    pub fn read_artifacts<Artifact: DeserializeOwned>(&self) -> Result<Artifacts<Artifact>> {
        let mut artifacts = ArtifactsMap::new();
        for (file, entry) in self.files.iter() {
            let file_name = format!("{}", file.display());
            artifacts.insert(file_name, entry.read_artifact_files()?);
        }
        Ok(Artifacts(artifacts))
    }

    /// Retains only the `CacheEntry` specified by the file + version combination.
    ///
    /// In other words, only keep those cache entries with the paths (keys) that the iterator yields
    /// and only keep the versions in the cache entry that the version iterator yields.
    pub fn retain<'a, I, V>(&mut self, files: I)
    where
        I: IntoIterator<Item = (&'a Path, V)>,
        V: IntoIterator<Item = &'a Version>,
    {
        let mut files: HashMap<_, _> = files.into_iter().map(|(p, v)| (p, v)).collect();

        self.files.retain(|file, entry| {
            if let Some(versions) = files.remove(file.as_path()) {
                entry.retain_versions(versions);
            }
            !entry.artifacts.is_empty()
        });
    }

    /// Inserts the provided cache entries, if there is an existing `CacheEntry` it will be updated
    /// but versions will be merged.
    pub fn extend<I>(&mut self, entries: I)
    where
        I: IntoIterator<Item = (PathBuf, CacheEntry)>,
    {
        for (file, entry) in entries.into_iter() {
            match self.files.entry(file) {
                Entry::Vacant(e) => {
                    e.insert(entry);
                }
                Entry::Occupied(mut other) => {
                    other.get_mut().merge_artifacts(entry);
                }
            }
        }
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
        tokio::fs::write(path, content).await.map_err(|err| SolcError::io(err, path))
    }
}

impl Default for SolFilesCache {
    fn default() -> Self {
        SolFilesCache { format: ETHERS_FORMAT_VERSION.to_string(), files: Default::default() }
    }
}

/// A `CacheEntry` in the cache file represents a solidity file
///
/// A solidity file can contain several contracts, for every contract a separate `Artifact` is
/// emitted. so the `CacheEntry` tracks the artifacts by name. A file can be compiled with multiple
/// `solc` versions generating version specific artifacts.
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
    /// artifacts.
    ///
    /// This map tracks the artifacts by `name -> (Version -> PathBuf)`.
    /// This mimics the default artifacts directory structure
    pub artifacts: BTreeMap<String, BTreeMap<Version, PathBuf>>,
}

impl CacheEntry {
    /// Returns the last modified timestamp `Duration`
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

    /// Reads all artifact files associated with the `CacheEntry`
    ///
    /// **Note:** all artifact file paths should be absolute, see [`Self::join`]
    fn read_artifact_files<Artifact: DeserializeOwned>(
        &self,
    ) -> Result<BTreeMap<String, Vec<ArtifactFile<Artifact>>>> {
        let mut artifacts = BTreeMap::new();
        for (artifact_name, versioned_files) in self.artifacts.iter() {
            let mut files = Vec::with_capacity(versioned_files.len());
            for (version, file) in versioned_files {
                let artifact: Artifact = utils::read_json_file(file)?;
                files.push(ArtifactFile { artifact, file: file.clone(), version: version.clone() });
            }
            artifacts.insert(artifact_name.clone(), files);
        }
        Ok(artifacts)
    }

    pub(crate) fn insert_artifacts<'a, I, T: 'a>(&mut self, artifacts: I)
    where
        I: IntoIterator<Item = (&'a String, Vec<&'a ArtifactFile<T>>)>,
    {
        for (name, artifacts) in artifacts.into_iter().filter(|(_, a)| !a.is_empty()) {
            let entries: BTreeMap<_, _> = artifacts
                .into_iter()
                .map(|artifact| (artifact.version.clone(), artifact.file.clone()))
                .collect();
            self.artifacts.insert(name.clone(), entries);
        }
    }

    /// Merges another `CacheEntries` artifacts into the existing set
    fn merge_artifacts(&mut self, other: CacheEntry) {
        for (name, artifacts) in other.artifacts {
            match self.artifacts.entry(name) {
                Entry::Vacant(entry) => {
                    entry.insert(artifacts);
                }
                Entry::Occupied(mut entry) => {
                    entry.get_mut().extend(artifacts.into_iter());
                }
            }
        }
    }

    /// Retains only those artifacts that match the provided version.
    pub fn retain_versions<'a, I>(&mut self, versions: I)
    where
        I: IntoIterator<Item = &'a Version>,
    {
        let versions = versions.into_iter().collect::<HashSet<_>>();
        self.artifacts.retain(|_, artifacts| {
            artifacts.retain(|version, _| versions.contains(version));
            !artifacts.is_empty()
        })
    }

    /// Returns `true` if the artifacts set contains the given version
    pub fn contains_version(&self, version: &Version) -> bool {
        self.artifacts_versions().any(|(v, _)| v == version)
    }

    /// Iterator that yields all artifact files and their version
    pub fn artifacts_versions(&self) -> impl Iterator<Item = (&Version, &PathBuf)> {
        self.artifacts.values().flat_map(|artifacts| artifacts.iter())
    }

    /// Iterator that yields all artifact files and their version
    pub fn artifacts_for_version<'a>(
        &'a self,
        version: &'a Version,
    ) -> impl Iterator<Item = &'a PathBuf> + 'a {
        self.artifacts_versions().filter_map(move |(ver, file)| (ver == version).then(|| file))
    }

    /// Iterator that yields all artifact files
    pub fn artifacts(&self) -> impl Iterator<Item = &PathBuf> {
        self.artifacts.values().flat_map(|artifacts| artifacts.values())
    }

    /// Mutable iterator over all artifact files
    pub fn artifacts_mut(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.artifacts.values_mut().flat_map(|artifacts| artifacts.values_mut())
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
