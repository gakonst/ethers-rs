// Code adapted from: https://github.com/althea-net/guac_rs/tree/master/web3/src/jsonrpc
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use thiserror::Error;

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
pub struct Request<'a, T> {
    id: u64,
    jsonrpc: &'a str,
    method: &'a str,
    params: T,
}

impl<'a, T> Request<'a, T> {
    /// Creates a new JSON RPC request
    pub fn new(id: u64, method: &'a str, params: T) -> Self {
        Self {
            id,
            jsonrpc: "2.0",
            method,
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response<T> {
    id: u64,
    jsonrpc: String,
    #[serde(flatten)]
    pub data: ResponseData<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ResponseData<R> {
    Error { error: JsonRpcError },
    Success { result: R },
}

impl<R> ResponseData<R> {
    /// Consume response and return value
    pub fn into_result(self) -> Result<R, JsonRpcError> {
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
