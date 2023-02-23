use ethers_core::types::U256;
use futures_channel::{mpsc, oneshot};
use serde::Deserialize;
use serde_json::value::RawValue;

use crate::{common::Request, JsonRpcError};

// Normal JSON-RPC response
pub type Response = Result<Box<RawValue>, JsonRpcError>;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SubId(pub U256);

impl SubId {
    pub(super) fn serialize_raw(&self) -> Result<Box<RawValue>, serde_json::Error> {
        let s = serde_json::to_string(&self)?;
        RawValue::from_string(s)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Notification {
    pub subscription: U256,
    pub result: Box<RawValue>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PubSubItem {
    Success { id: u64, result: Box<RawValue> },
    Error { id: u64, error: JsonRpcError },
    Notification { params: Notification },
}

#[derive(Debug, Clone)]
pub struct ConnectionDetails {
    pub url: String,
    #[cfg(not(target_arch = "wasm32"))]
    pub auth: Option<crate::Authorization>,
}

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
        let s = serde_json::to_string(&self.to_request(id))?;
        RawValue::from_string(s)
    }
}

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
        let s = serde_json::to_string(&self.to_request(id))?;
        RawValue::from_string(s)
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
        connect_async,
        tungstenite::{self, protocol::CloseFrame},
    };
    use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
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
