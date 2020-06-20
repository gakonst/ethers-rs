use crate::{provider::ProviderError, JsonRpcClient};

use async_trait::async_trait;
use async_tungstenite::tungstenite::{self, protocol::Message};
use futures_util::{
    lock::Mutex,
    sink::{Sink, SinkExt},
    stream::{Stream, StreamExt},
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

use super::common::{JsonRpcError, Request, ResponseData};

// Convenience methods for connecting with async-std/tokio:

#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
use async_tungstenite::WebSocketStream;

// connect_async
#[cfg(all(feature = "async-std-runtime", not(feature = "tokio-runtime")))]
use async_tungstenite::async_std::connect_async;
#[cfg(feature = "tokio-runtime")]
use async_tungstenite::tokio::{connect_async, TokioAdapter};

#[cfg(feature = "tokio-runtime")]
type TcpStream = TokioAdapter<tokio::net::TcpStream>;
#[cfg(all(feature = "async-std-runtime", not(feature = "tokio-runtime")))]
type TcpStream = async_std::net::TcpStream;

// If there is no TLS, just use the TCP Stream
#[cfg(all(feature = "tokio-runtime", not(feature = "tokio-tls")))]
pub type MaybeTlsStream = TcpStream;
#[cfg(all(feature = "async-std-runtime", not(feature = "async-std-tls")))]
pub type MaybeTlsStream = TcpStream;

// Use either
#[cfg(feature = "tokio-tls")]
type TlsStream<S> = real_tokio_native_tls::TlsStream<S>;
#[cfg(all(feature = "async-std-tls", not(feature = "tokio-tls")))]
type TlsStream<S> = async_tls::client::TlsStream<S>;

#[cfg(any(feature = "tokio-tls", feature = "async-std-tls"))]
pub use async_tungstenite::stream::Stream as StreamSwitcher;
#[cfg(feature = "tokio-tls")]
pub type MaybeTlsStream =
    StreamSwitcher<TcpStream, TokioAdapter<TlsStream<TokioAdapter<TcpStream>>>>;
#[cfg(all(feature = "async-std-tls", not(feature = "tokio-tls")))]
pub type MaybeTlsStream = StreamSwitcher<TcpStream, TlsStream<TcpStream>>;

/// A JSON-RPC Client over Websockets.
pub struct Provider<S> {
    id: AtomicU64,
    ws: Mutex<S>,
}

#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
impl Provider<WebSocketStream<MaybeTlsStream>> {
    /// Initializes a new WebSocket Client. The websocket connection must be initiated
    /// separately.
    pub async fn connect(
        url: impl tungstenite::client::IntoClientRequest + Unpin,
    ) -> Result<Self, tungstenite::Error> {
        let (ws, _) = connect_async(url).await?;
        Ok(Self::new(ws))
    }
}

impl<S> Provider<S>
where
    S: Send
        + Sync
        + Stream<Item = Result<Message, tungstenite::Error>>
        + Sink<Message, Error = tungstenite::Error>
        + Unpin,
{
    /// Initializes a new WebSocket Client. The websocket connection must be initiated
    /// separately.
    pub fn new(ws: S) -> Self {
        Self {
            id: AtomicU64::new(0),
            ws: Mutex::new(ws),
        }
    }
}

#[derive(Error, Debug)]
/// Error thrown when sending a WS message
pub enum ClientError {
    /// Thrown if deserialization failed
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    /// Thrown if the response could not be parsed
    JsonRpcError(#[from] JsonRpcError),

    /// Thrown if the websocket didn't respond to our message
    #[error("Websocket connection did not respond with data")]
    NoResponse,

    /// Thrown if there's an error over the WS connection
    #[error(transparent)]
    TungsteniteError(#[from] tungstenite::Error),
}

impl From<ClientError> for ProviderError {
    fn from(src: ClientError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

#[async_trait]
impl<S> JsonRpcClient for Provider<S>
where
    S: Send
        + Sync
        + Stream<Item = Result<Message, tungstenite::Error>>
        + Sink<Message, Error = tungstenite::Error>
        + Unpin,
{
    type Error = ClientError;

    /// Sends a POST request with the provided method and the params serialized as JSON
    /// over HTTP
    async fn request<T: Serialize + Send + Sync, R: for<'a> Deserialize<'a>>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, ClientError> {
        // we get a lock on the websocket to avoid race conditions with multiple borrows
        let mut lock = self.ws.lock().await;

        let next_id = self.id.load(Ordering::SeqCst) + 1;
        self.id.store(next_id, Ordering::SeqCst);

        // send the message
        let payload = serde_json::to_string(&Request::new(next_id, method, params))?;
        lock.send(Message::text(payload)).await?;

        // get the response bytes
        let resp = lock.next().await.ok_or(ClientError::NoResponse)??;

        let data: ResponseData<R> = match resp {
            Message::Text(inner) => serde_json::from_str(&inner)?,
            Message::Binary(inner) => serde_json::from_slice(&inner)?,
            // TODO: Should we do something if we receive a Ping, Pong or Close?
            _ => return Err(ClientError::NoResponse),
        };

        Ok(data.into_result()?)
    }
}
