//! Boilerplate error definitions.
use crate::abi::{human_readable, InvalidOutputType};
use thiserror::Error;

/// A type alias for std's Result with the Error as our error type.
pub type Result<T, E = ParseError> = std::result::Result<T, E>;

/// Error that can occur during human readable parsing
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    Message(String),
    // ethabi parser error
    #[error(transparent)]
    ParseError(#[from] ethabi::Error),
    // errors from human readable lexer
    #[error(transparent)]
    LexerError(#[from] human_readable::lexer::LexerError),
}

macro_rules! _format_err {
    ($($tt:tt)*) => {
        $crate::abi::ParseError::Message(format!($($tt)*))
    };
}
pub(crate) use _format_err as format_err;

macro_rules! _bail {
    ($($tt:tt)*) => { return Err($crate::abi::error::format_err!($($tt)*)) };
}
use crate::types::ParseBytesError;
pub(crate) use _bail as bail;

/// ABI codec related errors
#[derive(Error, Debug)]
pub enum AbiError {
    /// Thrown when the ABI decoding fails
    #[error(transparent)]
    DecodingError(#[from] ethabi::Error),

    /// Thrown when detokenizing an argument
    #[error(transparent)]
    DetokenizationError(#[from] InvalidOutputType),

    #[error("missing or wrong function selector")]
    WrongSelector,

    #[error(transparent)]
    ParseBytesError(#[from] ParseBytesError),
}
