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

#[derive(Clone, Debug)]
/// Mock transport used in test environments.
pub struct MockProvider {
    requests: Arc<Mutex<VecDeque<(String, Value)>>>,
    responses: Arc<Mutex<VecDeque<Value>>>,
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

    /// Pushes the `(method, input)` to the back of the `requests` queue,
    /// pops the responses from the back of the `responses` queue
    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        input: T,
    ) -> Result<R, MockError> {
        self.requests.lock().unwrap().push_back((method.to_owned(), serde_json::to_value(input)?));
        let mut data = self.responses.lock().unwrap();
        let element = data.pop_back().ok_or(MockError::EmptyResponses)?;
        let res: R = serde_json::from_value(element)?;

        Ok(res)
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
        assert_eq!(serde_json::to_value(data).expect("could not serialize data"), inp);
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
        self.responses.lock().unwrap().push_back(value);
        Ok(())
    }
}

#[derive(Error, Debug)]
/// Errors for the `MockProvider`
pub enum MockError {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error("empty responses array, please push some requests")]
    EmptyRequests,

    #[error("empty responses array, please push some responses")]
    EmptyResponses,
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
    use crate::Middleware;
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
