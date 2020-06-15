mod http;
pub use http::Provider as Http;

mod provider;

// ENS support
mod ens;

mod stream;
pub use stream::FilterStream;
// re-export `StreamExt` so that consumers can call `next()` on the `FilterStream`
// without having to import futures themselves
pub use futures_util::StreamExt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Debug};

pub use provider::{Provider, ProviderError};

#[async_trait]
/// Trait which must be implemented by data transports to be used with the Ethereum
/// JSON-RPC provider.
pub trait JsonRpcClient: Debug + Clone {
    /// A JSON-RPC Error
    type Error: Error + Into<ProviderError>;

    /// Sends a request with the provided JSON-RPC and parameters serialized as JSON
    async fn request<T, R>(&self, method: &str, params: Option<T>) -> Result<R, Self::Error>
    where
        T: Serialize + Send + Sync,
        R: for<'a> Deserialize<'a>;
}
