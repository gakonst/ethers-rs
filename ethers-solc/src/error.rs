use thiserror::Error;

pub type Result<T> = std::result::Result<T, SolcError>;

/// Various error types
#[derive(Debug, Error)]
pub enum SolcError {
    /// Internal solc error
    #[error("Solc Error: {0}")]
    SolcError(String),
    #[error("Missing pragma from solidity file")]
    PragmaNotFound,
    #[error("Could not find solc version locally or upstream")]
    VersionNotFound,
    #[error("Checksum mismatch")]
    ChecksumMismatch,
    #[error(transparent)]
    SemverError(#[from] semver::Error),
    /// Deserialization error
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    /// Filesystem IO error
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(feature = "svm")]
    #[error(transparent)]
    SvmError(#[from] svm::SolcVmError),
    #[error("No contracts found at \"{0}\"")]
    NoContracts(String),
    #[error(transparent)]
    PatternError(#[from] glob::PatternError),
    /// General purpose message
    #[error("{0}")]
    Message(String),
}

impl SolcError {
    pub(crate) fn solc(msg: impl Into<String>) -> Self {
        SolcError::SolcError(msg.into())
    }
    pub(crate) fn msg(msg: impl Into<String>) -> Self {
        SolcError::Message(msg.into())
    }
}
