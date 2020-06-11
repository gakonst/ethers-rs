use crate::{provider::ProviderError, JsonRpcClient};

use async_trait::async_trait;
use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fmt,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};
use thiserror::Error;
use url::Url;

/// A low-level JSON-RPC Client over HTTP.
///
/// # Example
///
/// ```no_run
/// use ethers_providers::{JsonRpcClient, Http};
/// use ethers_core::types::U64;
/// use std::str::FromStr;
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Http::from_str("http://localhost:8545")?;
/// let block_number: U64 = provider.request("eth_blockNumber", None::<()>).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Provider {
    id: AtomicU64,
    client: Client,
    url: Url,
}

#[derive(Error, Debug)]
/// Error thrown when sending an HTTP request
pub enum ClientError {
    /// Thrown if the request failed
    #[error(transparent)]
    ReqwestError(#[from] ReqwestError),
    #[error(transparent)]
    /// Thrown if the response could not be parsed
    JsonRpcError(#[from] errors::JsonRpcError),
}

impl From<ClientError> for ProviderError {
    fn from(src: ClientError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

#[async_trait]
impl JsonRpcClient for Provider {
    type Error = ClientError;

    /// Sends a POST request with the provided method and the params serialized as JSON
    /// over HTTP
    async fn request<T: Serialize + Send + Sync, R: for<'a> Deserialize<'a>>(
        &self,
        method: &str,
        params: Option<T>,
    ) -> Result<R, ClientError> {
        let next_id = self.id.load(Ordering::SeqCst) + 1;
        self.id.store(next_id, Ordering::SeqCst);

        let payload = Request::new(next_id, method, params);

        let res = self
            .client
            .post(self.url.as_ref())
            .json(&payload)
            .send()
            .await?;
        let res = res.json::<Response<R>>().await?;

        Ok(res.data.into_result()?)
    }
}

impl Provider {
    /// Initializes a new HTTP Client
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_providers::Http;
    /// use url::Url;
    ///
    /// let url = Url::parse("http://localhost:8545").unwrap();
    /// let provider = Http::new(url);
    /// ```
    pub fn new(url: impl Into<Url>) -> Self {
        Self {
            id: AtomicU64::new(0),
            client: Client::new(),
            url: url.into(),
        }
    }
}

impl FromStr for Provider {
    type Err = url::ParseError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(src)?;
        Ok(Provider::new(url))
    }
}

impl Clone for Provider {
    fn clone(&self) -> Self {
        Self {
            id: AtomicU64::new(0),
            client: self.client.clone(),
            url: self.url.clone(),
        }
    }
}

// leak private type w/o exposing it
mod errors {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone, Error)]
    /// A JSON-RPC 2.0 error
    pub struct JsonRpcError {
        /// The error code
        pub code: i64,
        /// The error message
        pub message: String,
        /// Additional data
        pub data: Option<Value>,
    }

    impl fmt::Display for JsonRpcError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "(code: {}, message: {}, data: {:?})",
                self.code, self.message, self.data
            )
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
/// A JSON-RPC request
struct Request<'a, T> {
    id: u64,
    jsonrpc: &'a str,
    method: &'a str,
    params: Option<T>,
}

impl<'a, T> Request<'a, T> {
    /// Creates a new JSON RPC request
    fn new(id: u64, method: &'a str, params: Option<T>) -> Self {
        Self {
            id,
            jsonrpc: "2.0",
            method,
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Response<T> {
    id: u64,
    jsonrpc: String,
    #[serde(flatten)]
    data: ResponseData<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum ResponseData<R> {
    Error { error: errors::JsonRpcError },
    Success { result: R },
}

impl<R> ResponseData<R> {
    /// Consume response and return value
    fn into_result(self) -> Result<R, errors::JsonRpcError> {
        match self {
            ResponseData::Success { result } => Ok(result),
            ResponseData::Error { error } => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response() {
        let response: Response<u64> =
            serde_json::from_str(r#"{"jsonrpc": "2.0", "result": 19, "id": 1}"#).unwrap();
        assert_eq!(response.id, 1);
        assert_eq!(response.data.into_result().unwrap(), 19);
    }
}
