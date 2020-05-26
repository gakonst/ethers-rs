//! Minimal HTTP JSON-RPC 2.0 Client
//! The request/response code is taken from [here](https://github.com/althea-net/guac_rs/blob/master/web3/src/jsonrpc)
use crate::JsonRpcClient;

use async_trait::async_trait;
use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};
use thiserror::Error;
use url::Url;

/// An HTTP Client
#[derive(Debug)]
pub struct Provider {
    id: AtomicU64,
    client: Client,
    url: Url,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error(transparent)]
    ReqwestError(#[from] ReqwestError),
    #[error(transparent)]
    JsonRpcError(#[from] JsonRpcError),
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
    pub fn new(url: impl Into<Url>) -> Self {
        Self {
            id: AtomicU64::new(0),
            client: Client::new(),
            url: url.into(),
        }
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
    Error { error: JsonRpcError },
    Success { result: R },
}

impl<R> ResponseData<R> {
    /// Consume response and return value
    fn into_result(self) -> Result<R, JsonRpcError> {
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
