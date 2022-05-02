//! https://www.jsonrpc.org/specification

use std::{error, fmt};

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub fn to_json(&self) -> String {
        serde_json::to_string(self)
    }
}

impl<T: Serialize> Serialize for Request<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Request", 4)?;

        state.serialize_field("jsonrpc", "2.0")?;
        state.serialize_field("method", &self.method)?;
        state.serialize_field("params", &self.params)?;
        state.serialize_field("id", &self.id)?;

        state.end()
    }
}

/// A JSON-RPC 2.0 response.
#[derive(Debug)]
pub enum Response<'a> {
    Success { id: u64, result: &'a RawValue },
    Error { id: u64, error: JsonRpcError },
    Notification { method: &'a str, params: Params<'a> },
}

/// A JSON-RPC 2.0 notification parameters object.
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Eq)]
pub struct Params<'a> {
    pub subscription: U256,
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
                formatter.write_str("a valid JSON-RPC 2.0 response object")
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
                                return Err(de::Error::duplicate_field("jsonrpc"));
                            }

                            let value = map.next_value()?;
                            if value != "2.0" {
                                return Err(de::Error::invalid_value(
                                    Unexpected::Str(value),
                                    &"2.0",
                                ));
                            }

                            jsonrpc = true;
                        }
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }

                            let value: u64 = map.next_value()?;
                            id = Some(value);
                        }
                        "result" => {
                            if result.is_some() {
                                return Err(de::Error::duplicate_field("result"));
                            }

                            let value: &RawValue = map.next_value()?;
                            result = Some(value);
                        }
                        "error" => {
                            if error.is_some() {
                                return Err(de::Error::duplicate_field("error"));
                            }

                            let value: JsonRpcError = map.next_value()?;
                            error = Some(value);
                        }
                        "method" => {
                            if method.is_some() {
                                return Err(de::Error::duplicate_field("method"));
                            }

                            let value: &str = map.next_value()?;
                            method = Some(value);
                        }
                        "params" => {
                            if params.is_some() {
                                return Err(de::Error::duplicate_field("params"));
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
                    return Err(de::Error::missing_field("jsonrpc"));
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

#[derive(Debug, Deserialize, Clone, Hash, PartialEq, Eq)]
/// A JSON-RPC 2.0 error.
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
        let (msg, code) = (self.message.as_ref(), self.code);
        match self.data {
            Some(data) => write!(f, "{msg} (code={code},data={data})"),
            None => write!(f, "{msg} (code={code})", self.),
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
