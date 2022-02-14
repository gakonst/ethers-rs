//! A configurable artifacts handler implementation

use crate::{
    artifacts::{
        output_selection::{ContractOutputSelection, EvmOutputSelection},
        CompactContract, CompactContractBytecode, CompactEvm, DevDoc, Ewasm, GasEstimates,
        Metadata, StorageLayout, UserDoc,
    },
    ArtifactOutput, Contract, SolcError,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

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

impl ConfigurableContractArtifact {
    /// Returns the inner element that contains the core bytecode related information
    pub fn into_contract_bytecode(self) -> CompactContractBytecode {
        self.compact
    }
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

    fn write_contract_extras(&self, contract: &Contract, file: &Path) -> Result<(), SolcError> {
        self.additional_files.write_extras(contract, file)
    }

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

impl AdditionalArtifactValues {
    /// Returns an instance where all values are set to `true`
    pub fn all() -> Self {
        Self {
            ast: true,
            userdoc: true,
            devdoc: true,
            method_identifiers: true,
            storage_layout: true,
            assembly: true,
            gas_estimates: true,
            compact_format: true,
            metadata: true,
            ir: true,
            ir_optimized: true,
            ewasm: true,
            __non_exhaustive: (),
        }
    }

    /// Sets the values based on a set of `ContractOutputSelection`
    pub fn from_output_selection(
        settings: impl IntoIterator<Item = ContractOutputSelection>,
    ) -> Self {
        let mut config = Self::default();
        for value in settings.into_iter() {
            match value {
                ContractOutputSelection::DevDoc => {
                    config.devdoc = true;
                }
                ContractOutputSelection::UserDoc => {
                    config.userdoc = true;
                }
                ContractOutputSelection::Metadata => {
                    config.metadata = true;
                }
                ContractOutputSelection::Ir => {
                    config.ir = true;
                }
                ContractOutputSelection::IrOptimized => {
                    config.ir_optimized = true;
                }
                ContractOutputSelection::StorageLayout => {
                    config.storage_layout = true;
                }
                ContractOutputSelection::Evm(evm) => match evm {
                    EvmOutputSelection::All => {
                        config.assembly = true;
                        config.gas_estimates = true;
                        config.method_identifiers = true;
                    }
                    EvmOutputSelection::Assembly => {
                        config.assembly = true;
                    }
                    EvmOutputSelection::MethodIdentifiers => {
                        config.method_identifiers = true;
                    }
                    EvmOutputSelection::GasEstimates => {
                        config.gas_estimates = true;
                    }
                    _ => {}
                },
                ContractOutputSelection::Ewasm(_) => {
                    config.ewasm = true;
                }
                _ => {}
            }
        }

        config
    }
}

/// Determines what to emit as additional file
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct AdditionalArtifactFiles {
    pub metadata: bool,
    pub ir_optimized: bool,
    pub ewasm: bool,
    pub assembly: bool,

    /// PRIVATE: This structure may grow, As such, constructing this structure should
    /// _always_ be done using a public constructor or update syntax:
    ///
    /// ```rust
    /// 
    /// use ethers_solc::AdditionalArtifactFiles;
    /// let config = AdditionalArtifactFiles {
    ///     metadata: true,
    ///     ..Default::default()
    /// };
    /// ```
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl AdditionalArtifactFiles {
    /// Returns an instance where all values are set to `true`
    pub fn all() -> Self {
        Self {
            metadata: true,
            ir_optimized: true,
            ewasm: true,
            assembly: true,
            __non_exhaustive: (),
        }
    }

    /// Write the set values as separate files
    pub fn write_extras(&self, contract: &Contract, file: &Path) -> Result<(), SolcError> {
        if self.metadata {
            if let Some(ref metadata) = contract.metadata {
                let file = file.join(".metadata.json");
                fs::write(&file, serde_json::to_string_pretty(metadata)?)
                    .map_err(|err| SolcError::io(err, file))?
            }
        }

        if self.ir_optimized {
            if let Some(ref iropt) = contract.ir_optimized {
                let file = file.with_extension("iropt");
                fs::write(&file, iropt).map_err(|err| SolcError::io(err, file))?
            }
        }

        if self.ewasm {
            if let Some(ref ir) = contract.ir {
                let file = file.with_extension("ir");
                fs::write(&file, ir).map_err(|err| SolcError::io(err, file))?
            }
        }

        if self.ewasm {
            if let Some(ref ewasm) = contract.ewasm {
                let file = file.with_extension("ewasm");
                fs::write(&file, serde_json::to_vec_pretty(ewasm)?)
                    .map_err(|err| SolcError::io(err, file))?;
            }
        }

        if self.assembly {
            if let Some(ref evm) = contract.evm {
                if let Some(ref asm) = evm.assembly {
                    let file = file.with_extension("asm");
                    fs::write(&file, asm).map_err(|err| SolcError::io(err, file))?
                }
            }
        }

        Ok(())
    }
}
