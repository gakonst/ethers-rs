use std::{io, path::PathBuf};
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
    Io(#[from] SolcIoError),
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
    pub(crate) fn io(err: io::Error, path: impl Into<PathBuf>) -> Self {
        SolcIoError::new(err, path).into()
    }
    pub(crate) fn solc(msg: impl Into<String>) -> Self {
        SolcError::SolcError(msg.into())
    }
    pub(crate) fn msg(msg: impl Into<String>) -> Self {
        SolcError::Message(msg.into())
    }
}

#[derive(Debug, Error)]
#[error("\"{}\": {io}", self.path.display())]
pub struct SolcIoError {
    io: io::Error,
    path: PathBuf,
}

impl SolcIoError {
    pub fn new(io: io::Error, path: impl Into<PathBuf>) -> Self {
        Self { io, path: path.into() }
    }
}

impl From<SolcIoError> for io::Error {
    fn from(err: SolcIoError) -> Self {
        err.io
    }
}
