//! A configurable artifacts handler implementation
//!
//! Configuring artifacts requires two pieces: the `ConfigurableArtifacts` handler, which contains
//! the configuration of how to construct the `ConfigurableArtifact` type based on a `Contract`. The
//! `ConfigurableArtifacts` populates a single `Artifact`, the `ConfigurableArtifact`, by default
//! with essential entries only, such as `abi`, `bytecode`,..., but may include additional values
//! based on its `ExtraOutputValues` that maps to various objects in the solc contract output, see
//! also: [`OutputSelection`](crate::artifacts::output_selection::OutputSelection). In addition to
//! that some output values can also be emitted as standalone files.

use crate::{
    artifacts::{
        bytecode::{CompactBytecode, CompactDeployedBytecode},
        contract::{CompactContract, CompactContractBytecode, Contract},
        output_selection::{
            BytecodeOutputSelection, ContractOutputSelection, DeployedBytecodeOutputSelection,
            EvmOutputSelection, EwasmOutputSelection,
        },
        Ast, CompactContractBytecodeCow, DevDoc, Evm, Ewasm, FunctionDebugData, GasEstimates,
        GeneratedSource, LosslessAbi, LosslessMetadata, Metadata, Offsets, Settings, StorageLayout,
        UserDoc,
    },
    sources::VersionedSourceFile,
    Artifact, ArtifactOutput, SolcConfig, SolcError, SourceFile,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap, fs, path::Path};

/// Represents the `Artifact` that `ConfigurableArtifacts` emits.
///
/// This is essentially a superset of [`CompactContractBytecode`].
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurableContractArtifact {
    /// The Ethereum Contract ABI. If empty, it is represented as an empty
    /// array. See <https://docs.soliditylang.org/en/develop/abi-spec.html>
    pub abi: Option<LosslessAbi>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytecode: Option<CompactBytecode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployed_bytecode: Option<CompactDeployedBytecode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembly: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opcodes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method_identifiers: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generated_sources: Vec<GeneratedSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_debug_data: Option<BTreeMap<String, FunctionDebugData>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_estimates: Option<GasEstimates>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_metadata: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ast: Option<Ast>,
    /// The identifier of the source file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
}

impl ConfigurableContractArtifact {
    /// Returns the inner element that contains the core bytecode related information
    pub fn into_contract_bytecode(self) -> CompactContractBytecode {
        self.into()
    }

    /// Looks for all link references in deployment and runtime bytecodes
    pub fn all_link_references(&self) -> BTreeMap<String, BTreeMap<String, Vec<Offsets>>> {
        let mut links = BTreeMap::new();
        if let Some(bcode) = &self.bytecode {
            links.extend(bcode.link_references.clone());
        }

        if let Some(d_bcode) = &self.deployed_bytecode {
            if let Some(bcode) = &d_bcode.bytecode {
                links.extend(bcode.link_references.clone());
            }
        }
        links
    }

    /// Returns the source file of this artifact's contract
    pub fn source_file(&self) -> Option<SourceFile> {
        self.id.map(|id| SourceFile { id, ast: self.ast.clone() })
    }
}

impl From<ConfigurableContractArtifact> for CompactContractBytecode {
    fn from(artifact: ConfigurableContractArtifact) -> Self {
        CompactContractBytecode {
            abi: artifact.abi.map(Into::into),
            bytecode: artifact.bytecode,
            deployed_bytecode: artifact.deployed_bytecode,
        }
    }
}

impl From<ConfigurableContractArtifact> for CompactContract {
    fn from(artifact: ConfigurableContractArtifact) -> Self {
        CompactContractBytecode::from(artifact).into()
    }
}

impl<'a> From<&'a ConfigurableContractArtifact> for CompactContractBytecodeCow<'a> {
    fn from(artifact: &'a ConfigurableContractArtifact) -> Self {
        CompactContractBytecodeCow {
            abi: artifact.abi.as_ref().map(|abi| Cow::Borrowed(&abi.abi)),
            bytecode: artifact.bytecode.as_ref().map(Cow::Borrowed),
            deployed_bytecode: artifact.deployed_bytecode.as_ref().map(Cow::Borrowed),
        }
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
///    "deployedBytecode": {...},
///    "methodIdentifiers": {...},
///    // additional values
///  }
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct ConfigurableArtifacts {
    /// A set of additional values to include in the contract's artifact file
    pub additional_values: ExtraOutputValues,

    /// A set of values that should be written to a separate file
    pub additional_files: ExtraOutputFiles,

    /// PRIVATE: This structure may grow, As such, constructing this structure should
    /// _always_ be done using a public constructor or update syntax:
    ///
    /// ```rust
    /// 
    /// use ethers_solc::{ExtraOutputFiles, ConfigurableArtifacts};
    /// let config = ConfigurableArtifacts {
    ///     additional_files: ExtraOutputFiles { metadata: true, ..Default::default() },
    ///     ..Default::default()
    /// };
    /// ```
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl ConfigurableArtifacts {
    pub fn new(
        extra_values: impl IntoIterator<Item = ContractOutputSelection>,
        extra_files: impl IntoIterator<Item = ContractOutputSelection>,
    ) -> Self {
        Self {
            additional_values: ExtraOutputValues::from_output_selection(extra_values),
            additional_files: ExtraOutputFiles::from_output_selection(extra_files),
            ..Default::default()
        }
    }

    /// Returns the `Settings` this configuration corresponds to
    pub fn settings(&self) -> Settings {
        SolcConfig::builder().additional_outputs(self.output_selection()).build().into()
    }

    /// Returns the output selection corresponding to this configuration
    pub fn output_selection(&self) -> Vec<ContractOutputSelection> {
        let mut selection = ContractOutputSelection::basic();

        if self.additional_values.ir {
            selection.push(ContractOutputSelection::Ir);
        }
        if self.additional_values.ir_optimized || self.additional_files.ir_optimized {
            selection.push(ContractOutputSelection::IrOptimized);
        }
        if self.additional_values.metadata || self.additional_files.metadata {
            selection.push(ContractOutputSelection::Metadata);
        }
        if self.additional_values.storage_layout {
            selection.push(ContractOutputSelection::StorageLayout);
        }
        if self.additional_values.devdoc {
            selection.push(ContractOutputSelection::DevDoc);
        }
        if self.additional_values.userdoc {
            selection.push(ContractOutputSelection::UserDoc);
        }
        if self.additional_values.gas_estimates {
            selection.push(EvmOutputSelection::GasEstimates.into());
        }
        if self.additional_values.assembly || self.additional_files.assembly {
            selection.push(EvmOutputSelection::Assembly.into());
        }
        if self.additional_values.ewasm || self.additional_files.ewasm {
            selection.push(EwasmOutputSelection::All.into());
        }
        if self.additional_values.function_debug_data {
            selection.push(BytecodeOutputSelection::FunctionDebugData.into());
        }
        if self.additional_values.method_identifiers {
            selection.push(EvmOutputSelection::MethodIdentifiers.into());
        }
        if self.additional_values.generated_sources {
            selection.push(
                EvmOutputSelection::ByteCode(BytecodeOutputSelection::GeneratedSources).into(),
            );
        }
        if self.additional_values.source_map {
            selection.push(EvmOutputSelection::ByteCode(BytecodeOutputSelection::SourceMap).into());
        }
        selection
    }
}

impl ArtifactOutput for ConfigurableArtifacts {
    type Artifact = ConfigurableContractArtifact;

    fn write_contract_extras(&self, contract: &Contract, file: &Path) -> Result<(), SolcError> {
        self.additional_files.write_extras(contract, file)
    }

    fn contract_to_artifact(
        &self,
        _file: &str,
        _name: &str,
        contract: Contract,
        source_file: Option<&SourceFile>,
    ) -> Self::Artifact {
        let mut artifact_userdoc = None;
        let mut artifact_devdoc = None;
        let mut artifact_raw_metadata = None;
        let mut artifact_metadata = None;
        let mut artifact_ir = None;
        let mut artifact_ir_optimized = None;
        let mut artifact_ewasm = None;
        let mut artifact_bytecode = None;
        let mut artifact_deployed_bytecode = None;
        let mut artifact_gas_estimates = None;
        let mut artifact_function_debug_data = None;
        let mut artifact_method_identifiers = None;
        let mut artifact_assembly = None;
        let mut artifact_storage_layout = None;
        let mut generated_sources = None;
        let mut opcodes = None;

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
            if let Some(LosslessMetadata { raw_metadata, metadata }) = metadata {
                artifact_raw_metadata = Some(raw_metadata);
                artifact_metadata = Some(metadata);
            }
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
            let Evm {
                assembly,
                mut bytecode,
                deployed_bytecode,
                method_identifiers,
                gas_estimates,
                ..
            } = evm;

            if self.additional_values.function_debug_data {
                artifact_function_debug_data =
                    bytecode.as_mut().map(|code| std::mem::take(&mut code.function_debug_data));
            }
            if self.additional_values.generated_sources {
                generated_sources =
                    bytecode.as_mut().map(|code| std::mem::take(&mut code.generated_sources));
            }

            if self.additional_values.opcodes {
                opcodes = bytecode.as_mut().and_then(|code| code.opcodes.take())
            }

            artifact_bytecode = bytecode.map(Into::into);
            artifact_deployed_bytecode = deployed_bytecode.map(Into::into);
            artifact_method_identifiers = Some(method_identifiers);

            if self.additional_values.gas_estimates {
                artifact_gas_estimates = gas_estimates;
            }
            if self.additional_values.assembly {
                artifact_assembly = assembly;
            }
        }

        ConfigurableContractArtifact {
            abi,
            bytecode: artifact_bytecode,
            deployed_bytecode: artifact_deployed_bytecode,
            assembly: artifact_assembly,
            opcodes,
            function_debug_data: artifact_function_debug_data,
            method_identifiers: artifact_method_identifiers,
            gas_estimates: artifact_gas_estimates,
            raw_metadata: artifact_raw_metadata,
            metadata: artifact_metadata,
            storage_layout: artifact_storage_layout,
            userdoc: artifact_userdoc,
            devdoc: artifact_devdoc,
            ir: artifact_ir,
            ir_optimized: artifact_ir_optimized,
            ewasm: artifact_ewasm,
            id: source_file.as_ref().map(|s| s.id),
            ast: source_file.and_then(|s| s.ast.clone()),
            generated_sources: generated_sources.unwrap_or_default(),
        }
    }

    fn standalone_source_file_to_artifact(
        &self,
        _path: &str,
        file: &VersionedSourceFile,
    ) -> Option<Self::Artifact> {
        file.source_file.ast.clone().map(|ast| ConfigurableContractArtifact {
            abi: Some(LosslessAbi::default()),
            id: Some(file.source_file.id),
            ast: Some(ast),
            bytecode: Some(CompactBytecode::empty()),
            deployed_bytecode: Some(CompactDeployedBytecode::empty()),
            ..Default::default()
        })
    }
}

/// Determines the additional values to include in the contract's artifact file
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct ExtraOutputValues {
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
    pub function_debug_data: bool,
    pub generated_sources: bool,
    pub source_map: bool,
    pub opcodes: bool,

    /// PRIVATE: This structure may grow, As such, constructing this structure should
    /// _always_ be done using a public constructor or update syntax:
    ///
    /// ```rust
    /// 
    /// use ethers_solc::ExtraOutputValues;
    /// let config = ExtraOutputValues {
    ///     ir: true,
    ///     ..Default::default()
    /// };
    /// ```
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl ExtraOutputValues {
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
            function_debug_data: true,
            generated_sources: true,
            source_map: true,
            opcodes: true,
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
                        config.generated_sources = true;
                        config.source_map = true;
                        config.opcodes = true;
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
                    EvmOutputSelection::ByteCode(BytecodeOutputSelection::FunctionDebugData) => {
                        config.function_debug_data = true;
                    }
                    EvmOutputSelection::ByteCode(BytecodeOutputSelection::Opcodes) => {
                        config.opcodes = true;
                    }
                    EvmOutputSelection::ByteCode(BytecodeOutputSelection::GeneratedSources) => {
                        config.generated_sources = true;
                    }
                    EvmOutputSelection::ByteCode(BytecodeOutputSelection::SourceMap) => {
                        config.source_map = true;
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
pub struct ExtraOutputFiles {
    pub abi: bool,
    pub metadata: bool,
    pub ir: bool,
    pub ir_optimized: bool,
    pub ewasm: bool,
    pub assembly: bool,
    pub source_map: bool,
    pub generated_sources: bool,
    pub bytecode: bool,
    pub deployed_bytecode: bool,

    /// PRIVATE: This structure may grow, As such, constructing this structure should
    /// _always_ be done using a public constructor or update syntax:
    ///
    /// ```rust
    /// 
    /// use ethers_solc::ExtraOutputFiles;
    /// let config = ExtraOutputFiles {
    ///     metadata: true,
    ///     ..Default::default()
    /// };
    /// ```
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl ExtraOutputFiles {
    /// Returns an instance where all values are set to `true`
    pub fn all() -> Self {
        Self {
            abi: true,
            metadata: true,
            ir: true,
            ir_optimized: true,
            ewasm: true,
            assembly: true,
            source_map: true,
            generated_sources: true,
            bytecode: true,
            deployed_bytecode: true,
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
                ContractOutputSelection::Abi => {
                    config.abi = true;
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
                ContractOutputSelection::Evm(evm) => match evm {
                    EvmOutputSelection::All => {
                        config.assembly = true;
                        config.generated_sources = true;
                        config.source_map = true;
                        config.bytecode = true;
                        config.deployed_bytecode = true;
                    }
                    EvmOutputSelection::Assembly => {
                        config.assembly = true;
                    }
                    EvmOutputSelection::ByteCode(BytecodeOutputSelection::GeneratedSources) => {
                        config.generated_sources = true;
                    }
                    EvmOutputSelection::ByteCode(BytecodeOutputSelection::Object) => {
                        config.bytecode = true;
                    }
                    EvmOutputSelection::ByteCode(BytecodeOutputSelection::SourceMap) => {
                        config.source_map = true;
                    }
                    EvmOutputSelection::DeployedByteCode(DeployedBytecodeOutputSelection::All) |
                    EvmOutputSelection::DeployedByteCode(
                        DeployedBytecodeOutputSelection::Object,
                    ) => {
                        config.deployed_bytecode = true;
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

    /// Write the set values as separate files
    pub fn write_extras(&self, contract: &Contract, file: &Path) -> Result<(), SolcError> {
        if self.abi {
            if let Some(ref abi) = contract.abi {
                let file = file.with_extension("abi.json");
                fs::write(&file, serde_json::to_string_pretty(abi)?)
                    .map_err(|err| SolcError::io(err, file))?
            }
        }

        if self.metadata {
            if let Some(ref metadata) = contract.metadata {
                let file = file.with_extension("metadata.json");
                fs::write(&file, serde_json::to_string_pretty(&metadata.raw_json()?)?)
                    .map_err(|err| SolcError::io(err, file))?
            }
        }

        if self.ir_optimized {
            if let Some(ref iropt) = contract.ir_optimized {
                let file = file.with_extension("iropt");
                fs::write(&file, iropt).map_err(|err| SolcError::io(err, file))?
            }
        }

        if self.ir {
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

        if self.generated_sources {
            if let Some(ref evm) = contract.evm {
                if let Some(ref bytecode) = evm.bytecode {
                    let file = file.with_extension("gensources");
                    fs::write(&file, serde_json::to_vec_pretty(&bytecode.generated_sources)?)
                        .map_err(|err| SolcError::io(err, file))?;
                }
            }
        }

        if self.source_map {
            if let Some(ref evm) = contract.evm {
                if let Some(ref bytecode) = evm.bytecode {
                    if let Some(ref sourcemap) = bytecode.source_map {
                        let file = file.with_extension("sourcemap");
                        fs::write(&file, sourcemap).map_err(|err| SolcError::io(err, file))?
                    }
                }
            }
        }

        if self.bytecode {
            if let Some(ref code) = contract.get_bytecode_bytes() {
                let code = hex::encode(code.as_ref());
                let file = file.with_extension("bin");
                fs::write(&file, code).map_err(|err| SolcError::io(err, file))?
            }
        }
        if self.deployed_bytecode {
            if let Some(ref code) = contract.get_deployed_bytecode_bytes() {
                let code = hex::encode(code.as_ref());
                let file = file.with_extension("deployed-bin");
                fs::write(&file, code).map_err(|err| SolcError::io(err, file))?
            }
        }

        Ok(())
    }
}
