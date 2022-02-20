use std::{io, path::PathBuf};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CompilerError>;

/// Various error types
#[derive(Debug, Error)]
pub enum CompilerError {
    /// Internal compiler error
    #[error("Compiler Error: {0}")]
    CompilerError(String),
    #[error("Missing pragma from solidity file")]
    PragmaNotFound,
    #[error("Could not find compiler version locally or upstream")]
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
    Io(#[from] CompilerIoError),
    /// Failed to resolve a file
    #[error("Failed to resolve file: {0}.\n Check configured remappings.")]
    Resolve(CompilerIoError),
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

    #[error("No artifact found for `{}:{}`", .0.display(), .1)]
    ArtifactNotFound(PathBuf, String),

    #[cfg(feature = "project-util")]
    #[error(transparent)]
    FsExtra(#[from] fs_extra::error::Error),
}

impl CompilerError {
    pub(crate) fn io(err: io::Error, path: impl Into<PathBuf>) -> Self {
        CompilerIoError::new(err, path).into()
    }
    pub(crate) fn solc(msg: impl Into<String>) -> Self {
        CompilerError::CompilerError(msg.into())
    }
    pub fn msg(msg: impl Into<String>) -> Self {
        CompilerError::Message(msg.into())
    }
}

#[derive(Debug, Error)]
#[error("\"{}\": {io}", self.path.display())]
pub struct CompilerIoError {
    io: io::Error,
    path: PathBuf,
}

impl CompilerIoError {
    pub fn new(io: io::Error, path: impl Into<PathBuf>) -> Self {
        Self { io, path: path.into() }
    }
}

impl From<CompilerIoError> for io::Error {
    fn from(err: CompilerIoError) -> Self {
        err.io
    }
}
