use std::{error::Error, fmt::Debug};
use thiserror::Error;

use crate::JsonRpcError;

/// An `RpcError` is an abstraction over error types returned by a
/// [`crate::JsonRpcClient`].
///
/// All clients can return [`JsonRpcError`] responses, as
/// well as serde deserialization errors. However, because client errors are
/// typically type-erased via the [`ProviderError`], the error info can be
/// difficult to access. This trait provides convenient access to the
/// underlying error types.
///
/// This trait deals only with behavior that is common to all clients.
/// Client-specific errorvariants cannot be accessed via this trait.
pub trait RpcError: Error + Debug + Send + Sync {
    /// Access an underlying JSON-RPC error (if any)
    ///
    /// Attempts to access an underlying [`JsonRpcError`]. If the underlying
    /// error is not a JSON-RPC error response, this function will return
    /// `None`.
    fn as_error_response(&self) -> Option<&JsonRpcError>;

    /// Returns `true` if the underlying error is a JSON-RPC error response
    fn is_error_response(&self) -> bool {
        self.as_error_response().is_some()
    }

    /// Access an underlying `serde_json` error (if any)
    ///
    /// Attempts to access an underlying [`serde_json::Error`]. If the
    /// underlying error is not a serde_json error, this function will return
    /// `None`.
    ///
    /// ### Implementor's Note
    ///
    /// When writing a stacked [`crate::JsonRpcClient`] abstraction (e.g. a quorum
    /// provider or retrying provider), be sure to account for `serde_json`
    /// errors at your layer, as well as at lower layers.
    fn as_serde_error(&self) -> Option<&serde_json::Error>;

    /// Returns `true` if the underlying error is a serde_json (de)serialization
    /// error. This method can be used to identify
    fn is_serde_error(&self) -> bool {
        self.as_serde_error().is_some()
    }
}

/// [`MiddlewareError`] is a companion trait to [`crate::Middleware`]. It
/// describes error behavior that is common to all Middleware errors.
///
/// Like [`crate::Middleware`], it allows moving down through layered errors.
///
/// Like [`RpcError`] it exposes convenient accessors to useful underlying
/// error information.
///
///
/// ## Not to Devs:
/// While this trait includes the same methods as [`RpcError`], it is not a
/// supertrait. This is so that 3rd party developers do not need to learn and
/// implement both traits. We provide default methods that delegate to inner
/// middleware errors on the assumption that it will eventually reach a
/// [`ProviderError`], which has correct behavior. This allows Middleware devs
/// to ignore the methods' presence if they want. Middleware are already plenty
/// complicated and we don't need to make it worse :)
pub trait MiddlewareError: Error + Sized + Send + Sync {
    /// The `Inner` type is the next lower middleware layer's error type.
    type Inner: MiddlewareError;

    /// Convert the next lower middleware layer's error to this layer's error
    fn from_err(e: Self::Inner) -> Self;

    /// Attempt to convert this error to the next lower middleware's error.
    /// Conversion fails if the error is not from an inner layer (i.e. the
    /// error originates at this middleware layer)
    fn as_inner(&self) -> Option<&Self::Inner>;

    /// Returns `true` if the underlying error stems from a lower middleware
    /// layer
    fn is_inner(&self) -> bool {
        self.as_inner().is_some()
    }

    /// Access an underlying `serde_json` error (if any)
    ///
    /// Attempts to access an underlying [`serde_json::Error`]. If the
    /// underlying error is not a serde_json error, this function will return
    /// `None`.
    ///
    /// ### Implementor's Note:
    ///
    /// When writing a custom middleware, if your middleware uses `serde_json`
    /// we recommend a custom implementation of this method. It should first
    /// check your Middleware's error for local `serde_json` errors, and then
    /// delegate to inner if none is found. Failing to implement this method may
    /// result in missed `serde_json` errors.
    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        self.as_inner()?.as_serde_error()
    }

    /// Returns `true` if the underlying error is a serde_json (de)serialization
    /// error. This method can be used to identify
    fn is_serde_error(&self) -> bool {
        self.as_serde_error().is_some()
    }

    /// Attempts to access an underlying [`ProviderError`], usually by
    /// traversing the entire middleware stack. Access fails if the underlying
    /// error is not a [`ProviderError`]
    fn as_provider_error(&self) -> Option<&ProviderError> {
        self.as_inner()?.as_provider_error()
    }

    /// Convert a [`ProviderError`] to this type, by successively wrapping it
    /// in the error types of all lower middleware
    fn from_provider_err(p: ProviderError) -> Self {
        Self::from_err(Self::Inner::from_provider_err(p))
    }

    /// Access an underlying JSON-RPC error (if any)
    ///
    /// Attempts to access an underlying [`JsonRpcError`]. If the underlying
    /// error is not a JSON-RPC error response, this function will return
    /// `None`.
    fn as_error_response(&self) -> Option<&JsonRpcError> {
        self.as_inner()?.as_error_response()
    }

    /// Returns `true` if the underlying error is a JSON-RPC error response
    fn is_error_response(&self) -> bool {
        self.as_error_response().is_some()
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

    /// Error in underlying lib `serde_json`
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// Error in underlying lib `hex`
    #[error(transparent)]
    HexError(#[from] hex::FromHexError),

    /// Error in underlying lib `reqwest`
    #[error(transparent)]
    HTTPError(#[from] reqwest::Error),

    /// Custom error from unknown source
    #[error("custom error: {0}")]
    CustomError(String),

    /// RPC method is not supported by this provider
    #[error("unsupported RPC")]
    UnsupportedRPC,

    /// Node is not supported by this provider
    #[error("unsupported node client")]
    UnsupportedNodeClient,

    /// Signer is not available to this provider.
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

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        match self {
            ProviderError::JsonRpcClientError(e) => e.as_serde_error(),
            ProviderError::SerdeJson(e) => Some(e),
            _ => None,
        }
    }
}

// Do not change these implementations, they are critical to proper middleware
// error stack behavior.
impl MiddlewareError for ProviderError {
    type Inner = Self;

    fn as_error_response(&self) -> Option<&super::JsonRpcError> {
        RpcError::as_error_response(self)
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        RpcError::as_serde_error(self)
    }

    fn from_err(e: Self::Inner) -> Self {
        e
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        // prevents infinite loops
        None
    }
}
