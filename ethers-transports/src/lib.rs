#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(clippy::type_complexity)]
#![doc = include_str!("../README.md")]

mod common;
pub use common::Authorization;

// only used with WS
#[cfg(feature = "ws")]
macro_rules! if_wasm {
    ($($item:item)*) => {$(
        #[cfg(target_arch = "wasm32")]
        $item
    )*}
}

// only used with WS
#[cfg(feature = "ws")]
macro_rules! if_not_wasm {
    ($($item:item)*) => {$(
        #[cfg(not(target_arch = "wasm32"))]
        $item
    )*}
}

#[cfg(all(target_family = "unix", feature = "ipc"))]
mod ipc;
use ethers_core::types::U256;
#[cfg(all(target_family = "unix", feature = "ipc"))]
pub use ipc::{Ipc, IpcError};
use serde_json::value::RawValue;

mod http;
pub use self::http::{ClientError as HttpClientError, Provider as Http};

#[cfg(feature = "ws")]
mod ws;
#[cfg(feature = "ws")]
pub use ws::{ClientError as WsClientError, Ws};

mod quorum;
pub use quorum::{JsonRpcClientWrapper, Quorum, QuorumError, QuorumProvider, WeightedProvider};

mod rw;
pub use rw::{RwClient, RwClientError};

mod mock;
pub use mock::{MockError, MockProvider};

use async_trait::async_trait;
use auto_impl::auto_impl;
use thiserror::Error;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[auto_impl(&, Box, Arc)]
/// Trait which must be implemented by data transports to be used with the Ethereum
/// JSON-RPC provider.
pub trait JsonRpcClient: std::fmt::Debug + Send + Sync {
    /// A JSON-RPC Error
    type Error: std::error::Error + Into<ProviderError>;

    /// Sends a request with the provided JSON-RPC and parameters serialized as JSON
    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: std::fmt::Debug + serde::Serialize + Send + Sync,
        R: serde::de::DeserializeOwned;
}

/// A transport implementation supporting pub sub subscriptions.
pub trait PubsubClient: JsonRpcClient {
    /// The type of stream this transport returns
    type NotificationStream: futures_core::Stream<Item = Box<RawValue>> + Send + Unpin;

    /// Add a subscription to this transport
    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, Self::Error>;

    /// Remove a subscription from this transport
    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), Self::Error>;
}

#[derive(Debug, Error)]
/// An error thrown when making a call to the provider
pub enum ProviderError {
    /// An internal error in the JSON RPC Client
    #[error(transparent)]
    JsonRpcClientError(#[from] Box<dyn std::error::Error + Send + Sync>),

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
