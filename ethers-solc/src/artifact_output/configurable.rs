//! A configurable artifacts handler implementation

use crate::{
    artifacts::{
        CompactContract, CompactContractBytecode, CompactEvm, DevDoc, Ewasm, GasEstimates,
        Metadata, StorageLayout, UserDoc,
    },
    ArtifactOutput, Contract,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents the `Artifact` that `ConfigurableArtifacts` emits.
///
/// This is essentially a superset of [`CompactContractBytecode`].
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurableContractArtifact {
    /// The essential values of the contract, abi, bytecode, deployedBytecode
    #[serde(flatten)]
    pub compact: CompactContractBytecode,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembly: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method_identifiers: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_estimates: Option<GasEstimates>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_layout: Option<StorageLayout>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub userdoc: Option<UserDoc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub devdoc: Option<DevDoc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ir_optimized: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ewasm: Option<Ewasm>,
}

impl From<ConfigurableContractArtifact> for CompactContractBytecode {
    fn from(artifact: ConfigurableContractArtifact) -> Self {
        artifact.compact
    }
}

impl From<ConfigurableContractArtifact> for CompactContract {
    fn from(artifact: ConfigurableContractArtifact) -> Self {
        artifact.compact.into()
    }
}

/// An `Artifact` implementation that can be configured to include additional content and emit
/// additional files
///
/// Creates a single json artifact with
/// ```json
///  {
///    "abi": [],
///    "bytecode": {...},
///    "deployedBytecode": {...}
///    // additional values
///  }
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct ConfigurableArtifacts {
    /// A set of additional values to include in the contract's artifact file
    pub additional_values: AdditionalArtifactValues,

    /// A set of values that should be written to a separate file
    pub additional_files: AdditionalArtifactFiles,

    /// PRIVATE: This structure may grow, As such, constructing this structure should
    /// _always_ be done using a public constructor or update syntax:
    ///
    /// ```rust
    /// 
    /// use ethers_solc::{AdditionalArtifactFiles, ConfigurableArtifacts};
    /// let config = ConfigurableArtifacts {
    ///     additional_files: AdditionalArtifactFiles { metadata: true, ..Default::default() },
    ///     ..Default::default()
    /// };
    /// ```
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl ArtifactOutput for ConfigurableArtifacts {
    type Artifact = ConfigurableContractArtifact;

    fn contract_to_artifact(&self, _file: &str, _name: &str, contract: Contract) -> Self::Artifact {
        let mut artifact_userdoc = None;
        let mut artifact_devdoc = None;
        let mut artifact_metadata = None;
        let mut artifact_ir = None;
        let mut artifact_ir_optimized = None;
        let mut artifact_ewasm = None;
        let mut artifact_bytecode = None;
        let mut artifact_deployed_bytecode = None;
        let mut artifact_gas_estimates = None;
        let mut artifact_method_identifiers = None;
        let mut artifact_assembly = None;
        let mut artifact_storage_layout = None;

        let Contract {
            abi,
            metadata,
            userdoc,
            devdoc,
            ir,
            storage_layout,
            evm,
            ewasm,
            ir_optimized,
        } = contract;

        if self.additional_values.metadata {
            artifact_metadata = metadata;
        }
        if self.additional_values.userdoc {
            artifact_userdoc = Some(userdoc);
        }
        if self.additional_values.devdoc {
            artifact_devdoc = Some(devdoc);
        }
        if self.additional_values.ewasm {
            artifact_ewasm = ewasm;
        }
        if self.additional_values.ir {
            artifact_ir = ir;
        }
        if self.additional_values.ir_optimized {
            artifact_ir_optimized = ir_optimized;
        }
        if self.additional_values.storage_layout {
            artifact_storage_layout = Some(storage_layout);
        }

        if let Some(evm) = evm {
            let CompactEvm {
                assembly,
                bytecode,
                deployed_bytecode,
                method_identifiers,
                gas_estimates,
                ..
            } = evm.into_compact();

            artifact_bytecode = bytecode;
            artifact_deployed_bytecode = deployed_bytecode;

            if self.additional_values.gas_estimates {
                artifact_gas_estimates = gas_estimates;
            }
            if self.additional_values.method_identifiers {
                artifact_method_identifiers = Some(method_identifiers);
            }
            if self.additional_values.assembly {
                artifact_assembly = assembly;
            }
        }

        let compact = CompactContractBytecode {
            abi,
            bytecode: artifact_bytecode,
            deployed_bytecode: artifact_deployed_bytecode,
        };

        ConfigurableContractArtifact {
            compact,
            assembly: artifact_assembly,
            method_identifiers: artifact_method_identifiers,
            gas_estimates: artifact_gas_estimates,
            metadata: artifact_metadata,
            storage_layout: artifact_storage_layout,
            userdoc: artifact_userdoc,
            devdoc: artifact_devdoc,
            ir: artifact_ir,
            ir_optimized: artifact_ir_optimized,
            ewasm: artifact_ewasm,
        }
    }
}

/// Determines the additional values to include in the contract's artifact file
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct AdditionalArtifactValues {
    pub ast: bool,
    pub userdoc: bool,
    pub devdoc: bool,
    pub method_identifiers: bool,
    pub storage_layout: bool,
    pub assembly: bool,
    pub gas_estimates: bool,
    pub compact_format: bool,
    pub metadata: bool,
    pub ir: bool,
    pub ir_optimized: bool,
    pub ewasm: bool,

    /// PRIVATE: This structure may grow, As such, constructing this structure should
    /// _always_ be done using a public constructor or update syntax:
    ///
    /// ```rust
    /// 
    /// use ethers_solc::AdditionalArtifactValues;
    /// let config = AdditionalArtifactValues {
    ///     ir: true,
    ///     ..Default::default()
    /// };
    /// ```
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

/// Determines what to emit as additional file
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct AdditionalArtifactFiles {
    pub metadata: bool,
    pub ir: bool,
    pub ir_optimized: bool,
    pub assembly: bool,
    pub method_identifiers: bool,

    /// PRIVATE: This structure may grow, As such, constructing this structure should
    /// _always_ be done using a public constructor or update syntax:
    ///
    /// ```rust
    /// 
    /// use ethers_solc::AdditionalArtifactFiles;
    /// let config = AdditionalArtifactFiles {
    ///     ir: true,
    ///     ..Default::default()
    /// };
    /// ```
    #[doc(hidden)]
    pub __non_exhaustive: (),
}
