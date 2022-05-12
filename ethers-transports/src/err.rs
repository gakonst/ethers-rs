use std::{error, fmt};

use crate::{
    jsonrpc::JsonRpcError,
    provider::{ErrorKind, ProviderError},
};

#[derive(Debug)]
pub enum TransportError {
    /// An error originating from the specific underlying transport
    /// implementation.
    ///
    /// Refer to the respective transport's module documentation to infer to
    /// which specific error type this can be downcast to for each transport.
    Transport(Box<dyn error::Error + Send + Sync>),
    /// An error indicating that some response from the underlying transport
    /// implementation could not be parsed as valid JSON.
    Json { input: String, source: serde_json::Error },
    /// An error originating from the JSON-RPC API provider indicating some kind
    /// of incorrect usage of the specified API.
    JsonRpc(JsonRpcError),
}

impl TransportError {
    pub(crate) fn transport(err: impl Into<Box<dyn error::Error + Send + Sync>>) -> Box<Self> {
        Box::new(Self::Transport(err.into()))
    }

    pub(crate) fn json(input: &str, source: serde_json::Error) -> Box<Self> {
        Box::new(Self::Json { input: input.to_string(), source })
    }

    pub(crate) fn jsonrpc(err: JsonRpcError) -> Box<Self> {
        Box::new(Self::JsonRpc(err))
    }

    pub(crate) fn to_provider_err(self: Box<Self>) -> Box<ProviderError> {
        Box::new(ProviderError { kind: ErrorKind::Transport(self), context: "".into() })
    }
}

impl error::Error for TransportError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Json { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(err) => write!(f, "{err}"),
            Self::Json { input, .. } => write!(f, "failed to parse JSON from input ({input})"),
            Self::JsonRpc(err) => write!(f, "{err}"),
        }
    }
}
