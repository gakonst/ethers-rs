//! Ethereum compatible providers
//! Currently supported:
//! - Raw HTTP POST requests
//!
//! TODO: WebSockets, multiple backends, popular APIs etc.

mod http;
mod provider;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Debug};

pub use provider::Provider;

/// An HTTP provider for interacting with an Ethereum-compatible blockchain
pub type HttpProvider = Provider<http::Provider>;

#[async_trait]
/// Implement this trait in order to plug in different backends
pub trait JsonRpcClient: Debug {
    type Error: Error;

    /// Sends a request with the provided method and the params serialized as JSON
    async fn request<T: Serialize + Send + Sync, R: for<'a> Deserialize<'a>>(
        &self,
        method: &str,
        params: Option<T>,
    ) -> Result<R, Self::Error>;
}
