//! Types matching the [JSONRPC 2.0 specification](https://www.jsonrpc.org/specification).

use std::{error, fmt};

use ethers_core::types::U256;
use serde::{
    de::{self, Unexpected},
    ser::SerializeStruct as _,
    Deserialize, Serialize,
};
use serde_json::{value::RawValue, Value};

use crate::connection::ConnectionError;

/// A JSONRPC 2.0 request.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Request<'a, T> {
    /// The unique request ID.
    pub id: u64,
    /// The name of the remote method to be called.
    pub method: &'a str,
    /// The request parameters (which must be either a list or a map).
    pub params: T,
}

impl<T: Serialize> Request<'_, T> {
    /// Serializes the request to a JSON string.
    ///
    /// # Panics
    ///
    /// Panics if the request can not be serialized to a raw JSON value.
    pub fn to_json(&self) -> Box<RawValue> {
        self.try_to_json().expect("failed to serialize request as JSON")
    }

    /// Attempts to serialize the request to a raw JSON value.
    pub fn try_to_json(&self) -> Result<Box<RawValue>, serde_json::Error> {
        serde_json::value::to_raw_value(self)
    }
}

impl<T: Serialize> Serialize for Request<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let has_params = std::mem::size_of::<T>() != 0;
        let len = if has_params { 4 } else { 3 };

        let mut state = serializer.serialize_struct("Request", len)?;

        state.serialize_field("jsonrpc", "2.0")?;
        state.serialize_field("method", &self.method)?;

        if has_params {
            state.serialize_field("params", &self.params)?;
        }

        state.serialize_field("id", &self.id)?;

        state.end()
    }
}

/// Either a [`Response`] or an [`Error`]
#[derive(Clone, Debug)]
pub(crate) enum ResponseOrError<'a> {
    Response(Response<'a>),
    Error(Error),
}

impl ResponseOrError<'_> {
    pub(crate) fn id(&self) -> u64 {
        match self {
            Self::Response(response) => response.id,
            Self::Error(error) => error.id,
        }
    }

    pub(crate) fn to_result(self) -> Result<Box<RawValue>, ConnectionError> {
        match self {
            Self::Response(Response { result, .. }) => Ok(result.to_owned()),
            Self::Error(Error { error, .. }) => Err(ConnectionError::jsonrpc(error)),
        }
    }
}

// FIXME: ideally, `Deserialize` would be derived for `ResponseOrError` as an
// untagged enum, but since it contains `RawValue`s, derserialization will
// always fail
pub(crate) fn deserialize_batch_response(
    input: &str,
) -> Result<Vec<ResponseOrError<'_>>, serde_json::Error> {
    let raw_responses: Vec<&RawValue> = serde_json::from_str(input)?;
    let mut responses = Vec::with_capacity(raw_responses.len());

    for raw in raw_responses {
        if let Ok(response) = serde_json::from_str(raw.get()) {
            responses.push(ResponseOrError::Response(response));
            continue;
        }

        if let Ok(error) = serde_json::from_str(raw.get()) {
            responses.push(ResponseOrError::Error(error));
            continue;
        }

        todo!()
    }

    Ok(responses)
}

/// An JSON-RPC 2.0 success response.
#[derive(Copy, Clone, Debug, Deserialize)]
pub(crate) struct Response<'a> {
    pub id: u64,
    #[allow(unused)]
    pub jsonrpc: JsonRpc2,
    #[serde(borrow)]
    pub result: &'a RawValue,
}

/// An JSON-RPC 2.0 error response.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Error {
    pub id: u64,
    #[allow(unused)]
    pub jsonrpc: JsonRpc2,
    pub error: JsonRpcError,
}

/// An JSON-RPC 2.0 notification.
#[derive(Clone, Copy, Debug, Deserialize)]
pub(crate) struct Notification<'a> {
    #[allow(unused)]
    pub method: &'a str,
    #[allow(unused)]
    pub jsonrpc: JsonRpc2,
    #[serde(borrow)]
    pub params: Params<'a>,
}

/// An JSON-RPC 2.0 notification parameters object.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Params<'a> {
    pub subscription: U256,
    #[serde(borrow)]
    pub result: &'a RawValue,
}

/// The JSON-RPC 2.0 ID value.
#[derive(Clone, Copy)]
pub(crate) struct JsonRpc2;

impl fmt::Debug for JsonRpc2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("2.0")
    }
}

impl fmt::Display for JsonRpc2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("2.0")
    }
}

impl<'de> Deserialize<'de> for JsonRpc2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match Deserialize::deserialize(deserializer)? {
            "2.0" => Ok(JsonRpc2),
            inv => Err(de::Error::invalid_value(Unexpected::Str(inv), &"2.0")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
/// A JSON-RPC 2.0 error.
pub struct JsonRpcError {
    /// The error code.
    pub code: i64,
    /// The error message.
    pub message: String,
    /// The optional additional error context data.
    pub data: Option<Value>,
}

impl fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (msg, code) = (self.message.as_str(), self.code);
        match &self.data {
            Some(data) => write!(f, "{msg} (code={code},data={data})"),
            None => write!(f, "{msg} (code={code})"),
        }
    }
}

impl error::Error for JsonRpcError {}

#[cfg(test)]
mod tests {
    use ethers_core::types::Address;

    use super::Request;

    #[test]
    fn serialize_request() {
        let request = Request { id: 1, method: "eth_getBalance", params: [Address::zero()] };
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(
            json,
            r###"{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x0000000000000000000000000000000000000000"],"id":1}"###
        )
    }
}
