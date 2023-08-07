use crate::{JsonRpcClient, ProviderError, PubsubClient};
use async_trait::async_trait;
use ethers_core::types::U256;
use futures_channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{value::RawValue, Value};
use std::{
    borrow::Borrow,
    collections::{HashMap, VecDeque},
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
    current_stream_handle: Arc<Mutex<U256>>,
    stream_handles: Arc<
        Mutex<
            HashMap<
                U256,
                (UnboundedSender<Box<RawValue>>, Arc<Mutex<UnboundedReceiver<Box<RawValue>>>>),
            >,
        >,
    >,
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

impl PubsubClient for MockProvider {
    type NotificationStream = mpsc::UnboundedReceiver<Box<RawValue>>;
    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, Self::Error> {
        let (mut stream_handle, sink_handle) = mpsc::unbounded::<Box<RawValue>>();

        let stream_handles = self.stream_handles.lock().unwrap();
        let (_, receiver) = stream_handles.get(&id.into()).unwrap().clone();

        // Spawn a task that forwards items from a mock stream to the subscription stream
        tokio::task::spawn(async move {
            let mut receiver_clone = receiver.lock().unwrap();
            while let Ok(Some(x)) = receiver_clone.try_next() {
                // This should always succeed
                stream_handle.start_send(x).unwrap();
            }
        });

        Ok(sink_handle)
    }

    fn unsubscribe<T: Into<U256>>(&self, _id: T) -> Result<(), Self::Error> {
        Ok(())
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
            stream_handles: Arc::new(Mutex::new(HashMap::new())),
            current_stream_handle: Arc::new(Mutex::new(0.into())),
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

    fn init_stream(&mut self) -> U256 {
        let (stream_handle, sink_handle) = mpsc::unbounded::<Box<RawValue>>();

        let mutex_sink = Arc::new(Mutex::new(sink_handle));
        let mut current_stream_handle = self.current_stream_handle.lock().unwrap();
        *current_stream_handle += 1.into();

        let mut mock_stream_handles = self.stream_handles.lock().unwrap();

        mock_stream_handles.insert(*current_stream_handle, (stream_handle, mutex_sink));

        *current_stream_handle
    }

    async fn drain_sync_queue_to_stream(&mut self, stream_id: U256) -> Result<(), MockError> {
        let stream_handles = self.stream_handles.lock().unwrap();

        let stream = stream_handles.get(&stream_id);
        assert!(stream.is_some());
        let (mut stream, _) = stream.unwrap().clone();

        loop {
            // T is a dummy type to pass the type check
            match self.request::<[u64; 0], Box<RawValue>>("", []).await {
                Ok(value) => {
                    stream.start_send(value).unwrap();
                }
                Err(MockError::EmptyResponses) => {
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    /// Sets up a subscription stream, draining all of the currently pushed values
    /// into the stream
    pub async fn setup_subscription(&mut self) -> Result<(), MockError> {
        // Initialize a mock stream
        let stream_id = self.init_stream();

        // drain the current present mock data into a stream that can be subscribed to
        self.drain_sync_queue_to_stream(stream_id).await?;

        // Push the subscription id to the responses queue
        // Need to do this because the JSONRPC subscription request returns an subscription id
        self.push(stream_id).unwrap();

        Ok(())
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
    use crate::{JsonRpcError, Middleware, Provider};
    use ethers_core::types::{Transaction, U64};
    use futures_util::StreamExt;

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

    #[tokio::test]
    async fn provider_allows_subscriptions() {
        let tx_json = r#"[
        {
            "hash": "0x1dddc43e70bb5727fa75ae1213007511a2e4472792f08ca7c31c92eaa603bd75",
            "nonce": "0xd137",
            "blockHash": "0xae541fc4dc35d1d8bc2a018160e5ac8876d51ad76539d0b134ac5b82d64e7bda",
            "blockNumber": "0x10fa231",
            "transactionIndex": "0x0",
            "from": "0xa009fa1ac416ec02f6f902a3a4a584b092ae6123",
            "to": "0xfbeedcfe378866dab6abbafd8b2986f5c1768737",
            "value": "0x10fa231",
            "gasPrice": "0x5cc1b8224",
            "gas": "0x55730",
            "input": "0x00000002fffffffffffffffffffffffffffffffffffffffffdf698a3256fb1602e22d800c02aaa39b223fe8d0a0e5c4f27ead9083c756cc295ad61b0a150d79219dcf64e1e6cc01f0b64c4ce00271000000000000000000000000000000000000000000000000027ef5a74cb7b2e81000000000000000000000000000000000000000000000000001dcc42a1d98a0e",
            "v": "0x1",
            "r": "0x9bcbb85e056904ee2524fc32f860d784916433013fc3802a28ebdc5770e958a",
            "s": "0x77739e70d671eb5adbbeff7f63cd121d0695d2ee56814dc80f9e5bdf7e8521f0",
            "type": "0x2",
            "accessList": [],
            "maxPriorityFeePerGas": "0x0",
            "maxFeePerGas": "0x70e2ae3f1",
            "chainId": "0x1"
        },
        {
            "hash": "0x077daf1a23be6c48bf5e101b85cc79d9e81969ef901a7099b4fedac3c0d59809",
            "nonce": "0x22e",
            "blockHash": "0xae541fc4dc35d1d8bc2a018160e5ac8876d51ad76539d0b134ac5b82d64e7bda",
            "blockNumber": "0x10fa231",
            "transactionIndex": "0x1",
            "from": "0xe398c02cf1e030b541bdc87efece27ad5ef1e783",
            "to": "0x7a250d5630b4cf539739df2c5dacb4c659f2488d",
            "value": "0x0",
            "gasPrice": "0xb2703a824",
            "gas": "0x7a120",
            "input": "0x791ac94700000000000000000000000000000000000000000000000000000a29e1e7c600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000e398c02cf1e030b541bdc87efece27ad5ef1e7830000000000000000000000000000000000000000000000000000000064c5999f00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000ea778a02ab20ce0a8132a0e5312b53a5f23cef5000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
            "v": "0x0",
            "r": "0xd768f4d808fc1cb0eedca99363b78d9fa42555b4f26cbf5fa48ba8af96bff159",
            "s": "0x7f4cd55d6d06422ce14f58e72b0f366b479f606d129e4fc959a5eb348c93e888",
            "type": "0x2",
            "accessList": [],
            "maxPriorityFeePerGas": "0x55ae82600",
            "maxFeePerGas": "0x174876e800",
            "chainId": "0x1"
        },
        {
            "hash": "0xd95178efd41bf911a49590193b754de5aec1a2a89105a770a3ec11f395b30c6b",
            "nonce": "0x10f7d",
            "blockHash": "0xae541fc4dc35d1d8bc2a018160e5ac8876d51ad76539d0b134ac5b82d64e7bda",
            "blockNumber": "0x10fa231",
            "transactionIndex": "0x2",
            "from": "0xe9f82f15910e161999777036e20cb4108f4df800",
            "to": "0x5050e08626c499411b5d0e0b5af0e83d3fd82edf",
            "value": "0xc100",
            "gasPrice": "0x5cc1b8224",
            "gas": "0x39414",
            "input": "0x78e111f60000000000000000000000007af98c047dbe5221c317cd404273714aa653917a00000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000144c22b6075000000000000000000000000cf6daab95c476106eca715d48de4b13287ffdeaa00000000000000000000000095ad61b0a150d79219dcf64e1e6cc01f0b64c4ce000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000000000000000000000000000003634481b27f114000000000000000000000000000000000000000000000008455a40a4c83980000000000000000000000000000000000000000a53a7b608b7eb800000000000000000000000000000000000000000000000000000000000000000000111c579d90000000000000000000000000000000000000000000000000000000111c579d90000000000000000000000000000000000000000000000000000000064c597887f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "v": "0x1",
            "r": "0x6d6aa7218ee0d3e0c707e327bf7b1d02a1d4d202c63815943930d93748635e73",
            "s": "0xc7a27b9813287947c4b62acd505fc05553197f58c417be20d25a09099ebc9fc",
            "type": "0x2",
            "accessList": [],
            "maxPriorityFeePerGas": "0x0",
            "maxFeePerGas": "0x8b2294336",
            "chainId": "0x1"
        }]"#;

        let (pr, mut mock) = Provider::mocked();
        let vec_tx: Vec<Transaction> = serde_json::from_str(tx_json).unwrap();

        for tx in &vec_tx {
            mock.push(tx.clone().hash).unwrap();
        }

        assert!(mock.setup_subscription().await.is_ok());

        let mut subscription = pr.subscribe_pending_txs().await.unwrap();

        for i in (vec_tx.len() - 1)..0 {
            let received_tx_hash = subscription.next().await.unwrap();
            assert_eq!(vec_tx[i].hash, received_tx_hash);
        }
    }
}
