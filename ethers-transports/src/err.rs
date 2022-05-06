use std::error;

use crate::jsonrpc::JsonRpcError;

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
}
