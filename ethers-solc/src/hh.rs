//! Hardhat support

use crate::{
    artifacts::{
        bytecode::{Bytecode, BytecodeObject, DeployedBytecode},
        contract::{CompactContract, CompactContractBytecode, Contract, ContractBytecode},
        CompactContractBytecodeCow, LosslessAbi, Offsets,
    },
    ArtifactOutput, SourceFile, VersionedSourceFile,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::btree_map::BTreeMap};

const HH_ARTIFACT_VERSION: &str = "hh-sol-artifact-1";

/// A hardhat artifact
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardhatArtifact {
    #[serde(rename = "_format")]
    pub format: String,
    /// A string with the contract's name.
    pub contract_name: String,
    /// The source name of this contract in the workspace like `contracts/Greeter.sol`
    pub source_name: String,
    /// The contract's ABI
    pub abi: LosslessAbi,
    /// A "0x"-prefixed hex string of the unlinked deployment bytecode. If the contract is not
    /// deployable, this has the string "0x"
    pub bytecode: Option<BytecodeObject>,
    /// A "0x"-prefixed hex string of the unlinked runtime/deployed bytecode. If the contract is
    /// not deployable, this has the string "0x"
    pub deployed_bytecode: Option<BytecodeObject>,
    /// The bytecode's link references object as returned by solc. If the contract doesn't need to
    /// be linked, this value contains an empty object.
    #[serde(default)]
    pub link_references: BTreeMap<String, BTreeMap<String, Vec<Offsets>>>,
    /// The deployed bytecode's link references object as returned by solc. If the contract doesn't
    /// need to be linked, this value contains an empty object.
    #[serde(default)]
    pub deployed_link_references: BTreeMap<String, BTreeMap<String, Vec<Offsets>>>,
}

impl<'a> From<&'a HardhatArtifact> for CompactContractBytecodeCow<'a> {
    fn from(artifact: &'a HardhatArtifact) -> Self {
        let c: ContractBytecode = artifact.clone().into();
        CompactContractBytecodeCow {
            abi: Some(Cow::Borrowed(&artifact.abi.abi)),
            bytecode: c.bytecode.map(|b| Cow::Owned(b.into())),
            deployed_bytecode: c.deployed_bytecode.map(|b| Cow::Owned(b.into())),
        }
    }
}

impl From<HardhatArtifact> for CompactContract {
    fn from(artifact: HardhatArtifact) -> Self {
        CompactContract {
            abi: Some(artifact.abi.abi),
            bin: artifact.bytecode,
            bin_runtime: artifact.deployed_bytecode,
        }
    }
}

impl From<HardhatArtifact> for ContractBytecode {
    fn from(artifact: HardhatArtifact) -> Self {
        let bytecode: Option<Bytecode> = artifact.bytecode.as_ref().map(|t| {
            let mut bcode: Bytecode = t.clone().into();
            bcode.link_references = artifact.link_references.clone();
            bcode
        });

        let deployed_bytecode: Option<DeployedBytecode> = artifact.bytecode.as_ref().map(|t| {
            let mut bcode: Bytecode = t.clone().into();
            bcode.link_references = artifact.deployed_link_references.clone();
            bcode.into()
        });

        ContractBytecode { abi: Some(artifact.abi.abi), bytecode, deployed_bytecode }
    }
}

impl From<HardhatArtifact> for CompactContractBytecode {
    fn from(artifact: HardhatArtifact) -> Self {
        let c: ContractBytecode = artifact.into();

        c.into()
    }
}

/// Hardhat style artifacts handler
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct HardhatArtifacts {
    _priv: (),
}

impl ArtifactOutput for HardhatArtifacts {
    type Artifact = HardhatArtifact;

    fn contract_to_artifact(
        &self,
        file: &str,
        name: &str,
        contract: Contract,
        _source_file: Option<&SourceFile>,
    ) -> Self::Artifact {
        let (bytecode, link_references, deployed_bytecode, deployed_link_references) =
            if let Some(evm) = contract.evm {
                let (deployed_bytecode, deployed_link_references) =
                    if let Some(code) = evm.deployed_bytecode.and_then(|code| code.bytecode) {
                        (Some(code.object), code.link_references)
                    } else {
                        (None, Default::default())
                    };

                let (bytecode, link_ref) = if let Some(bc) = evm.bytecode {
                    (Some(bc.object), bc.link_references)
                } else {
                    (None, Default::default())
                };

                (bytecode, link_ref, deployed_bytecode, deployed_link_references)
            } else {
                (Default::default(), Default::default(), None, Default::default())
            };

        HardhatArtifact {
            format: HH_ARTIFACT_VERSION.to_string(),
            contract_name: name.to_string(),
            source_name: file.to_string(),
            abi: contract.abi.unwrap_or_default(),
            bytecode,
            deployed_bytecode,
            link_references,
            deployed_link_references,
        }
    }

    fn standalone_source_file_to_artifact(
        &self,
        _path: &str,
        _file: &VersionedSourceFile,
    ) -> Option<Self::Artifact> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Artifact;

    #[test]
    fn can_parse_hh_artifact() {
        let s = include_str!("../test-data/hh-greeter-artifact.json");
        let artifact = serde_json::from_str::<HardhatArtifact>(s).unwrap();
        let compact = artifact.into_compact_contract();
        assert!(compact.abi.is_some());
        assert!(compact.bin.is_some());
        assert!(compact.bin_runtime.is_some());
    }
}
