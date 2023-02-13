use std::{error::Error, fmt::Debug};
use thiserror::Error;

use crate::JsonRpcError;

pub trait RpcError: Error + Debug + Send + Sync {
    fn as_error_response(&self) -> Option<&JsonRpcError>;
}

pub trait MiddlewareError: Error + Sized + Send + Sync {
    type Inner: MiddlewareError;

    fn from_err(e: Self::Inner) -> Self;

    fn as_inner(&self) -> Option<&Self::Inner>;

    fn as_provider_error(&self) -> Option<&ProviderError> {
        self.as_inner()?.as_provider_error()
    }

    fn from_provider_err(p: ProviderError) -> Self {
        Self::from_err(Self::Inner::from_provider_err(p))
    }

    fn as_error_response(&self) -> Option<&JsonRpcError> {
        MiddlewareError::as_error_response(self.as_inner()?)
    }
}

#[derive(Debug, Error)]
/// An error thrown when making a call to the provider
pub enum ProviderError {
    /// An internal error in the JSON RPC Client
    #[error("{0}")]
    JsonRpcClientError(Box<dyn crate::RpcError + Send + Sync>),

    /// An error during ENS name resolution
    #[error("ens name not found: {0}")]
    EnsError(String),

    /// Invalid reverse ENS name
    #[error("reverse ens name not pointing to itself: {0}")]
    EnsNotOwned(String),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    HexError(#[from] hex::FromHexError),

    #[error(transparent)]
    HTTPError(#[from] reqwest::Error),

    #[error("custom error: {0}")]
    CustomError(String),

    #[error("unsupported RPC")]
    UnsupportedRPC,

    #[error("unsupported node client")]
    UnsupportedNodeClient,

    #[error("Attempted to sign a transaction with no available signer. Hint: did you mean to use a SignerMiddleware?")]
    SignerUnavailable,
}

impl RpcError for ProviderError {
    fn as_error_response(&self) -> Option<&super::JsonRpcError> {
        if let ProviderError::JsonRpcClientError(err) = self {
            err.as_error_response()
        } else {
            None
        }
    }
}

impl MiddlewareError for ProviderError {
    type Inner = Self;

    fn as_error_response(&self) -> Option<&super::JsonRpcError> {
        RpcError::as_error_response(self)
    }

    fn from_err(e: Self::Inner) -> Self {
        e
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        None
    }
}
