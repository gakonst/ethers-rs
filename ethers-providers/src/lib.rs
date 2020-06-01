pub mod http;
pub use http::Provider as Http;
mod provider;

// ENS support
mod ens;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Debug};

pub use provider::{Provider, ProviderError};

/// An HTTP provider for interacting with an Ethereum-compatible blockchain
pub type HttpProvider = Provider<Http>;

#[async_trait]
/// Trait which must be implemented by data transports to be used with the Ethereum
/// JSON-RPC provider.
pub trait JsonRpcClient: Debug {
    /// A JSON-RPC Error
    type Error: Error + Into<ProviderError>;

    /// Sends a request with the provided JSON-RPC and parameters serialized as JSON
    async fn request<T: Serialize + Send + Sync, R: for<'a> Deserialize<'a>>(
        &self,
        method: &str,
        params: Option<T>,
    ) -> Result<R, Self::Error>;
}
