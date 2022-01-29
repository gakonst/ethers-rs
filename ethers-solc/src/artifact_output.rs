//! Output artifact handling

use crate::{
    artifacts::{CompactContract, CompactContractBytecode, Contract, FileToContractsMap},
    contracts::VersionedContracts,
    error::Result,
    HardhatArtifact, ProjectPathsConfig, SolcError,
};
use ethers_core::{abi::Abi, types::Bytes};
use semver::Version;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::btree_map::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ArtifactInfo {
    /// path to the file where the `artifact` was written to
    pub file: PathBuf,
    /// `solc` version that produced this artifact
    pub version: Version,
}

/// Represents an artifact file representing a [`crate::Contract`]
#[derive(Debug, Clone, PartialEq)]
pub struct ArtifactFile<T> {
    /// The Artifact that was written
    pub artifact: T,
    /// path to the file where the `artifact` was written to
    pub file: PathBuf,
    /// `solc` version that produced this artifact
    pub version: Version,
}

impl<T: Serialize> ArtifactFile<T> {
    /// Writes the given contract to the `out` path creating all parent directories
    pub fn write(&self) -> Result<()> {
        if let Some(parent) = self.file.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                SolcError::msg(format!(
                    "Failed to create artifact parent folder \"{}\": {}",
                    parent.display(),
                    err
                ))
            })?;
        }
        fs::write(&self.file, serde_json::to_vec_pretty(&self.artifact)?)
            .map_err(|err| SolcError::io(err, &self.file))?;
        Ok(())
    }
}

impl<T> ArtifactFile<T> {
    /// Sets the file to `root` adjoined to `self.file`.
    pub fn join(&mut self, root: impl AsRef<Path>) {
        self.file = root.as_ref().join(&self.file);
    }

    /// Removes `base` from the artifact's path
    pub fn strip_prefix(&mut self, base: impl AsRef<Path>) {
        if let Ok(prefix) = self.file.strip_prefix(base) {
            self.file = prefix.to_path_buf();
        }
    }
}

/// local helper type alias
pub(crate) type ArtifactsMap<T> = FileToContractsMap<Vec<ArtifactFile<T>>>;

/// Represents a set of Artifacts
#[derive(Debug, Clone, PartialEq)]
pub struct Artifacts<T>(pub ArtifactsMap<T>);

impl<T> Default for Artifacts<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> AsRef<ArtifactsMap<T>> for Artifacts<T> {
    fn as_ref(&self) -> &ArtifactsMap<T> {
        &self.0
    }
}

impl<T> AsMut<ArtifactsMap<T>> for Artifacts<T> {
    fn as_mut(&mut self) -> &mut ArtifactsMap<T> {
        &mut self.0
    }
}

impl<T: Serialize> Artifacts<T> {
    /// Writes all artifacts into the given `artifacts_root` folder
    pub fn write_all(&self) -> Result<()> {
        for artifact in self.artifact_files() {
            artifact.write()?;
        }
        Ok(())
    }
}

impl<T> Artifacts<T> {
    /// Sets the artifact files location to `root` adjoined to `self.file`.
    pub fn join_all(&mut self, root: impl AsRef<Path>) -> &mut Self {
        let root = root.as_ref();
        self.artifact_files_mut().for_each(|artifact| artifact.join(root));
        self
    }

    /// Removes `base` from all artifacts
    pub fn strip_prefix_all(&mut self, base: impl AsRef<Path>) -> &mut Self {
        let base = base.as_ref();
        self.artifact_files_mut().for_each(|artifact| artifact.strip_prefix(base));
        self
    }

    /// Returns all `ArtifactFile`s for the contract with the matching name
    fn get_contract_artifact_files(&self, contract_name: &str) -> Option<&Vec<ArtifactFile<T>>> {
        self.0.values().find_map(|all| all.get(contract_name))
    }

    /// Returns true if this type contains an artifact with the given path for the given contract
    pub fn has_contract_artifact(&self, contract_name: &str, artifact_path: &Path) -> bool {
        self.get_contract_artifact_files(contract_name)
            .map(|artifacts| artifacts.into_iter().any(|artifact| artifact.file == artifact_path))
            .unwrap_or_default()
    }

    /// Returns true if this type contains an artifact with the given path
    pub fn has_artifact(&self, artifact_path: &Path) -> bool {
        self.artifact_files().any(|artifact| artifact.file == artifact_path)
    }

    /// Iterate over all artifact files
    pub fn artifact_files(&self) -> impl Iterator<Item = &ArtifactFile<T>> {
        self.0.values().flat_map(|c| c.values().flat_map(|artifacts| artifacts.iter()))
    }
    /// Iterate over all artifact files
    pub fn artifact_files_mut(&mut self) -> impl Iterator<Item = &mut ArtifactFile<T>> {
        self.0.values_mut().flat_map(|c| c.values_mut().flat_map(|artifacts| artifacts.iter_mut()))
    }

    /// Returns an iterator over _all_ artifacts and `<file name:contract name>`
    pub fn into_artifacts<O: ArtifactOutput<Artifact = T>>(
        self,
    ) -> impl Iterator<Item = (String, T)> {
        self.0.into_values().flat_map(|contract_artifacts| {
            contract_artifacts.into_iter().flat_map(|(_contract_name, artifacts)| {
                artifacts.into_iter().filter_map(|artifact| {
                    O::contract_name(&artifact.file).map(|name| {
                        (
                            format!(
                                "{}:{}",
                                artifact.file.file_name().unwrap().to_string_lossy(),
                                name
                            ),
                            artifact.artifact,
                        )
                    })
                })
            })
        })
    }

    /// Finds the first artifact `T` with a matching contract name
    pub fn find(&self, contract_name: impl AsRef<str>) -> Option<&T> {
        let contract_name = contract_name.as_ref();
        self.0.iter().find_map(|(_file, contracts)| {
            contracts.get(contract_name).and_then(|c| c.get(0).map(|a| &a.artifact))
        })
    }

    /// Removes the first artifact `T` with a matching contract name
    ///
    /// *Note:* if there are multiple artifacts (contract compiled with different solc) then this
    /// returns the first artifact in that set
    pub fn remove(&mut self, contract_name: impl AsRef<str>) -> Option<T> {
        let contract_name = contract_name.as_ref();
        self.0.iter_mut().find_map(|(_file, contracts)| {
            let mut artifact = None;
            if let Some((c, mut artifacts)) = contracts.remove_entry(contract_name) {
                if !artifacts.is_empty() {
                    artifact = Some(artifacts.remove(0).artifact);
                }
                if !artifacts.is_empty() {
                    contracts.insert(c, artifacts);
                }
            }
            artifact
        })
    }
}

// /// Bundled Artifacts: `file -> (contract name -> (Artifact, Version))`
// pub type Artifacts<T> = FileToContractsMap<Vec<(T, Version)>>;

/// A trait representation for a [`crate::Contract`] artifact
pub trait Artifact {
    /// Returns the artifact's `Abi` and bytecode
    fn into_inner(self) -> (Option<Abi>, Option<Bytes>);

    /// Turns the artifact into a container type for abi, compact bytecode and deployed bytecode
    fn into_compact_contract(self) -> CompactContract;

    /// Turns the artifact into a container type for abi, full bytecode and deployed bytecode
    fn into_contract_bytecode(self) -> CompactContractBytecode;

    /// Returns the contents of this type as a single tuple of abi, bytecode and deployed bytecode
    fn into_parts(self) -> (Option<Abi>, Option<Bytes>, Option<Bytes>);
}

impl<T> Artifact for T
where
    T: Into<CompactContractBytecode> + Into<CompactContract>,
{
    fn into_inner(self) -> (Option<Abi>, Option<Bytes>) {
        let artifact = self.into_compact_contract();
        (artifact.abi, artifact.bin.and_then(|bin| bin.into_bytes()))
    }

    fn into_compact_contract(self) -> CompactContract {
        self.into()
    }

    fn into_contract_bytecode(self) -> CompactContractBytecode {
        self.into()
    }

    fn into_parts(self) -> (Option<Abi>, Option<Bytes>, Option<Bytes>) {
        self.into_compact_contract().into_parts()
    }
}

/// Handler invoked with the output of `solc`
///
/// Implementers of this trait are expected to take care of [`crate::Contract`] to
/// [`crate::ArtifactOutput::Artifact`] conversion and how that `Artifact` type is stored on disk,
/// this includes artifact file location and naming.
///
/// Depending on the [`crate::Project`] contracts and their compatible versions,
/// [`crate::ProjectCompiler::compile()`] may invoke different `solc` executables on the same
/// solidity file leading to multiple [`crate::CompilerOutput`]s for the same `.sol` file.
/// In addition to the `solidity file` to `contract` relationship (1-N*)
/// [`crate::VersionedContracts`] also tracks the `contract` to (`artifact` + `solc version`)
/// relationship (1-N+).
pub trait ArtifactOutput {
    /// Represents the artifact that will be stored for a `Contract`
    type Artifact: Artifact + DeserializeOwned + Serialize;

    /// Handle the aggregated set of compiled contracts from the solc [`crate::CompilerOutput`].
    ///
    /// This will be invoked with all aggregated contracts from (multiple) solc `CompilerOutput`.
    /// See [`crate::AggregatedCompilerOutput`]
    fn on_output(
        contracts: &VersionedContracts,
        layout: &ProjectPathsConfig,
    ) -> Result<Artifacts<Self::Artifact>> {
        let mut artifacts = Self::output_to_artifacts(contracts);
        artifacts.join_all(&layout.artifacts);
        artifacts.write_all()?;
        Ok(artifacts)
    }

    /// Returns the file name for the contract's artifact
    /// `Greeter.json`
    fn output_file_name(name: impl AsRef<str>) -> PathBuf {
        format!("{}.json", name.as_ref()).into()
    }

    /// Returns the file name for the contract's artifact and the given version
    /// `Greeter.0.8.11.json`
    fn output_file_name_versioned(name: impl AsRef<str>, version: &Version) -> PathBuf {
        format!("{}.{}.{}.{}.json", name.as_ref(), version.major, version.minor, version.patch)
            .into()
    }

    /// Returns the path to the contract's artifact location based on the contract's file and name
    ///
    /// This returns `contract.sol/contract.json` by default
    fn output_file(contract_file: impl AsRef<Path>, name: impl AsRef<str>) -> PathBuf {
        let name = name.as_ref();
        contract_file
            .as_ref()
            .file_name()
            .map(Path::new)
            .map(|p| p.join(Self::output_file_name(name)))
            .unwrap_or_else(|| Self::output_file_name(name))
    }

    /// Returns the path to the contract's artifact location based on the contract's file, name and
    /// version
    ///
    /// This returns `contract.sol/contract.0.8.11.json` by default
    fn output_file_versioned(
        contract_file: impl AsRef<Path>,
        name: impl AsRef<str>,
        version: &Version,
    ) -> PathBuf {
        let name = name.as_ref();
        contract_file
            .as_ref()
            .file_name()
            .map(Path::new)
            .map(|p| p.join(Self::output_file_name_versioned(name, version)))
            .unwrap_or_else(|| Self::output_file_name_versioned(name, version))
    }

    /// The inverse of `contract_file_name`
    ///
    /// Expected to return the solidity contract's name derived from the file path
    /// `sources/Greeter.sol` -> `Greeter`
    fn contract_name(file: impl AsRef<Path>) -> Option<String> {
        file.as_ref().file_stem().and_then(|s| s.to_str().map(|s| s.to_string()))
    }

    /// Whether the corresponding artifact of the given contract file and name exists
    fn output_exists(
        contract_file: impl AsRef<Path>,
        name: impl AsRef<str>,
        root: impl AsRef<Path>,
    ) -> bool {
        root.as_ref().join(Self::output_file(contract_file, name)).exists()
    }

    /// Read the artifact that's stored at the given path
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///     - The file does not exist
    ///     - The file's content couldn't be deserialized into the `Artifact` type
    fn read_cached_artifact(path: impl AsRef<Path>) -> Result<Self::Artifact> {
        let path = path.as_ref();
        let file = fs::File::open(path).map_err(|err| SolcError::io(err, path))?;
        let file = io::BufReader::new(file);
        Ok(serde_json::from_reader(file)?)
    }

    /// Read the cached artifacts that are located the paths the iterator yields
    ///
    /// See [`Self::read_cached_artifact()`]
    fn read_cached_artifacts<T, I>(files: I) -> Result<BTreeMap<PathBuf, Self::Artifact>>
    where
        I: IntoIterator<Item = T>,
        T: Into<PathBuf>,
    {
        let mut artifacts = BTreeMap::default();
        for path in files.into_iter() {
            let path = path.into();
            let artifact = Self::read_cached_artifact(&path)?;
            artifacts.insert(path, artifact);
        }
        Ok(artifacts)
    }

    /// Convert a contract to the artifact type
    ///
    /// This is the core conversion function that takes care of converting a `Contract` into the
    /// associated `Artifact` type
    fn contract_to_artifact(_file: &str, _name: &str, contract: Contract) -> Self::Artifact;

    /// Convert the compiler output into a set of artifacts
    ///
    /// **Note:** This does only convert, but _NOT_ write the artifacts to disk, See
    /// [`Self::on_output()`]
    fn output_to_artifacts(contracts: &VersionedContracts) -> Artifacts<Self::Artifact> {
        let mut artifacts = ArtifactsMap::new();
        for (file, contracts) in contracts.as_ref().iter() {
            let mut entries = BTreeMap::new();
            for (name, versioned_contracts) in contracts {
                let mut contracts = Vec::with_capacity(versioned_contracts.len());
                // check if the same contract compiled with multiple solc versions
                for contract in versioned_contracts {
                    let artifact_path = if versioned_contracts.len() > 1 {
                        Self::output_file_versioned(file, name, &contract.version)
                    } else {
                        Self::output_file(file, name)
                    };
                    let artifact =
                        Self::contract_to_artifact(file, name, contract.contract.clone());

                    contracts.push(ArtifactFile {
                        artifact,
                        file: artifact_path,
                        version: contract.version.clone(),
                    });
                }
                entries.insert(name.to_string(), contracts);
            }
            artifacts.insert(file.to_string(), entries);
        }

        Artifacts(artifacts)
    }
}

/// An Artifacts implementation that uses a compact representation
///
/// Creates a single json artifact with
/// ```json
///  {
///    "abi": [],
///    "bin": "...",
///    "runtime-bin": "..."
///  }
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MinimalCombinedArtifacts;

impl ArtifactOutput for MinimalCombinedArtifacts {
    type Artifact = CompactContractBytecode;

    fn contract_to_artifact(_file: &str, _name: &str, contract: Contract) -> Self::Artifact {
        Self::Artifact::from(contract)
    }
}

/// An Artifacts handler implementation that works the same as `MinimalCombinedArtifacts` but also
/// supports reading hardhat artifacts if an initial attempt to deserialize an artifact failed
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MinimalCombinedArtifactsHardhatFallback;

impl ArtifactOutput for MinimalCombinedArtifactsHardhatFallback {
    type Artifact = CompactContractBytecode;

    fn on_output(
        output: &VersionedContracts,
        layout: &ProjectPathsConfig,
    ) -> Result<Artifacts<Self::Artifact>> {
        MinimalCombinedArtifacts::on_output(output, layout)
    }

    fn read_cached_artifact(path: impl AsRef<Path>) -> Result<Self::Artifact> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|err| SolcError::io(err, path))?;
        if let Ok(a) = serde_json::from_str(&content) {
            Ok(a)
        } else {
            tracing::error!("Failed to deserialize compact artifact");
            tracing::trace!("Fallback to hardhat artifact deserialization");
            let artifact = serde_json::from_str::<HardhatArtifact>(&content)?;
            tracing::trace!("successfully deserialized hardhat artifact");
            Ok(artifact.into_contract_bytecode())
        }
    }

    fn contract_to_artifact(file: &str, name: &str, contract: Contract) -> Self::Artifact {
        MinimalCombinedArtifacts::contract_to_artifact(file, name, contract)
    }
}
