//! Support for compiling contracts
use crate::{
    artifacts::Sources,
    config::{ProjectPaths, SolcConfig},
    error::{Result, SolcError},
    filter::{FilteredSource, FilteredSourceInfo, FilteredSources},
    resolver::GraphEdges,
    utils, ArtifactFile, ArtifactOutput, Artifacts, ArtifactsMap, OutputContext, Project,
    ProjectPathsConfig, Source,
};
use semver::Version;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{
        btree_map::{BTreeMap, Entry},
        hash_map, BTreeSet, HashMap, HashSet,
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
const ETHERS_FORMAT_VERSION: &str = "ethers-rs-sol-cache-3";

/// The file name of the default cache file
pub const SOLIDITY_FILES_CACHE_FILENAME: &str = "solidity-files-cache.json";

/// A multi version cache file
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SolFilesCache {
    #[serde(rename = "_format")]
    pub format: String,
    /// contains all directories used for the project
    pub paths: ProjectPaths,
    pub files: BTreeMap<PathBuf, CacheEntry>,
}

impl SolFilesCache {
    /// Create a new cache instance with the given files
    pub fn new(files: BTreeMap<PathBuf, CacheEntry>, paths: ProjectPaths) -> Self {
        Self { format: ETHERS_FORMAT_VERSION.to_string(), files, paths }
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// How many entries the cache contains where each entry represents a sourc file
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// How many `Artifacts` this cache references, where a source file can have multiple artifacts
    pub fn artifacts_len(&self) -> usize {
        self.entries().map(|entry| entry.artifacts().count()).sum()
    }

    /// Returns an iterator over all `CacheEntry` this cache contains
    pub fn entries(&self) -> impl Iterator<Item = &CacheEntry> {
        self.files.values()
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
    ///
    /// See also [`Self::read_joined()`]
    ///
    /// # Errors
    ///
    /// If the cache file does not exist
    ///
    /// # Example
    ///
    /// ```
    /// # fn t() {
    /// use ethers_solc::cache::SolFilesCache;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let mut cache = SolFilesCache::read(project.cache_path()).unwrap();
    /// cache.join_artifacts_files(project.artifacts_path());
    /// # }
    /// ```
    #[tracing::instrument(skip_all, name = "sol-files-cache::read")]
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        tracing::trace!("reading solfiles cache at {}", path.display());
        let cache: SolFilesCache = utils::read_json_file(path)?;
        tracing::trace!("read cache \"{}\" with {} entries", cache.format, cache.files.len());
        Ok(cache)
    }

    /// Reads the cache json file from the given path and returns the cache with paths adjoined to
    /// the `ProjectPathsConfig`.
    ///
    /// This expects the `artifact` files to be relative to the artifacts dir of the `paths` and the
    /// `CachEntry` paths to be relative to the root dir of the `paths`
    ///
    ///
    ///
    /// # Example
    ///
    /// ```
    /// # fn t() {
    /// use ethers_solc::cache::SolFilesCache;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let cache = SolFilesCache::read_joined(&project.paths).unwrap();
    /// # }
    /// ```
    pub fn read_joined(paths: &ProjectPathsConfig) -> Result<Self> {
        let mut cache = SolFilesCache::read(&paths.cache)?;
        cache.join_entries(&paths.root).join_artifacts_files(&paths.artifacts);
        Ok(cache)
    }

    /// Write the cache as json file to the given path
    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        utils::create_parent_dir_all(path)?;
        let file = fs::File::create(path).map_err(|err| SolcError::io(err, path))?;
        tracing::trace!(
            "writing cache with {} entries to json file: \"{}\"",
            self.len(),
            path.display()
        );
        serde_json::to_writer_pretty(file, self)?;
        tracing::trace!("cache file located: \"{}\"", path.display());
        Ok(())
    }

    /// Sets the `CacheEntry`'s file paths to `root` adjoined to `self.file`.
    pub fn join_entries(&mut self, root: impl AsRef<Path>) -> &mut Self {
        let root = root.as_ref();
        self.files = std::mem::take(&mut self.files)
            .into_iter()
            .map(|(path, entry)| (root.join(path), entry))
            .collect();
        self
    }

    /// Removes `base` from all `CacheEntry` paths
    pub fn strip_entries_prefix(&mut self, base: impl AsRef<Path>) -> &mut Self {
        let base = base.as_ref();
        self.files = std::mem::take(&mut self.files)
            .into_iter()
            .map(|(path, entry)| (path.strip_prefix(base).map(Into::into).unwrap_or(path), entry))
            .collect();
        self
    }

    /// Sets the artifact files location to `base` adjoined to the `CachEntries` artifacts.
    pub fn join_artifacts_files(&mut self, base: impl AsRef<Path>) -> &mut Self {
        let base = base.as_ref();
        self.files.values_mut().for_each(|entry| entry.join_artifacts_files(base));
        self
    }

    /// Removes `base` from all artifact file paths
    pub fn strip_artifact_files_prefixes(&mut self, base: impl AsRef<Path>) -> &mut Self {
        let base = base.as_ref();
        self.files.values_mut().for_each(|entry| entry.strip_artifact_files_prefixes(base));
        self
    }

    /// Removes all `CacheEntry` which source files don't exist on disk
    ///
    /// **NOTE:** this assumes the `files` are absolute
    pub fn remove_missing_files(&mut self) {
        tracing::trace!("remove non existing files from cache");
        self.files.retain(|file, _| {
            let exists = file.exists();
            if !exists {
                tracing::trace!("remove {} from cache", file.display());
            }
            exists
        })
    }

    /// Checks if all artifact files exist
    pub fn all_artifacts_exist(&self) -> bool {
        self.files.values().all(|entry| entry.all_artifacts_exist())
    }

    /// Strips the given prefix from all `file` paths that identify a `CacheEntry` to make them
    /// relative to the given `base` argument
    ///
    /// In other words this sets the keys (the file path of a solidity file) relative to the `base`
    /// argument, so that the key `/Users/me/project/src/Greeter.sol` will be changed to
    /// `src/Greeter.sol` if `base` is `/Users/me/project`
    ///
    /// # Example
    ///
    /// ```
    /// # fn t() {
    /// use ethers_solc::artifacts::contract::CompactContract;
    /// use ethers_solc::cache::SolFilesCache;
    /// use ethers_solc::Project;
    /// let project = Project::builder().build().unwrap();
    /// let cache = SolFilesCache::read(project.cache_path())
    ///     .unwrap()
    ///     .with_stripped_file_prefixes(project.root());
    /// let artifact: CompactContract = cache.read_artifact("src/Greeter.sol", "Greeter").unwrap();
    /// # }
    /// ```
    ///
    /// **Note:** this only affects the source files, see [`Self::strip_artifact_files_prefixes()`]
    pub fn with_stripped_file_prefixes(mut self, base: impl AsRef<Path>) -> Self {
        let base = base.as_ref();
        self.files = self
            .files
            .into_iter()
            .map(|(f, e)| (utils::source_name(&f, base).to_path_buf(), e))
            .collect();
        self
    }

    /// Returns the path to the artifact of the given `(file, contract)` pair
    ///
    /// # Example
    ///
    /// ```
    /// # fn t() {
    /// use ethers_solc::cache::SolFilesCache;
    /// use ethers_solc::Project;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let cache = SolFilesCache::read_joined(&project.paths).unwrap();
    /// cache.find_artifact_path("/Users/git/myproject/src/Greeter.sol", "Greeter");
    /// # }
    /// ```
    pub fn find_artifact_path(
        &self,
        contract_file: impl AsRef<Path>,
        contract_name: impl AsRef<str>,
    ) -> Option<&PathBuf> {
        let entry = self.entry(contract_file)?;
        entry.find_artifact_path(contract_name)
    }

    /// Finds the path to the artifact of the given `(file, contract)` pair, see
    /// [`Self::find_artifact_path()`], and reads the artifact as json file
    /// # Example
    ///
    /// ```
    /// fn t() {
    /// use ethers_solc::cache::SolFilesCache;
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::contract::CompactContract;
    ///
    /// let project = Project::builder().build().unwrap();
    /// let cache = SolFilesCache::read_joined(&project.paths).unwrap();
    /// let artifact: CompactContract = cache.read_artifact("/Users/git/myproject/src/Greeter.sol", "Greeter").unwrap();
    /// # }
    /// ```
    ///
    /// **NOTE**: unless the cache's `files` keys were modified `contract_file` is expected to be
    /// absolute, see [``]
    pub fn read_artifact<Artifact: DeserializeOwned>(
        &self,
        contract_file: impl AsRef<Path>,
        contract_name: impl AsRef<str>,
    ) -> Result<Artifact> {
        let contract_file = contract_file.as_ref();
        let contract_name = contract_name.as_ref();

        let artifact_path =
            self.find_artifact_path(contract_file, contract_name).ok_or_else(|| {
                SolcError::ArtifactNotFound(contract_file.to_path_buf(), contract_name.to_string())
            })?;

        utils::read_json_file(artifact_path)
    }

    /// Reads all cached artifacts from disk using the given ArtifactOutput handler
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::cache::SolFilesCache;
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::contract::CompactContractBytecode;
    /// # fn t() {
    /// let project = Project::builder().build().unwrap();
    /// let cache = SolFilesCache::read_joined(&project.paths).unwrap();
    /// let artifacts = cache.read_artifacts::<CompactContractBytecode>().unwrap();
    /// # }
    /// ```
    pub fn read_artifacts<Artifact: DeserializeOwned + Send + Sync>(
        &self,
    ) -> Result<Artifacts<Artifact>> {
        use rayon::prelude::*;

        let artifacts = self
            .files
            .par_iter()
            .map(|(file, entry)| {
                let file_name = format!("{}", file.display());
                entry.read_artifact_files().map(|files| (file_name, files))
            })
            .collect::<Result<ArtifactsMap<_>>>()?;
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
        let mut files: HashMap<_, _> = files.into_iter().collect();

        self.files.retain(|file, entry| {
            if entry.artifacts.is_empty() {
                // keep entries that didn't emit any artifacts in the first place, such as a
                // solidity file that only includes error definitions
                return true
            }

            if let Some(versions) = files.remove(file.as_path()) {
                entry.retain_versions(versions);
            } else {
                return false
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
        SolFilesCache {
            format: ETHERS_FORMAT_VERSION.to_string(),
            files: Default::default(),
            paths: Default::default(),
        }
    }
}

impl<'a> From<&'a ProjectPathsConfig> for SolFilesCache {
    fn from(config: &'a ProjectPathsConfig) -> Self {
        let paths = config.paths_relative();
        SolFilesCache::new(Default::default(), paths)
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
    /// identifier name see [`crate::utils::source_name()`]
    pub source_name: PathBuf,
    /// what config was set when compiling this file
    pub solc_config: SolcConfig,
    /// fully resolved imports of the file
    ///
    /// all paths start relative from the project's root: `src/importedFile.sol`
    pub imports: BTreeSet<PathBuf>,
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

    /// Returns the artifact path for the contract name
    /// ```
    /// use ethers_solc::cache::CacheEntry;
    /// # fn t(entry: CacheEntry) {
    /// entry.find_artifact_path("Greeter");
    /// # }
    /// ```
    pub fn find_artifact_path(&self, contract_name: impl AsRef<str>) -> Option<&PathBuf> {
        self.artifacts.get(contract_name.as_ref())?.iter().next().map(|(_, p)| p)
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
    /// **Note:** all artifact file paths should be absolute.
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
                    entry.get_mut().extend(artifacts);
                }
            }
        }
    }

    /// Retains only those artifacts that match the provided versions.
    ///
    /// Removes an artifact entry if none of its versions is included in the `versions` set.
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

    /// Returns the artifact file for the contract and version pair
    pub fn find_artifact(&self, contract: &str, version: &Version) -> Option<&PathBuf> {
        self.artifacts.get(contract).and_then(|files| files.get(version))
    }

    /// Iterator that yields all artifact files and their version
    pub fn artifacts_for_version<'a>(
        &'a self,
        version: &'a Version,
    ) -> impl Iterator<Item = &'a PathBuf> + 'a {
        self.artifacts_versions().filter_map(move |(ver, file)| (ver == version).then_some(file))
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
    pub fn join_artifacts_files(&mut self, base: impl AsRef<Path>) {
        let base = base.as_ref();
        self.artifacts_mut().for_each(|p| *p = base.join(&*p))
    }

    /// Removes `base` from the artifact's path
    pub fn strip_artifact_files_prefixes(&mut self, base: impl AsRef<Path>) {
        let base = base.as_ref();
        self.artifacts_mut().for_each(|p| {
            if let Ok(rem) = p.strip_prefix(base) {
                *p = rem.to_path_buf();
            }
        })
    }
}

/// A helper abstraction over the [`SolFilesCache`] used to determine what files need to compiled
/// and which `Artifacts` can be reused.
#[derive(Debug)]
pub(crate) struct ArtifactsCacheInner<'a, T: ArtifactOutput> {
    /// The preexisting cache file.
    pub cache: SolFilesCache,

    /// All already existing artifacts.
    pub cached_artifacts: Artifacts<T::Artifact>,

    /// Relationship between all the files.
    pub edges: GraphEdges,

    /// The project.
    pub project: &'a Project<T>,

    /// All the files that were filtered because they haven't changed.
    pub filtered: HashMap<PathBuf, (Source, HashSet<Version>)>,

    /// The corresponding cache entries for all sources that were deemed to be dirty.
    ///
    /// `CacheEntry` are grouped by their Solidity file.
    /// During preprocessing the `artifacts` field of a new `CacheEntry` is left blank, because in
    /// order to determine the artifacts of the solidity file, the file needs to be compiled first.
    /// Only after the `CompilerOutput` is received and all compiled contracts are handled, see
    /// [`crate::ArtifactOutput::on_output`] all artifacts, their disk paths, are determined and
    /// can be populated before the updated [`crate::SolFilesCache`] is finally written to disk.
    pub dirty_source_files: HashMap<PathBuf, (CacheEntry, HashSet<Version>)>,

    /// The file hashes.
    pub content_hashes: HashMap<PathBuf, String>,
}

impl<'a, T: ArtifactOutput> ArtifactsCacheInner<'a, T> {
    /// Creates a new cache entry for the file
    fn create_cache_entry(&self, file: &Path, source: &Source) -> CacheEntry {
        let imports = self
            .edges
            .imports(file)
            .into_iter()
            .map(|import| utils::source_name(import, self.project.root()).to_path_buf())
            .collect();

        let entry = CacheEntry {
            last_modification_date: CacheEntry::read_last_modification_date(file)
                .unwrap_or_default(),
            content_hash: source.content_hash(),
            source_name: utils::source_name(file, self.project.root()).into(),
            solc_config: self.project.solc_config.clone(),
            imports,
            version_requirement: self.edges.version_requirement(file).map(|v| v.to_string()),
            // artifacts remain empty until we received the compiler output
            artifacts: Default::default(),
        };

        entry
    }

    /// inserts a new cache entry for the given file
    ///
    /// If there is already an entry available for the file the given version is added to the set
    fn insert_new_cache_entry(&mut self, file: &Path, source: &Source, version: Version) {
        if let Some((_, versions)) = self.dirty_source_files.get_mut(file) {
            versions.insert(version);
        } else {
            let entry = self.create_cache_entry(file, source);
            self.dirty_source_files.insert(file.to_path_buf(), (entry, HashSet::from([version])));
        }
    }

    /// inserts the filtered source with the given version
    fn insert_filtered_source(&mut self, file: PathBuf, source: Source, version: Version) {
        match self.filtered.entry(file) {
            hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().1.insert(version);
            }
            hash_map::Entry::Vacant(entry) => {
                entry.insert((source, HashSet::from([version])));
            }
        }
    }

    /// Returns the set of [Source]s that need to be included in the `CompilerOutput` in order to
    /// recompile the project.
    ///
    /// We define _dirty_ sources as files that:
    ///   - are new
    ///   - were changed
    ///   - their imports were changed
    ///   - their artifact is missing
    ///
    /// A _dirty_ file is always included in the `CompilerInput`.
    /// A _dirty_ file can also include clean files - files that do not match any of the above
    /// criteria - which solc also requires in order to compile a dirty file.
    ///
    /// Therefore, these files will also be included in the filtered output but not marked as dirty,
    /// so that their `OutputSelection` can be optimized in the `CompilerOutput` and their (empty)
    /// artifacts ignored.
    fn filter(&mut self, sources: Sources, version: &Version) -> FilteredSources {
        // all files that are not dirty themselves, but are pulled from a dirty file
        let mut imports_of_dirty = HashSet::new();

        // separates all source files that fit the criteria (dirty) from those that don't (clean)
        let (mut filtered_sources, clean_sources) = sources
            .into_iter()
            .map(|(file, source)| self.filter_source(file, source, version))
            .fold(
                (BTreeMap::default(), Vec::new()),
                |(mut dirty_sources, mut clean_sources), source| {
                    if source.dirty {
                        // mark all files that are imported by a dirty file
                        imports_of_dirty.extend(self.edges.all_imported_nodes(source.idx));
                        dirty_sources.insert(source.file, FilteredSource::Dirty(source.source));
                    } else {
                        clean_sources.push(source);
                    }

                    (dirty_sources, clean_sources)
                },
            );

        // track new cache entries for dirty files
        for (file, filtered) in filtered_sources.iter() {
            self.insert_new_cache_entry(file, filtered.source(), version.clone());
        }

        for clean_source in clean_sources {
            let FilteredSourceInfo { file, source, idx, .. } = clean_source;
            if imports_of_dirty.contains(&idx) {
                // file is pulled in by a dirty file
                filtered_sources.insert(file.clone(), FilteredSource::Clean(source.clone()));
            }
            self.insert_filtered_source(file, source, version.clone());
        }

        filtered_sources.into()
    }

    /// Returns the state of the given source file.
    fn filter_source(
        &self,
        file: PathBuf,
        source: Source,
        version: &Version,
    ) -> FilteredSourceInfo {
        let idx = self.edges.node_id(&file);
        if !self.is_dirty(&file, version) &&
            self.edges.imports(&file).iter().all(|file| !self.is_dirty(file, version))
        {
            FilteredSourceInfo { file, source, idx, dirty: false }
        } else {
            FilteredSourceInfo { file, source, idx, dirty: true }
        }
    }

    /// returns `false` if the corresponding cache entry remained unchanged otherwise `true`
    fn is_dirty(&self, file: &Path, version: &Version) -> bool {
        if let Some(hash) = self.content_hashes.get(file) {
            if let Some(entry) = self.cache.entry(file) {
                if entry.content_hash.as_bytes() != hash.as_bytes() {
                    tracing::trace!("changed content hash for source file \"{}\"", file.display());
                    return true
                }
                if self.project.solc_config != entry.solc_config {
                    tracing::trace!("changed solc config for source file \"{}\"", file.display());
                    return true
                }

                // only check artifact's existence if the file generated artifacts.
                // e.g. a solidity file consisting only of import statements (like interfaces that
                // re-export) do not create artifacts
                if !entry.artifacts.is_empty() {
                    if !entry.contains_version(version) {
                        tracing::trace!(
                            "missing linked artifacts for source file `{}` for version \"{}\"",
                            file.display(),
                            version
                        );
                        return true
                    }

                    if entry.artifacts_for_version(version).any(|artifact_path| {
                        let missing_artifact = !self.cached_artifacts.has_artifact(artifact_path);
                        if missing_artifact {
                            tracing::trace!("missing artifact \"{}\"", artifact_path.display());
                        }
                        missing_artifact
                    }) {
                        return true
                    }
                }
                // all things match, can be reused
                return false
            }
            tracing::trace!("Missing cache entry for {}", file.display());
        } else {
            tracing::trace!("Missing content hash for {}", file.display());
        }
        true
    }

    /// Adds the file's hashes to the set if not set yet
    fn fill_hashes(&mut self, sources: &Sources) {
        for (file, source) in sources {
            if let hash_map::Entry::Vacant(entry) = self.content_hashes.entry(file.clone()) {
                entry.insert(source.content_hash());
            }
        }
    }
}

/// Abstraction over configured caching which can be either non-existent or an already loaded cache
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub(crate) enum ArtifactsCache<'a, T: ArtifactOutput> {
    /// Cache nothing on disk
    Ephemeral(GraphEdges, &'a Project<T>),
    /// Handles the actual cached artifacts, detects artifacts that can be reused
    Cached(ArtifactsCacheInner<'a, T>),
}

impl<'a, T: ArtifactOutput> ArtifactsCache<'a, T> {
    pub fn new(project: &'a Project<T>, edges: GraphEdges) -> Result<Self> {
        /// Returns the [SolFilesCache] to use
        ///
        /// Returns a new empty cache if the cache does not exist or `invalidate_cache` is set.
        fn get_cache<T: ArtifactOutput>(
            project: &Project<T>,
            invalidate_cache: bool,
        ) -> SolFilesCache {
            // the currently configured paths
            let paths = project.paths.paths_relative();

            if !invalidate_cache && project.cache_path().exists() {
                if let Ok(cache) = SolFilesCache::read_joined(&project.paths) {
                    if cache.paths == paths {
                        // unchanged project paths
                        return cache
                    }
                }
            }

            // new empty cache
            SolFilesCache::new(Default::default(), paths)
        }

        let cache = if project.cached {
            // we only read the existing cache if we were able to resolve the entire graph
            // if we failed to resolve an import we invalidate the cache so don't get any false
            // positives
            let invalidate_cache = !edges.unresolved_imports().is_empty();

            // read the cache file if it already exists
            let mut cache = get_cache(project, invalidate_cache);

            cache.remove_missing_files();

            // read all artifacts
            let cached_artifacts = if project.paths.artifacts.exists() {
                tracing::trace!("reading artifacts from cache...");
                // if we failed to read the whole set of artifacts we use an empty set
                let artifacts = cache.read_artifacts::<T::Artifact>().unwrap_or_default();
                tracing::trace!("read {} artifacts from cache", artifacts.artifact_files().count());
                artifacts
            } else {
                Default::default()
            };

            let cache = ArtifactsCacheInner {
                cache,
                cached_artifacts,
                edges,
                project,
                filtered: Default::default(),
                dirty_source_files: Default::default(),
                content_hashes: Default::default(),
            };

            ArtifactsCache::Cached(cache)
        } else {
            // nothing to cache
            ArtifactsCache::Ephemeral(edges, project)
        };

        Ok(cache)
    }

    /// Returns the graph data for this project
    pub fn graph(&self) -> &GraphEdges {
        match self {
            ArtifactsCache::Ephemeral(graph, _) => graph,
            ArtifactsCache::Cached(inner) => &inner.edges,
        }
    }

    #[cfg(test)]
    #[allow(unused)]
    #[doc(hidden)]
    // only useful for debugging for debugging purposes
    pub fn as_cached(&self) -> Option<&ArtifactsCacheInner<'a, T>> {
        match self {
            ArtifactsCache::Ephemeral(_, _) => None,
            ArtifactsCache::Cached(cached) => Some(cached),
        }
    }

    pub fn output_ctx(&self) -> OutputContext {
        match self {
            ArtifactsCache::Ephemeral(_, _) => Default::default(),
            ArtifactsCache::Cached(inner) => OutputContext::new(&inner.cache),
        }
    }

    pub fn project(&self) -> &'a Project<T> {
        match self {
            ArtifactsCache::Ephemeral(_, project) => project,
            ArtifactsCache::Cached(cache) => cache.project,
        }
    }

    /// Adds the file's hashes to the set if not set yet
    pub fn fill_content_hashes(&mut self, sources: &Sources) {
        match self {
            ArtifactsCache::Ephemeral(_, _) => {}
            ArtifactsCache::Cached(cache) => cache.fill_hashes(sources),
        }
    }

    /// Filters out those sources that don't need to be compiled
    pub fn filter(&mut self, sources: Sources, version: &Version) -> FilteredSources {
        match self {
            ArtifactsCache::Ephemeral(_, _) => sources.into(),
            ArtifactsCache::Cached(cache) => cache.filter(sources, version),
        }
    }

    /// Consumes the `Cache`, rebuilds the `SolFileCache` by merging all artifacts that were
    /// filtered out in the previous step (`Cache::filtered`) and the artifacts that were just
    /// compiled and written to disk `written_artifacts`.
    ///
    /// Returns all the _cached_ artifacts.
    pub fn consume(
        self,
        written_artifacts: &Artifacts<T::Artifact>,
        write_to_disk: bool,
    ) -> Result<Artifacts<T::Artifact>> {
        match self {
            ArtifactsCache::Ephemeral(_, _) => {
                tracing::trace!("no cache configured, ephemeral");
                Ok(Default::default())
            }
            ArtifactsCache::Cached(cache) => {
                let ArtifactsCacheInner {
                    mut cache,
                    mut cached_artifacts,
                    mut dirty_source_files,
                    filtered,
                    project,
                    ..
                } = cache;

                // keep only those files that were previously filtered (not dirty, reused)
                cache.retain(filtered.iter().map(|(p, (_, v))| (p.as_path(), v)));

                // add the written artifacts to the cache entries, this way we can keep a mapping
                // from solidity file to its artifacts
                // this step is necessary because the concrete artifacts are only known after solc
                // was invoked and received as output, before that we merely know the file and
                // the versions, so we add the artifacts on a file by file basis
                for (file, written_artifacts) in written_artifacts.as_ref() {
                    let file_path = Path::new(&file);
                    if let Some((cache_entry, versions)) = dirty_source_files.get_mut(file_path) {
                        cache_entry.insert_artifacts(written_artifacts.iter().map(
                            |(name, artifacts)| {
                                let artifacts = artifacts
                                    .iter()
                                    .filter(|artifact| versions.contains(&artifact.version))
                                    .collect::<Vec<_>>();
                                (name, artifacts)
                            },
                        ));
                    }

                    // cached artifacts that were overwritten also need to be removed from the
                    // `cached_artifacts` set
                    if let Some((f, mut cached)) = cached_artifacts.0.remove_entry(file) {
                        tracing::trace!("checking {} for obsolete cached artifact entries", file);
                        cached.retain(|name, cached_artifacts| {
                            if let Some(written_files) = written_artifacts.get(name) {
                                // written artifact clashes with a cached artifact, so we need to decide whether to keep or to remove the cached
                                cached_artifacts.retain(|f| {
                                    // we only keep those artifacts that don't conflict with written artifacts and which version was a compiler target
                                    let retain = written_files
                                        .iter()
                                        .all(|other| other.version != f.version) && filtered.get(
                                        &PathBuf::from(file)).map(|(_, versions)| {
                                            versions.contains(&f.version)
                                        }).unwrap_or_default();
                                    if !retain {
                                        tracing::trace!(
                                            "purging obsolete cached artifact {:?} for contract {} and version {}",
                                            f.file,
                                            name,
                                            f.version
                                        );
                                    }
                                    retain
                                });
                                return !cached_artifacts.is_empty()
                            }
                            false
                        });

                        if !cached.is_empty() {
                            cached_artifacts.0.insert(f, cached);
                        }
                    }
                }

                // add the new cache entries to the cache file
                cache
                    .extend(dirty_source_files.into_iter().map(|(file, (entry, _))| (file, entry)));

                // write to disk
                if write_to_disk {
                    // make all `CacheEntry` paths relative to the project root and all artifact
                    // paths relative to the artifact's directory
                    cache
                        .strip_entries_prefix(project.root())
                        .strip_artifact_files_prefixes(project.artifacts_path());
                    cache.write(project.cache_path())?;
                }

                Ok(cached_artifacts)
            }
        }
    }
}
