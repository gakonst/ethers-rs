//! Boilerplate error definitions.
use thiserror::Error;

/// A type alias for std's Result with the Error as our error type.
pub type Result<T, E = ParseError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    ParseError(#[from] super::Error),
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
pub(crate) use _bail as bail;
