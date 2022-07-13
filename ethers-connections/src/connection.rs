#[cfg(feature = "http")]
pub mod http;
#[cfg(all(unix, feature = "ipc"))]
pub mod ipc;
#[cfg(feature = "ws")]
pub mod ws;

pub mod noop;

#[cfg(any(all(unix, feature = "ipc"), feature = "ws"))]
mod common;

use std::{error, fmt};

use crate::{
    jsonrpc::JsonRpcError,
    provider::{ErrorKind, ProviderError},
};

/// An error that occurred while exchanging requests and responses over a
/// [`Connection`].
#[derive(Debug)]
pub enum ConnectionError {
    Connection(Box<dyn error::Error + Send + Sync + 'static>),
    Json { input: Box<str>, source: serde_json::Error },
    JsonRpc(JsonRpcError),
}

impl ConnectionError {
    pub(crate) fn connection(err: impl Into<Box<dyn error::Error + Send + Sync>>) -> Self {
        Self::Connection(err.into())
    }

    pub(crate) fn json(input: &str, source: serde_json::Error) -> Self {
        Self::Json { input: input.into(), source }
    }

    pub(crate) fn jsonrpc(err: JsonRpcError) -> Self {
        Self::JsonRpc(err)
    }

    pub(crate) fn to_provider_err(self) -> Box<ProviderError> {
        Box::new(ProviderError { kind: ErrorKind::Connection(self), context: "".into() })
    }
}

impl error::Error for ConnectionError {}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(err) => err.fmt(f),
            Self::Json { input, .. } => write!(f, "failed to parse JSON from input ({input})"),
            Self::JsonRpc(err) => err.fmt(f),
        }
    }
}
