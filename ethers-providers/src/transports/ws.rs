use crate::{
    provider::ProviderError,
    transports::common::{JsonRpcError, Request},
    JsonRpcClient, PubsubClient,
};
use ethers_core::types::U256;

use async_trait::async_trait;
use futures_channel::{mpsc, oneshot};
use futures_util::{
    sink::{Sink, SinkExt},
    stream::{Fuse, Stream, StreamExt},
};
use serde::{
    de::{DeserializeOwned, Error},
    Serialize,
};
use serde_json::value::RawValue;
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::{self, Debug},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use thiserror::Error;

use super::common::{Notification, Response};

if_wasm! {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::spawn_local;
    use ws_stream_wasm::*;

    type Message = WsMessage;
    type WsError = ws_stream_wasm::WsErr;
    type WsStreamItem = Message;

    macro_rules! error {
        ( $( $t:tt )* ) => {
            web_sys::console::error_1(&format!( $( $t )* ).into());
        }
    }
    macro_rules! warn {
        ( $( $t:tt )* ) => {
            web_sys::console::warn_1(&format!( $( $t )* ).into());
        }
    }
    macro_rules! debug {
        ( $( $t:tt )* ) => {
            web_sys::console::log_1(&format!( $( $t )* ).into());
        }
    }
}

if_not_wasm! {
    use tokio_tungstenite::{
        connect_async,
        tungstenite::{
            self,
            protocol::CloseFrame,
        },
    };
    type Message = tungstenite::protocol::Message;
    type WsError = tungstenite::Error;
    type WsStreamItem = Result<Message, WsError>;
    use super::Authorization;
    use tracing::{debug, error, warn};
    use http::Request as HttpRequest;
    use tungstenite::client::IntoClientRequest;
}

type Pending = oneshot::Sender<Result<Box<RawValue>, JsonRpcError>>;
type Subscription = mpsc::UnboundedSender<Box<RawValue>>;

/// Instructions for the `WsServer`.
enum Instruction {
    /// JSON-RPC request
    Request { id: u64, request: String, sender: Pending },
    /// Create a new subscription
    Subscribe { id: U256, sink: Subscription },
    /// Cancel an existing subscription
    Unsubscribe { id: U256 },
}

/// A JSON-RPC Client over Websockets.
///
/// ```no_run
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// use ethers_providers::Ws;
///
/// let ws = Ws::connect("wss://localhost:8545").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Ws {
    id: Arc<AtomicU64>,
    instructions: mpsc::UnboundedSender<Instruction>,
}

impl Debug for Ws {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebsocketProvider").field("id", &self.id).finish()
    }
}

impl Ws {
    /// Initializes a new WebSocket Client, given a Stream/Sink Websocket implementer.
    /// The websocket connection must be initiated separately.
    pub fn new<S: 'static>(ws: S) -> Self
    where
        S: Send + Sync + Stream<Item = WsStreamItem> + Sink<Message, Error = WsError> + Unpin,
    {
        let (sink, stream) = mpsc::unbounded();
        // Spawn the server
        WsServer::new(ws, stream).spawn();

        Self { id: Arc::new(AtomicU64::new(0)), instructions: sink }
    }

    /// Returns true if the WS connection is active, false otherwise
    pub fn ready(&self) -> bool {
        !self.instructions.is_closed()
    }

    /// Initializes a new WebSocket Client
    #[cfg(target_arch = "wasm32")]
    pub async fn connect(url: &str) -> Result<Self, ClientError> {
        let (_, wsio) = WsMeta::connect(url, None).await.expect_throw("Could not create websocket");

        Ok(Self::new(wsio))
    }

    /// Initializes a new WebSocket Client
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn connect(url: impl IntoClientRequest + Unpin) -> Result<Self, ClientError> {
        let (ws, _) = connect_async(url).await?;
        Ok(Self::new(ws))
    }

    /// Initializes a new WebSocket Client with authentication
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn connect_with_auth(
        uri: impl IntoClientRequest + Unpin,
        auth: Authorization,
    ) -> Result<Self, ClientError> {
        let mut request: HttpRequest<()> = uri.into_client_request()?;

        let mut auth_value = http::HeaderValue::from_str(&auth.to_string())?;
        auth_value.set_sensitive(true);

        request.headers_mut().insert(http::header::AUTHORIZATION, auth_value);
        Self::connect(request).await
    }

    fn send(&self, msg: Instruction) -> Result<(), ClientError> {
        self.instructions.unbounded_send(msg).map_err(to_client_error)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl JsonRpcClient for Ws {
    type Error = ClientError;

    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, ClientError> {
        let next_id = self.id.fetch_add(1, Ordering::SeqCst);

        // send the message
        let (sender, receiver) = oneshot::channel();
        let payload = Instruction::Request {
            id: next_id,
            request: serde_json::to_string(&Request::new(next_id, method, params))?,
            sender,
        };

        // send the data
        self.send(payload)?;

        // wait for the response (the request itself may have errors as well)
        let res = receiver.await??;

        // parse it
        Ok(serde_json::from_str(res.get())?)
    }
}

impl PubsubClient for Ws {
    type NotificationStream = mpsc::UnboundedReceiver<Box<RawValue>>;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, ClientError> {
        let (sink, stream) = mpsc::unbounded();
        self.send(Instruction::Subscribe { id: id.into(), sink })?;
        Ok(stream)
    }

    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), ClientError> {
        self.send(Instruction::Unsubscribe { id: id.into() })
    }
}

struct WsServer<S> {
    ws: Fuse<S>,
    instructions: Fuse<mpsc::UnboundedReceiver<Instruction>>,

    pending: BTreeMap<u64, Pending>,
    subscriptions: BTreeMap<U256, Subscription>,
}

impl<S> WsServer<S>
where
    S: Send + Sync + Stream<Item = WsStreamItem> + Sink<Message, Error = WsError> + Unpin,
{
    /// Instantiates the Websocket Server
    fn new(ws: S, requests: mpsc::UnboundedReceiver<Instruction>) -> Self {
        Self {
            // Fuse the 2 steams together, so that we can `select` them in the
            // Stream implementation
            ws: ws.fuse(),
            instructions: requests.fuse(),
            pending: BTreeMap::default(),
            subscriptions: BTreeMap::default(),
        }
    }

    /// Returns whether the all work has been completed.
    ///
    /// If this method returns `true`, then the `instructions` channel has been closed and all
    /// pending requests and subscriptions have been completed.
    fn is_done(&self) -> bool {
        self.instructions.is_done() && self.pending.is_empty() && self.subscriptions.is_empty()
    }

    /// Spawns the event loop
    fn spawn(mut self)
    where
        S: 'static,
    {
        let f = async move {
            loop {
                if self.is_done() {
                    debug!("work complete");
                    break
                }
                match self.tick().await {
                    Err(ClientError::UnexpectedClose) => {
                        error!("{}", ClientError::UnexpectedClose);
                        break
                    }
                    Err(e) => {
                        panic!("WS Server panic: {}", e);
                    }
                    _ => {}
                }
            }
        };

        #[cfg(target_arch = "wasm32")]
        spawn_local(f);

        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(f);
    }

    // dispatch an RPC request
    async fn service_request(
        &mut self,
        id: u64,
        request: String,
        sender: Pending,
    ) -> Result<(), ClientError> {
        if self.pending.insert(id, sender).is_some() {
            warn!("Replacing a pending request with id {:?}", id);
        }

        if let Err(e) = self.ws.send(Message::Text(request)).await {
            error!("WS connection error: {:?}", e);
            self.pending.remove(&id);
        }
        Ok(())
    }

    /// Dispatch a subscription request
    async fn service_subscribe(&mut self, id: U256, sink: Subscription) -> Result<(), ClientError> {
        if self.subscriptions.insert(id, sink).is_some() {
            warn!("Replacing already-registered subscription with id {:?}", id);
        }
        Ok(())
    }

    /// Dispatch a unsubscribe request
    async fn service_unsubscribe(&mut self, id: U256) -> Result<(), ClientError> {
        if self.subscriptions.remove(&id).is_none() {
            warn!("Unsubscribing from non-existent subscription with id {:?}", id);
        }
        Ok(())
    }

    /// Dispatch an outgoing message
    async fn service(&mut self, instruction: Instruction) -> Result<(), ClientError> {
        match instruction {
            Instruction::Request { id, request, sender } => {
                self.service_request(id, request, sender).await
            }
            Instruction::Subscribe { id, sink } => self.service_subscribe(id, sink).await,
            Instruction::Unsubscribe { id } => self.service_unsubscribe(id).await,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn handle_ping(&mut self, inner: Vec<u8>) -> Result<(), ClientError> {
        self.ws.send(Message::Pong(inner)).await?;
        Ok(())
    }

    async fn handle_text(&mut self, inner: String) -> Result<(), ClientError> {
        if let Ok(response) = serde_json::from_str::<Response<'_>>(&inner) {
            if let Some(request) = self.pending.remove(&response.id()) {
                if !request.is_canceled() {
                    request.send(response.into_result()).map_err(to_client_error)?;
                }
            }

            return Ok(())
        }

        if let Ok(notification) = serde_json::from_str::<Notification<'_>>(&inner) {
            let id = notification.params.subscription;
            if let Entry::Occupied(stream) = self.subscriptions.entry(id) {
                if let Err(err) = stream.get().unbounded_send(notification.params.result.to_owned())
                {
                    if err.is_disconnected() {
                        // subscription channel was closed on the receiver end
                        stream.remove();
                    }
                    return Err(to_client_error(err))
                }
            }

            return Ok(())
        }

        Err(ClientError::JsonError(serde_json::Error::custom(
            "response is neither a valid jsonrpc response nor notification",
        )))
    }

    #[cfg(target_arch = "wasm32")]
    async fn handle(&mut self, resp: Message) -> Result<(), ClientError> {
        match resp {
            Message::Text(inner) => self.handle_text(inner).await,
            Message::Binary(buf) => Err(ClientError::UnexpectedBinary(buf)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn handle(&mut self, resp: Message) -> Result<(), ClientError> {
        match resp {
            Message::Text(inner) => self.handle_text(inner).await,
            Message::Frame(_) => Ok(()), // Server is allowed to send Raw frames
            Message::Ping(inner) => self.handle_ping(inner).await,
            Message::Pong(_) => Ok(()), // Server is allowed to send unsolicited pongs.
            Message::Close(Some(frame)) => Err(ClientError::WsClosed(frame)),
            Message::Close(None) => Err(ClientError::UnexpectedClose),
            Message::Binary(buf) => Err(ClientError::UnexpectedBinary(buf)),
        }
    }

    /// Processes 1 instruction or 1 incoming websocket message
    #[allow(clippy::single_match)]
    #[cfg(target_arch = "wasm32")]
    async fn tick(&mut self) -> Result<(), ClientError> {
        futures_util::select! {
            // Handle requests
            instruction = self.instructions.select_next_some() => {
                self.service(instruction).await?;
            },
            // Handle ws messages
            resp = self.ws.next() => match resp {
                Some(resp) => self.handle(resp).await?,
                None => {
                    return Err(ClientError::UnexpectedClose);
                },
            }
        };

        Ok(())
    }

    /// Processes 1 instruction or 1 incoming websocket message
    #[allow(clippy::single_match)]
    #[cfg(not(target_arch = "wasm32"))]
    async fn tick(&mut self) -> Result<(), ClientError> {
        futures_util::select! {
            // Handle requests
            instruction = self.instructions.select_next_some() => {
                self.service(instruction).await?;
            },
            // Handle ws messages
            resp = self.ws.next() => match resp {
                Some(Ok(resp)) => self.handle(resp).await?,
                // TODO: Log the error?
                Some(Err(_)) => {},
                None => {
                    return Err(ClientError::UnexpectedClose);
                },
            }
        };

        Ok(())
    }
}

// TrySendError is private :(
fn to_client_error<T: Debug>(err: T) -> ClientError {
    ClientError::ChannelError(format!("{:?}", err))
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

    /// Thrown if the websocket responds with binary data
    #[error("Websocket responded with unexpected binary data")]
    UnexpectedBinary(Vec<u8>),

    /// Thrown if there's an error over the WS connection
    #[error(transparent)]
    TungsteniteError(#[from] WsError),

    #[error("{0}")]
    ChannelError(String),

    #[error(transparent)]
    Canceled(#[from] oneshot::Canceled),

    /// Remote server sent a Close message
    #[error("Websocket closed with info: {0:?}")]
    #[cfg(not(target_arch = "wasm32"))]
    WsClosed(CloseFrame<'static>),

    /// Remote server sent a Close message
    #[error("Websocket closed with info")]
    #[cfg(target_arch = "wasm32")]
    WsClosed,

    /// Something caused the websocket to close
    #[error("WebSocket connection closed unexpectedly")]
    UnexpectedClose,

    /// Could not create an auth header for websocket handshake
    #[error(transparent)]
    #[cfg(not(target_arch = "wasm32"))]
    WsAuth(#[from] http::header::InvalidHeaderValue),

    /// Unable to create a valid Uri
    #[error(transparent)]
    #[cfg(not(target_arch = "wasm32"))]
    UriError(#[from] http::uri::InvalidUri),

    /// Unable to create a valid Request
    #[error(transparent)]
    #[cfg(not(target_arch = "wasm32"))]
    RequestError(#[from] http::Error),
}

impl From<ClientError> for ProviderError {
    fn from(src: ClientError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

#[cfg(test)]
#[cfg(not(feature = "celo"))]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use ethers_core::{
        types::{Block, TxHash, U256},
        utils::Ganache,
    };

    #[tokio::test]
    async fn request() {
        let ganache = Ganache::new().block_time(1u64).spawn();
        let ws = Ws::connect(ganache.ws_endpoint()).await.unwrap();

        let block_num: U256 = ws.request("eth_blockNumber", ()).await.unwrap();
        std::thread::sleep(std::time::Duration::new(3, 0));
        let block_num2: U256 = ws.request("eth_blockNumber", ()).await.unwrap();
        assert!(block_num2 > block_num);
    }

    #[tokio::test]
    async fn subscription() {
        let ganache = Ganache::new().block_time(1u64).spawn();
        let ws = Ws::connect(ganache.ws_endpoint()).await.unwrap();

        // Subscribing requires sending the sub request and then subscribing to
        // the returned sub_id
        let sub_id: U256 = ws.request("eth_subscribe", ["newHeads"]).await.unwrap();
        let mut stream = ws.subscribe(sub_id).unwrap();

        let mut blocks = Vec::new();
        for _ in 0..3 {
            let item = stream.next().await.unwrap();
            let block: Block<TxHash> = serde_json::from_str(item.get()).unwrap();
            blocks.push(block.number.unwrap_or_default().as_u64());
        }

        assert_eq!(sub_id, 1.into());
        assert_eq!(blocks, vec![1, 2, 3])
    }

    #[tokio::test]
    async fn deserialization_fails() {
        let ganache = Ganache::new().block_time(1u64).spawn();
        let (ws, _) = tokio_tungstenite::connect_async(ganache.ws_endpoint()).await.unwrap();
        let malformed_data = String::from("not a valid message");
        let (_, stream) = mpsc::unbounded();
        let resp = WsServer::new(ws, stream).handle_text(malformed_data).await;
        assert!(resp.is_err(), "Deserialization should not fail silently");
    }
}
