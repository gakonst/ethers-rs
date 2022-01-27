//! Output artifact handling

use crate::{
    artifacts::{
        CompactContract, CompactContractRef, Contract, VersionedContract, VersionedContracts,
    },
    error::Result,
    HardhatArtifact, ProjectPathsConfig, SolcError,
};
use ethers_core::{abi::Abi, types::Bytes};
use semver::Version;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

/// Represents a written [`crate::Contract`] artifact
#[derive(Debug, Clone)]
pub struct WrittenArtifact<T> {
    /// The Artifact that was written
    pub artifact: T,
    /// path to the file where the `artifact` was written to
    pub file: PathBuf,
    /// `solc` version that produced this artifact
    pub version: Version,
}

/// Bundled Artifacts: `file -> (contract name -> (Artifact, Version))`
pub type Artifacts<T> = BTreeMap<String, BTreeMap<String, Vec<(T, Version)>>>;

/// A trait representation for a [`crate::Contract`] artifact
pub trait Artifact {
    /// Returns the artifact's `Abi` and bytecode
    fn into_inner(self) -> (Option<Abi>, Option<Bytes>);

    /// Turns the artifact into a container type for abi, bytecode and deployed bytecode
    fn into_compact_contract(self) -> CompactContract;

    /// Returns the contents of this type as a single tuple of abi, bytecode and deployed bytecode
    fn into_parts(self) -> (Option<Abi>, Option<Bytes>, Option<Bytes>);
}

impl<T: Into<CompactContract>> Artifact for T {
    fn into_inner(self) -> (Option<Abi>, Option<Bytes>) {
        let artifact = self.into_compact_contract();
        (artifact.abi, artifact.bin.and_then(|bin| bin.into_bytes()))
    }

    fn into_compact_contract(self) -> CompactContract {
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
    type Artifact: Artifact + DeserializeOwned;

    /// Handle the aggregated set of compiled contracts from the solc [`crate::CompilerOutput`].
    ///
    /// This will be invoked with all aggregated contracts from (multiple) solc `CompilerOutput`.
    /// See [`crate::AggregatedCompilerOutput`]
    fn on_output(contracts: &VersionedContracts, layout: &ProjectPathsConfig) -> Result<()>;

    /// Returns the file name for the contract's artifact
    /// `Greeter.0.8.11.json`
    fn output_file_name(name: impl AsRef<str>) -> PathBuf {
        format!("{}.json", name.as_ref()).into()
    }

    /// Returns the file name for the contract's artifact and the given version
    /// `Greeter.0.8.11.json`
    fn versioned_output_file_name(name: impl AsRef<str>, version: &Version) -> PathBuf {
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
    fn versioned_output_file(
        contract_file: impl AsRef<Path>,
        name: impl AsRef<str>,
        version: &Version,
    ) -> PathBuf {
        let name = name.as_ref();
        contract_file
            .as_ref()
            .file_name()
            .map(Path::new)
            .map(|p| p.join(Self::versioned_output_file_name(name, version)))
            .unwrap_or_else(|| Self::versioned_output_file_name(name, version))
    }

    /// The inverse of `contract_file_name`
    ///
    /// Expected to return the solidity contract's name derived from the file path
    /// `sources/Greeter.sol` -> `Greeter`
    fn contract_name(file: impl AsRef<Path>) -> Option<String> {
        // TODO support version
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
    fn contract_to_artifact(file: &str, name: &str, contract: Contract) -> Self::Artifact;

    /// Convert a contract to the artifact type
    fn contract_to_versioned_artifact(
        file: &str,
        name: &str,
        version: &Version,
        contract: Contract,
    ) -> Self::Artifact {
        todo!()
    }

    fn versioned_contracts_to_artifacts(
        file: &str,
        name: &str,
        contracts: Vec<VersionedContract>,
    ) -> Vec<(Self::Artifact, Version)> {
        todo!()
    }

    /// Convert the compiler output into a set of artifacts
    fn output_to_artifacts(contracts: VersionedContracts) -> Artifacts<Self::Artifact> {
        contracts
            .into_iter()
            .map(|(file, contracts)| {
                let contracts = contracts
                    .into_iter()
                    .map(|(name, versioned)| {
                        let contracts =
                            Self::versioned_contracts_to_artifacts(&file, &name, versioned);
                        (name, contracts)
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
    type Artifact = CompactContract;

    fn on_output(
        contracts: &VersionedContracts,
        layout: &ProjectPathsConfig,
    ) -> Result<Artifacts<Self::Artifact>> {
        fs::create_dir_all(&layout.artifacts)
            .map_err(|err| SolcError::msg(format!("Failed to create artifacts dir: {}", err)))?;
        let mut artifacts = Artifacts::new();

        for (file, contracts) in contracts.iter() {
            for (name, versioned_contracts) in contracts {
                let mut contracts = Vec::with_capacity(versioned_contracts.len());

                // check if the same contract compiled with multiple solc versions
                for contract in versioned_contracts {
                    let artifact_path = if versioned_contracts.len() > 1 {
                        Self::versioned_output_file(file, name, &contract.version)
                    } else {
                        Self::output_file(file, name)
                    };
                    let artifact = write_contract::<Self::Artifact>(
                        &layout.artifacts.join(&artifact_path),
                        &contract.contract,
                    )?;
                    contracts.push((artifact, artifact_path));
                }
            }
        }

        Ok(artifacts)
    }

    fn contract_to_artifact(_file: &str, _name: &str, contract: Contract) -> Self::Artifact {
        Self::Artifact::from(contract)
    }
}

/// Writes the given
fn write_contract<C>(out: &Path, contract: &Contract) -> Result<C>
where
    C: From<&Contract> + Serialize,
{
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            SolcError::msg(format!(
                "Failed to create artifact parent folder \"{}\": {}",
                parent.display(),
                err
            ))
        })?;
    }
    let c = C::from(contract);
    fs::write(out, serde_json::to_vec_pretty(&c)?).map_err(|err| SolcError::io(err, out))?;
    Ok(c)
}

/// An Artifacts handler implementation that works the same as `MinimalCombinedArtifacts` but also
/// supports reading hardhat artifacts if an initial attempt to deserialize an artifact failed
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MinimalCombinedArtifactsHardhatFallback;

impl ArtifactOutput for MinimalCombinedArtifactsHardhatFallback {
    type Artifact = CompactContract;

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
            Ok(artifact.into_compact_contract())
        }
    }

    fn contract_to_artifact(file: &str, name: &str, contract: Contract) -> Self::Artifact {
        MinimalCombinedArtifacts::contract_to_artifact(file, name, contract)
    }
}
