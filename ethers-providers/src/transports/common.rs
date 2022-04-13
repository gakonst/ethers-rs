// Code adapted from: https://github.com/althea-net/guac_rs/tree/master/web3/src/jsonrpc
use std::fmt;

use serde::{
    de::{self, MapAccess, Unexpected, Visitor},
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

/// A JSON-RPC Notifcation
#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct Notification<'a> {
    #[serde(alias = "JSONRPC")]
    jsonrpc: &'a str,
    method: &'a str,
    #[serde(borrow)]
    pub params: Subscription<'a>,
}

#[derive(Deserialize, Debug)]
pub struct Subscription<'a> {
    pub subscription: U256,
    #[serde(borrow)]
    pub result: &'a RawValue,
}

#[derive(Debug)]
pub enum Response<'a> {
    Success { id: u64, jsonrpc: &'a str, result: &'a RawValue },
    Error { id: u64, jsonrpc: &'a str, error: JsonRpcError },
}

impl Response<'_> {
    pub fn id(&self) -> u64 {
        match self {
            Self::Success { id, .. } => *id,
            Self::Error { id, .. } => *id,
        }
    }

    pub fn as_result(&self) -> Result<&RawValue, &JsonRpcError> {
        match self {
            Self::Success { result, .. } => Ok(*result),
            Self::Error { error, .. } => Err(error),
        }
    }

    pub fn into_result(self) -> Result<Box<RawValue>, JsonRpcError> {
        match self {
            Self::Success { result, .. } => Ok(result.to_owned()),
            Self::Error { error, .. } => Err(error),
        }
    }
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
                let mut id = None;
                let mut jsonrpc = None;
                let mut result = None;
                let mut error = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "id" => {
                            let value: u64 = map.next_value()?;
                            let prev = id.replace(value);
                            if prev.is_some() {
                                return Err(de::Error::duplicate_field("id"))
                            }
                        }
                        "jsonrpc" => {
                            let value: &'de str = map.next_value()?;
                            if value != "2.0" {
                                return Err(de::Error::invalid_value(Unexpected::Str(value), &"2.0"))
                            }

                            let prev = jsonrpc.replace(value);
                            if prev.is_some() {
                                return Err(de::Error::duplicate_field("jsonrpc"))
                            }
                        }
                        "result" => {
                            let value: &RawValue = map.next_value()?;
                            let prev = result.replace(value);
                            if prev.is_some() {
                                return Err(de::Error::duplicate_field("result"))
                            }
                        }
                        "error" => {
                            let value: JsonRpcError = map.next_value()?;
                            let prev = error.replace(value);
                            if prev.is_some() {
                                return Err(de::Error::duplicate_field("error"))
                            }
                        }
                        key => {
                            return Err(de::Error::unknown_field(
                                key,
                                &["id", "jsonrpc", "result", "error"],
                            ))
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let jsonrpc = jsonrpc.ok_or_else(|| de::Error::missing_field("jsonrpc"))?;

                match (result, error) {
                    (Some(result), None) => Ok(Response::Success { id, jsonrpc, result }),
                    (None, Some(error)) => Ok(Response::Error { id, jsonrpc, error }),
                    _ => Err(de::Error::custom(
                        "response must have either a `result` or `error` field",
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
    use super::*;

    #[test]
    fn deser_response() {
        let _ =
            serde_json::from_str::<Response<'_>>(r#"{"jsonrpc":"2.0","result":19}"#).unwrap_err();
        let _ = serde_json::from_str::<Response<'_>>(r#"{"jsonrpc":"3.0","result":19,"id":1}"#)
            .unwrap_err();

        let response: Response<'_> =
            serde_json::from_str(r#"{"jsonrpc":"2.0","result":19,"id":1}"#).unwrap();

        assert_eq!(response.id(), 1);
        let result: u64 = serde_json::from_str(response.into_result().unwrap().get()).unwrap();
        assert_eq!(result, 19);

        let response: Response<'_> = serde_json::from_str(
            r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"error occurred"},"id":2}"#,
        )
        .unwrap();

        assert_eq!(response.id(), 2);
        let err = response.into_result().unwrap_err();
        assert_eq!(err.code, -32000);
        assert_eq!(err.message, "error occurred");
    }

    #[test]
    fn ser_request() {
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
