use std::fmt::Debug;

use async_trait::async_trait;
use auto_impl::auto_impl;
use ethers_core::types::U256;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::value::RawValue;

use crate::{ProviderError, RpcError};

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[auto_impl(&, Box, Arc)]
/// Trait which must be implemented by data transports to be used with the Ethereum
/// JSON-RPC provider.
pub trait JsonRpcClient: Debug + Send + Sync {
    /// A JSON-RPC Error
    type Error: Into<ProviderError> + RpcError;

    /// Sends a request with the provided JSON-RPC and parameters serialized as JSON
    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send;
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
