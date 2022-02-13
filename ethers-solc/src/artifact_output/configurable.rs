//! A configurable artifacts handler implementation

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

/// Determines the additional values to include in the contract's artifact file
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct AdditionalArtifactValues {
    pub ast: bool,
    pub userdoc: bool,
    pub devdoc: bool,
    pub hashes: bool,
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
    pub evm_assembly: bool,

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
