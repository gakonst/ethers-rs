// Code adapted from: https://github.com/althea-net/guac_rs/tree/master/web3/src/jsonrpc

use std::{boxed::Box, fmt};

use serde::{
    de::{self, DeserializeOwned, MapAccess, Unexpected, Visitor},
    Deserialize, Serialize,
};
use serde_json::{value::RawValue, Value};
use thiserror::Error;

use ethers_core::types::U256;

#[derive(Deserialize, Debug, Clone, Error)]
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
        write!(f, "(code: {}, message: {}, data: {:?})", self.code, self.message, self.data)
    }
}

fn is_zst<T>(_t: &T) -> bool {
    std::mem::size_of::<T>() == 0
}

#[derive(Serialize, Deserialize, Debug)]
/// A JSON-RPC request
pub struct Request<'a, T> {
    id: u64,
    jsonrpc: &'a str,
    method: &'a str,
    #[serde(skip_serializing_if = "is_zst")]
    params: T,
}

impl<'a, T> Request<'a, T> {
    /// Creates a new JSON RPC request
    pub fn new(id: u64, method: &'a str, params: T) -> Self {
        Self { id, jsonrpc: "2.0", method, params }
    }
}

/// A JSON-RPC response
#[derive(Debug)]
pub enum Response<'a> {
    Success { id: u64, result: &'a RawValue },
    Error { id: u64, error: JsonRpcError },
    Notification { method: &'a str, params: Params<'a> },
}

/// Error thrown when handling batches of JSON-RPC request and responses.
#[derive(Error, Debug)]
pub enum BatchError {
    /// Thrown if deserialization failed
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    /// Thrown if a response could not be parsed
    JsonRpcError(#[from] JsonRpcError),

    /// Thrown if the batch is empty.
    EmptyBatch,
}

impl std::fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBatch => write!(f, "The batch is empty."),
            other => other.fmt(f),
        }
    }
}

/// A batch of JSON-RPC requests.
#[derive(Clone, Debug, Default)]
pub struct BatchRequest {
    requests: Vec<Value>,
}

impl BatchRequest {
    /// Creates a new empty batch of JSON-RPC requests.
    ///
    /// # Examples
    ///
    /// ```
    /// use ethers_providers::BatchRequest;
    /// // Creates an empty batch.
    /// let batch = BatchRequest::new();
    /// ```
    pub fn new() -> Self {
        Self { requests: Vec::new() }
    }

    /// Creates a new empty batch with the specified capacity.
    ///
    /// # Arguments
    ///
    /// `capacity` -  capacity of the new empty batch of JSON-RPC requests.
    ///
    /// # Examples
    ///
    /// ```
    /// use ethers_providers::BatchRequest;
    /// // Creates an empty batch with a pre-allocated capacity of 10.
    /// let batch = BatchRequest::with_capacity(10);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self { requests: Vec::with_capacity(capacity) }
    }

    /// Returns the number of requests in the batch.
    pub fn len(&self) -> usize {
        self.requests.len()
    }

    /// Adds a new request to the batch.
    ///
    /// # Arguments
    ///
    /// * `method` - method for the JSON-RPC RPC request.
    ///
    /// * `params` - parameters for the JSON-RPC request.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), ethers_providers::BatchError> {
    /// use ethers_providers::BatchRequest;
    /// # use ethers_core::types::{Address, H256, BlockNumber};
    /// # let address: Address = "0xd5a37dC5C9A396A03dd1136Fc76A1a02B1c88Ffa".parse().unwrap();
    /// # let pos = H256::from_low_u64_be(8);
    /// // Creates an empty batch.
    /// let mut batch = BatchRequest::new();
    /// // Adds a request to the batch.
    /// batch.add_request("eth_getStorageAt", (address, pos, BlockNumber::Latest))?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_request<T>(&mut self, method: &str, params: T) -> Result<(), BatchError>
    where
        T: Serialize,
    {
        self.requests.push(serde_json::to_value(&Request::new(0, method, params))?);

        Ok(())
    }

    /// Sets the ids of the requests.
    ///
    /// # Arguments
    ///
    /// `first` - id for the first request in the batch.
    ///
    /// # Panics
    ///
    /// If one of the requests is malformed.
    pub(crate) fn set_ids(&mut self, mut first: u64) -> Result<(), BatchError> {
        for request in self.requests_mut()? {
            *request
                .get_mut("id")
                .expect("Malformed JSON-RPC request: {request}, id is missing.") = first.into();
            first += 1;
        }

        Ok(())
    }

    /// Returns a mutable reference to the underlying JSON-RPC requests.
    ///
    /// # Errors
    ///
    /// Returns `BatchError::EmptyBatch` if the batch is empty.
    pub(crate) fn requests_mut(&mut self) -> Result<&mut [Value], BatchError> {
        (self.requests.len() > 0).then(move || &mut self.requests[..]).ok_or(BatchError::EmptyBatch)
    }

    /// Returns an immutable reference to the underlying JSON-RPC requests.
    ///
    /// # Errors
    ///
    /// Returns `BatchError::EmptyBatch` if the batch is empty.
    pub(crate) fn requests(&self) -> Result<&[Value], BatchError> {
        (self.requests.len() > 0).then(|| &self.requests[..]).ok_or(BatchError::EmptyBatch)
    }
}

/// A batch of JSON-RPC responses.
#[derive(Clone, Debug)]
pub struct BatchResponse {
    responses: Vec<(u64, Result<Box<RawValue>, JsonRpcError>)>,
}

impl BatchResponse {
    /// Creates a new batch of JSON-RPC responses.
    ///
    /// # Arguments
    ///
    /// `responses` - vector of JSON-RPC responses.
    pub(crate) fn new(responses: Vec<Response>) -> Self {
        let mut responses = responses
            .into_iter()
            .map(|response| match response {
                Response::Success { id, result } => (id, Ok(result.to_owned())),
                Response::Error { id, error } => (id, Err(error)),
                _ => unreachable!(),
            })
            .collect::<Vec<(u64, Result<Box<RawValue>, JsonRpcError>)>>();
        // Sort the responses by descending id, as the order the requests were issued and the order
        // the responses were given may differ. Order is reversed because we pop elements when
        // retrieving the responses.
        responses.sort_by_key(|(id, _)| std::cmp::Reverse(*id));

        Self { responses }
    }

    /// Returns the id of the batch, that is the id of the first response.
    pub(crate) fn id(&self) -> Result<u64, BatchError> {
        // The id of the first request in the batch, be it successful or not, corresponds to the
        // id of the channel to send the response into.
        self.responses.last().map(|(id, _)| *id).ok_or(BatchError::EmptyBatch)
    }

    /// Returns the next response in the batch or `None` if the batch is empty.
    ///
    /// # Errors
    ///
    /// Returns the error corresponding to a given JSON-RPC request if it failed.
    pub fn next_response<T>(&mut self) -> Option<Result<T, BatchError>>
    where
        T: DeserializeOwned,
    {
        // The order is reversed.
        let item = self.responses.pop();
        // Deserializes and returns the response.
        item.map(|(_, body)| {
            body.map_err(Into::into)
                .and_then(|res| serde_json::from_str::<T>(res.get()).map_err(Into::into))
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct Params<'a> {
    pub subscription: U256,
    #[serde(borrow)]
    pub result: &'a RawValue,
}

// FIXME: ideally, this could be auto-derived as an untagged enum, but due to
// https://github.com/serde-rs/serde/issues/1183 this currently fails
impl<'de: 'a, 'a> Deserialize<'de> for Response<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ResponseVisitor<'a>(&'a ());
        impl<'de: 'a, 'a> Visitor<'de> for ResponseVisitor<'a> {
            type Value = Response<'a>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid jsonrpc 2.0 response object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut jsonrpc = false;

                // response & error
                let mut id = None;
                // only response
                let mut result = None;
                // only error
                let mut error = None;
                // only notification
                let mut method = None;
                let mut params = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "jsonrpc" => {
                            if jsonrpc {
                                return Err(de::Error::duplicate_field("jsonrpc"))
                            }

                            let value = map.next_value()?;
                            if value != "2.0" {
                                return Err(de::Error::invalid_value(Unexpected::Str(value), &"2.0"))
                            }

                            jsonrpc = true;
                        }
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"))
                            }

                            let value: u64 = map.next_value()?;
                            id = Some(value);
                        }
                        "result" => {
                            if result.is_some() {
                                return Err(de::Error::duplicate_field("result"))
                            }

                            let value: &RawValue = map.next_value()?;
                            result = Some(value);
                        }
                        "error" => {
                            if error.is_some() {
                                return Err(de::Error::duplicate_field("error"))
                            }

                            let value: JsonRpcError = map.next_value()?;
                            error = Some(value);
                        }
                        "method" => {
                            if method.is_some() {
                                return Err(de::Error::duplicate_field("method"))
                            }

                            let value: &str = map.next_value()?;
                            method = Some(value);
                        }
                        "params" => {
                            if params.is_some() {
                                return Err(de::Error::duplicate_field("params"))
                            }

                            let value: Params = map.next_value()?;
                            params = Some(value);
                        }
                        key => {
                            return Err(de::Error::unknown_field(
                                key,
                                &["id", "jsonrpc", "result", "error", "params", "method"],
                            ))
                        }
                    }
                }

                // jsonrpc version must be present in all responses
                if !jsonrpc {
                    return Err(de::Error::missing_field("jsonrpc"))
                }

                match (id, result, error, method, params) {
                    (Some(id), Some(result), None, None, None) => {
                        Ok(Response::Success { id, result })
                    }
                    (Some(id), None, Some(error), None, None) => Ok(Response::Error { id, error }),
                    (None, None, None, Some(method), Some(params)) => {
                        Ok(Response::Notification { method, params })
                    }
                    _ => Err(de::Error::custom(
                        "response must be either a success/error or notification object",
                    )),
                }
            }
        }

        deserializer.deserialize_map(ResponseVisitor(&()))
    }
}

/// Basic or bearer authentication in http or websocket transport
///
/// Use to inject username and password or an auth token into requests
#[derive(Clone, Debug)]
pub enum Authorization {
    Basic(String),
    Bearer(String),
}

impl Authorization {
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        let auth_secret = base64::encode(username.into() + ":" + &password.into());
        Self::Basic(auth_secret)
    }

    pub fn bearer(token: impl Into<String>) -> Self {
        Self::Bearer(token.into())
    }
}

impl fmt::Display for Authorization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Authorization::Basic(auth_secret) => write!(f, "Basic {}", auth_secret),
            Authorization::Bearer(token) => write!(f, "Bearer {}", token),
        }
    }
}

#[cfg(test)]
mod tests {
    use ethers_core::types::U64;

    use super::*;

    #[test]
    fn deser_response() {
        let _ =
            serde_json::from_str::<Response<'_>>(r#"{"jsonrpc":"2.0","result":19}"#).unwrap_err();
        let _ = serde_json::from_str::<Response<'_>>(r#"{"jsonrpc":"3.0","result":19,"id":1}"#)
            .unwrap_err();

        let response: Response<'_> =
            serde_json::from_str(r#"{"jsonrpc":"2.0","result":19,"id":1}"#).unwrap();

        match response {
            Response::Success { id, result } => {
                assert_eq!(id, 1);
                let result: u64 = serde_json::from_str(result.get()).unwrap();
                assert_eq!(result, 19);
            }
            _ => panic!("expected `Success` response"),
        }

        let response: Response<'_> = serde_json::from_str(
            r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"error occurred"},"id":2}"#,
        )
        .unwrap();

        match response {
            Response::Error { id, error } => {
                assert_eq!(id, 2);
                assert_eq!(error.code, -32000);
                assert_eq!(error.message, "error occurred");
                assert!(error.data.is_none());
            }
            _ => panic!("expected `Error` response"),
        }

        let response: Response<'_> =
            serde_json::from_str(r#"{"jsonrpc":"2.0","result":"0xfa","id":0}"#).unwrap();

        match response {
            Response::Success { id, result } => {
                assert_eq!(id, 0);
                let result: U64 = serde_json::from_str(result.get()).unwrap();
                assert_eq!(result.as_u64(), 250);
            }
            _ => panic!("expected `Success` response"),
        }
    }

    #[test]
    fn ser_request() {
        let request: Request<()> = Request::new(0, "eth_chainId", ());
        assert_eq!(
            &serde_json::to_string(&request).unwrap(),
            r#"{"id":0,"jsonrpc":"2.0","method":"eth_chainId"}"#
        );

        let request: Request<()> = Request::new(300, "method_name", ());
        assert_eq!(
            &serde_json::to_string(&request).unwrap(),
            r#"{"id":300,"jsonrpc":"2.0","method":"method_name"}"#
        );

        let request: Request<u32> = Request::new(300, "method_name", 1);
        assert_eq!(
            &serde_json::to_string(&request).unwrap(),
            r#"{"id":300,"jsonrpc":"2.0","method":"method_name","params":1}"#
        );
    }
}
