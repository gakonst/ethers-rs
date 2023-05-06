use crate::{JsonRpcClient, ProviderError};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::{
    borrow::Borrow,
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use thiserror::Error;

/// Helper type that can be used to pass through the `params` value.
/// This is necessary because the wrapper provider is supposed to skip the `params` if it's of
/// size 0, see `crate::transports::common::Request`
#[derive(Debug)]
enum MockParams {
    Value(Value),
    Zst,
}

/// Helper response type for `MockProvider`, allowing custom JSON-RPC errors to be provided.
/// `Value` for successful responses, `Error` for JSON-RPC errors.
#[derive(Clone, Debug)]
pub enum MockResponse {
    /// Successful response with a `serde_json::Value`.
    Value(Value),

    /// Error response with a `JsonRpcError`.
    Error(super::JsonRpcError),
}

#[derive(Clone, Debug)]
/// Mock transport used in test environments.
pub struct MockProvider {
    requests: Arc<Mutex<VecDeque<(String, MockParams)>>>,
    responses: Arc<Mutex<VecDeque<MockResponse>>>,
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl JsonRpcClient for MockProvider {
    type Error = MockError;

    /// Pushes the `(method, params)` to the back of the `requests` queue,
    /// pops the responses from the back of the `responses` queue
    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, MockError> {
        let params = if std::mem::size_of::<T>() == 0 {
            MockParams::Zst
        } else {
            MockParams::Value(serde_json::to_value(params)?)
        };
        self.requests.lock().unwrap().push_back((method.to_owned(), params));
        let mut data = self.responses.lock().unwrap();
        let element = data.pop_back().ok_or(MockError::EmptyResponses)?;
        match element {
            MockResponse::Value(value) => {
                let res: R = serde_json::from_value(value)?;
                Ok(res)
            }
            MockResponse::Error(error) => Err(MockError::JsonRpcError(error)),
        }
    }
}

impl MockProvider {
    /// Checks that the provided request was submitted by the client
    pub fn assert_request<T: Serialize + Send + Sync>(
        &self,
        method: &str,
        data: T,
    ) -> Result<(), MockError> {
        let (m, inp) = self.requests.lock().unwrap().pop_front().ok_or(MockError::EmptyRequests)?;
        assert_eq!(m, method);
        assert!(!matches!(inp, MockParams::Value(serde_json::Value::Null)));
        if std::mem::size_of::<T>() == 0 {
            assert!(matches!(inp, MockParams::Zst));
        } else if let MockParams::Value(inp) = inp {
            assert_eq!(serde_json::to_value(data).expect("could not serialize data"), inp);
        } else {
            unreachable!("Zero sized types must be denoted with MockParams::Zst")
        }

        Ok(())
    }

    /// Instantiates a mock transport
    pub fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(VecDeque::new())),
            responses: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Pushes the data to the responses
    pub fn push<T: Serialize + Send + Sync, K: Borrow<T>>(&self, data: K) -> Result<(), MockError> {
        let value = serde_json::to_value(data.borrow())?;
        self.responses.lock().unwrap().push_back(MockResponse::Value(value));
        Ok(())
    }

    /// Pushes the data or error to the responses
    pub fn push_response(&self, response: MockResponse) {
        self.responses.lock().unwrap().push_back(response);
    }
}

#[derive(Error, Debug)]
/// Errors for the `MockProvider`
pub enum MockError {
    /// (De)Serialization error
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// Empty requests array
    #[error("empty requests array, please push some requests")]
    EmptyRequests,

    /// Empty responses array
    #[error("empty responses array, please push some responses")]
    EmptyResponses,

    /// Custom JsonRpcError
    #[error("JSON-RPC error: {0}")]
    JsonRpcError(super::JsonRpcError),
}

impl crate::RpcError for MockError {
    fn as_error_response(&self) -> Option<&super::JsonRpcError> {
        match self {
            MockError::JsonRpcError(e) => Some(e),
            _ => None,
        }
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        match self {
            MockError::SerdeJson(e) => Some(e),
            _ => None,
        }
    }
}

impl From<MockError> for ProviderError {
    fn from(src: MockError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use crate::{JsonRpcError, Middleware};
    use ethers_core::types::U64;

    #[tokio::test]
    async fn pushes_request_and_response() {
        let mock = MockProvider::new();
        mock.push(U64::from(12)).unwrap();
        let block: U64 = mock.request("eth_blockNumber", ()).await.unwrap();
        mock.assert_request("eth_blockNumber", ()).unwrap();
        assert_eq!(block.as_u64(), 12);
    }

    #[tokio::test]
    async fn empty_responses() {
        let mock = MockProvider::new();
        // tries to get a response without pushing a response
        let err = mock.request::<_, ()>("eth_blockNumber", ()).await.unwrap_err();
        match err {
            MockError::EmptyResponses => {}
            _ => panic!("expected empty responses"),
        };
    }

    #[tokio::test]
    async fn pushes_error_response() {
        let mock = MockProvider::new();
        let error = JsonRpcError {
            code: 3,
            data: Some(serde_json::from_str(r#""0x556f1830...""#).unwrap()),
            message: "execution reverted".to_string(),
        };
        mock.push_response(MockResponse::Error(error.clone()));

        let result: Result<U64, MockError> = mock.request("eth_blockNumber", ()).await;
        match result {
            Err(MockError::JsonRpcError(e)) => {
                assert_eq!(e.code, error.code);
                assert_eq!(e.message, error.message);
                assert_eq!(e.data, error.data);
            }
            _ => panic!("Expected JsonRpcError"),
        }
    }

    #[tokio::test]
    async fn empty_requests() {
        let mock = MockProvider::new();
        // tries to assert a request without making one
        let err = mock.assert_request("eth_blockNumber", ()).unwrap_err();
        match err {
            MockError::EmptyRequests => {}
            _ => panic!("expected empty request"),
        };
    }

    #[tokio::test]
    async fn composes_with_provider() {
        let (provider, mock) = crate::Provider::mocked();

        mock.push(U64::from(12)).unwrap();
        let block = provider.get_block_number().await.unwrap();
        assert_eq!(block.as_u64(), 12);
    }
}
