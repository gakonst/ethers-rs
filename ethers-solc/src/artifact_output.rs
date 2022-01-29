//! Output artifact handling

use crate::{
    artifacts::{
        CompactContract, CompactContractBytecode, Contract, FileToContractsMap, VersionedContract,
        VersionedContracts,
    },
    error::Result,
    output::AggregatedCompilerOutput,
    HardhatArtifact, ProjectPathsConfig, SolcError,
};
use ethers_core::{abi::Abi, types::Bytes};
use semver::Version;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::btree_map::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

/// Represents a written [`crate::Contract`] artifact
#[derive(Debug, Clone, PartialEq)]
pub struct WrittenArtifact<T> {
    /// The Artifact that was written
    pub artifact: T,
    /// path to the file where the `artifact` was written to
    pub file: PathBuf,
    /// `solc` version that produced this artifact
    pub version: Version,
}

/// local helper type alias
type ArtifactsMap<T> = FileToContractsMap<Vec<WrittenArtifact<T>>>;

/// Represents the written Artifacts
#[derive(Debug, Clone, PartialEq)]
pub struct WrittenArtifacts<T>(pub ArtifactsMap<T>);

impl<T> Default for WrittenArtifacts<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> AsRef<ArtifactsMap<T>> for WrittenArtifacts<T> {
    fn as_ref(&self) -> &ArtifactsMap<T> {
        &self.0
    }
}

impl<T> AsMut<ArtifactsMap<T>> for WrittenArtifacts<T> {
    fn as_mut(&mut self) -> &mut ArtifactsMap<T> {
        &mut self.0
    }
}

impl<T> WrittenArtifacts<T> {
    /// Returns an iterator over _all_ artifacts and `<file name:contract name>`
    pub fn into_artifacts<O: ArtifactOutput<Artifact = T>>(
        self,
    ) -> impl Iterator<Item = (String, T)> {
        self.0.into_values().flat_map(|contract_artifacts| {
            contract_artifacts.into_iter().flat_map(|(contract_name, artifacts)| {
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
        self.0.iter().find_map(|(file, contracts)| {
            contracts.get(contract_name).and_then(|c| c.get(0).map(|a| &a.artifact))
        })
    }

    /// Removes the first artifact `T` with a matching contract name
    ///
    /// *Note:* if there are multiple artifacts (contract compiled with different solc) then this
    /// returns the first artifact in that set
    pub fn remove(&mut self, contract_name: impl AsRef<str>) -> Option<T> {
        let contract_name = contract_name.as_ref();
        self.0.iter_mut().find_map(|(file, contracts)| {
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

/// Bundled Artifacts: `file -> (contract name -> (Artifact, Version))`
pub type Artifacts<T> = FileToContractsMap<Vec<(T, Version)>>;

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
    ) -> Result<WrittenArtifacts<Self::Artifact>> {
        fs::create_dir_all(&layout.artifacts)
            .map_err(|err| SolcError::msg(format!("Failed to create artifacts dir: {}", err)))?;
        let mut artifacts = ArtifactsMap::new();

        for (file, contracts) in contracts.iter() {
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

                    write_contract::<Self::Artifact>(
                        &layout.artifacts.join(&artifact_path),
                        &artifact,
                    )?;

                    contracts.push(WrittenArtifact {
                        artifact,
                        file: artifact_path,
                        version: contract.version.clone(),
                    });
                }
                entries.insert(name.to_string(), contracts);
            }
            artifacts.insert(file.to_string(), entries);
        }

        Ok(WrittenArtifacts(artifacts))
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

    fn read_cached_artifact(path: impl AsRef<Path>) -> Result<Self::Artifact> {
        let path = path.as_ref();
        let file = fs::File::open(path).map_err(|err| SolcError::io(err, path))?;
        let file = io::BufReader::new(file);
        Ok(serde_json::from_reader(file)?)
    }

    /// Read the cached artifacts from disk
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
    fn contract_to_artifact(_file: &str, _name: &str, contract: Contract) -> Self::Artifact;

    /// Convert the compiler output into a set of artifacts
    fn output_to_artifacts(output: AggregatedCompilerOutput) -> Artifacts<Self::Artifact> {
        output
            .contracts
            .into_iter()
            .map(|(file, all_contracts)| {
                let contracts = all_contracts
                    .into_iter()
                    .map(|(name, versioned_contracts)| {
                        let artifacts = versioned_contracts
                            .into_iter()
                            .map(|c| {
                                let VersionedContract { contract, version } = c;
                                let artifact = Self::contract_to_artifact(&file, &name, contract);
                                (artifact, version)
                            })
                            .collect();
                        (name, artifacts)
                    })
                    .collect();

                (file, contracts)
            })
            .collect()
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
    ) -> Result<WrittenArtifacts<Self::Artifact>> {
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

/// Writes the given contract to the `out` path creating all parent directories
fn write_contract<T: Serialize>(out: &Path, artifact: &T) -> Result<()> {
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            SolcError::msg(format!(
                "Failed to create artifact parent folder \"{}\": {}",
                parent.display(),
                err
            ))
        })?;
    }
    fs::write(out, serde_json::to_vec_pretty(artifact)?).map_err(|err| SolcError::io(err, out))?;
    Ok(())
}
