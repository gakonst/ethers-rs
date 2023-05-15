use std::fmt;

use ethers_core::types::U256;
use futures_channel::{mpsc, oneshot};
use serde::{de, Deserialize};
use serde_json::value::{to_raw_value, RawValue};

use crate::{common::Request, JsonRpcError};

// Normal JSON-RPC response
pub type Response = Result<Box<RawValue>, JsonRpcError>;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SubId(pub U256);

impl SubId {
    pub(super) fn serialize_raw(&self) -> Result<Box<RawValue>, serde_json::Error> {
        to_raw_value(&self)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Notification {
    pub subscription: U256,
    pub result: Box<RawValue>,
}

#[derive(Debug, Clone)]
pub enum PubSubItem {
    Success { id: u64, result: Box<RawValue> },
    Error { id: u64, error: JsonRpcError },
    Notification { params: Notification },
}

// FIXME: ideally, this could be auto-derived as an untagged enum, but due to
// https://github.com/serde-rs/serde/issues/1183 this currently fails
impl<'de> Deserialize<'de> for PubSubItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ResponseVisitor;
        impl<'de> de::Visitor<'de> for ResponseVisitor {
            type Value = PubSubItem;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid jsonrpc 2.0 response object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
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
                                return Err(de::Error::invalid_value(
                                    de::Unexpected::Str(value),
                                    &"2.0",
                                ))
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

                            let value: Box<RawValue> = map.next_value()?;
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

                            let value: String = map.next_value()?;
                            method = Some(value);
                        }
                        "params" => {
                            if params.is_some() {
                                return Err(de::Error::duplicate_field("params"))
                            }

                            let value: Notification = map.next_value()?;
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
                        Ok(PubSubItem::Success { id, result })
                    }
                    (Some(id), None, Some(error), None, None) => {
                        Ok(PubSubItem::Error { id, error })
                    }
                    (None, None, None, Some(_), Some(params)) => {
                        Ok(PubSubItem::Notification { params })
                    }
                    _ => Err(de::Error::custom(
                        "response must be either a success/error or notification object",
                    )),
                }
            }
        }

        deserializer.deserialize_map(ResponseVisitor)
    }
}

impl std::fmt::Display for PubSubItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PubSubItem::Success { id, .. } => write!(f, "Req success. ID: {id}"),
            PubSubItem::Error { id, .. } => write!(f, "Req error. ID: {id}"),
            PubSubItem::Notification { params } => {
                write!(f, "Notification for sub: {:?}", params.subscription)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionDetails {
    pub url: String,
    #[cfg(not(target_arch = "wasm32"))]
    pub auth: Option<crate::Authorization>,
}

impl ConnectionDetails {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(url: impl AsRef<str>, auth: Option<crate::Authorization>) -> Self {
        Self { url: url.as_ref().to_string(), auth }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn new(url: impl AsRef<str>) -> Self {
        Self { url: url.as_ref().to_string() }
    }
}

impl<T> From<T> for ConnectionDetails
where
    T: AsRef<str>,
{
    #[cfg(not(target_arch = "wasm32"))]
    fn from(value: T) -> Self {
        ConnectionDetails { url: value.as_ref().to_string(), auth: None }
    }
    #[cfg(target_arch = "wasm32")]
    fn from(value: T) -> Self {
        ConnectionDetails { url: value.as_ref().to_string() }
    }
}

#[derive(Debug)]
pub(super) struct InFlight {
    pub method: String,
    pub params: Box<RawValue>,
    pub channel: oneshot::Sender<Response>,
}

impl InFlight {
    pub(super) fn to_request(&self, id: u64) -> Request<'_, Box<RawValue>> {
        Request::new(id, &self.method, self.params.clone())
    }

    pub(super) fn serialize_raw(&self, id: u64) -> Result<Box<RawValue>, serde_json::Error> {
        to_raw_value(&self.to_request(id))
    }
}

#[derive(Debug)]
pub(super) struct ActiveSub {
    pub params: Box<RawValue>,
    pub channel: mpsc::UnboundedSender<Box<RawValue>>,
    pub current_server_id: Option<U256>,
}

impl ActiveSub {
    pub(super) fn to_request(&self, id: u64) -> Request<'static, Box<RawValue>> {
        Request::new(id, "eth_subscribe", self.params.clone())
    }

    pub(super) fn serialize_raw(&self, id: u64) -> Result<Box<RawValue>, serde_json::Error> {
        to_raw_value(&self.to_request(id))
    }
}

/// Instructions for the `WsServer`.
pub enum Instruction {
    /// JSON-RPC request
    Request { method: String, params: Box<RawValue>, sender: oneshot::Sender<Response> },
    /// Cancel an existing subscription
    Unsubscribe { id: U256 },
}

#[cfg(target_arch = "wasm32")]
mod aliases {
    pub use wasm_bindgen::prelude::*;
    pub use wasm_bindgen_futures::spawn_local;
    pub use ws_stream_wasm::*;

    pub type Message = WsMessage;
    pub type WsError = ws_stream_wasm::WsErr;
    pub type WsStreamItem = Message;

    pub type InternalStream = futures_util::stream::Fuse<WsStream>;
}

#[cfg(not(target_arch = "wasm32"))]
mod aliases {
    pub use tokio_tungstenite::{
        connect_async, connect_async_with_config,
        tungstenite::{self, protocol::CloseFrame},
    };
    use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
    pub type WebSocketConfig = tungstenite::protocol::WebSocketConfig;
    pub type Message = tungstenite::protocol::Message;
    pub type WsError = tungstenite::Error;
    pub type WsStreamItem = Result<Message, WsError>;

    pub use http::Request as HttpRequest;
    pub use tracing::{debug, error, trace, warn};
    pub use tungstenite::client::IntoClientRequest;

    pub use tokio::time::sleep;

    pub type InternalStream =
        futures_util::stream::Fuse<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>;

    impl IntoClientRequest for super::ConnectionDetails {
        fn into_client_request(
            self,
        ) -> tungstenite::Result<tungstenite::handshake::client::Request> {
            let mut request: HttpRequest<()> = self.url.into_client_request()?;
            if let Some(auth) = self.auth {
                let mut auth_value = http::HeaderValue::from_str(&auth.to_string())?;
                auth_value.set_sensitive(true);

                request.headers_mut().insert(http::header::AUTHORIZATION, auth_value);
            }

            request.into_client_request()
        }
    }
}

pub use aliases::*;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_desers_pubsub_items() {
        let a = "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":\"0xcd0c3e8af590364c09d0fa6a1210faf5\"}";
        serde_json::from_str::<PubSubItem>(a).unwrap();
    }
}
