use thiserror::Error;

pub type Result<T> = std::result::Result<T, SolcError>;

/// Various error types
#[derive(Debug, Error)]
pub enum SolcError {
    /// Internal solc error
    #[error("Solc Error: {0}")]
    SolcError(String),
    #[error(transparent)]
    SemverError(#[from] semver::Error),
    /// Deserialization error
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    /// Deserialization error
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl SolcError {
    pub(crate) fn solc(msg: impl Into<String>) -> Self {
        SolcError::SolcError(msg.into())
    }
}
