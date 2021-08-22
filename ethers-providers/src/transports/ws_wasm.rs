use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use ws_stream_wasm::*;

use crate::{
    provider::ProviderError,
    transports::common::{JsonRpcError, Notification, Request, Response},
    JsonRpcClient, PubsubClient,
};
use ethers_core::types::U256;

use async_trait::async_trait;
use futures_channel::{mpsc, oneshot};
use futures_util::{
    sink::{Sink, SinkExt},
    stream::{Fuse, Stream, StreamExt},
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{self, Debug},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use thiserror::Error;

type Pending = oneshot::Sender<Result<serde_json::Value, JsonRpcError>>;
type Subscription = mpsc::UnboundedSender<serde_json::Value>;

type Message = WsMessage;
type WsError = ws_stream_wasm::WsErr;

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

/// Instructions for the `WsServer`.
enum Instruction {
    /// JSON-RPC request
    Request {
        id: u64,
        request: String,
        sender: Pending,
    },
    /// Create a new subscription
    Subscribe { id: U256, sink: Subscription },
    /// Cancel an existing subscription
    Unsubscribe { id: U256 },
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum Incoming {
    Notification(Notification<serde_json::Value>),
    Response(Response<serde_json::Value>),
}

/// A JSON-RPC Client over Websockets.
///
/// ```no_run
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// use ethers::providers::Ws;
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
        f.debug_struct("WebsocketProvider")
            .field("id", &self.id)
            .finish()
    }
}

impl Ws {
    /// Initializes a new WebSocket Client, given a Stream/Sink Websocket implementer.
    /// The websocket connection must be initiated separately.
    pub fn new<S: 'static>(ws: S) -> Self
        where
            S: Send
            + Sync
            + Stream<Item = Message>
            + Sink<Message, Error = WsError>
            + Unpin,
    {
        let (sink, stream) = mpsc::unbounded();

        // Spawn the server
        WsServer::new(ws, stream).spawn();

        Self {
            id: Arc::new(AtomicU64::new(0)),
            instructions: sink,
        }
    }

    /// Returns true if the WS connection is active, false otherwise
    pub fn ready(&self) -> bool {
        !self.instructions.is_closed()
    }

    /// Initializes a new WebSocket Client
    pub async fn connect(url: &str) -> Result<Self, ClientError> {
        let (_,  wsio) = WsMeta::connect( url, None ).await.expect_throw( "Could not create websocket");

        Ok(Self::new(wsio))
    }

    fn send(&self, msg: Instruction) -> Result<(), ClientError> {
        self.instructions
            .unbounded_send(msg)
            .map_err(to_client_error)
    }
}

#[async_trait(?Send)]
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

        // wait for the response
        let res = receiver.await?;

        // in case the request itself has any errors
        let res = res?;

        // parse it
        Ok(serde_json::from_value(res)?)
    }
}

impl PubsubClient for Ws {
    type NotificationStream = mpsc::UnboundedReceiver<serde_json::Value>;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, ClientError> {
        let (sink, stream) = mpsc::unbounded();
        self.send(Instruction::Subscribe {
            id: id.into(),
            sink,
        })?;
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
        S: Send
        + Sync
        + Stream<Item = Message>
        + Sink<Message, Error = WsError>
        + Unpin,
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

    /// Spawns the event loop
    fn spawn(mut self)
        where
            S: 'static,
    {
        let f = async move {
            loop {
                match self.tick().await {
                    Err(ClientError::UnexpectedClose) => {
                        break;
                    }
                    Err(e) => {
                        panic!("WS Server panic: {}", e);
                    }
                    _ => {}
                }
            }
        };

       spawn_local(f);
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
            warn!(
                "Unsubscribing from non-existent subscription with id {:?}",
                id
            );
        }
        Ok(())
    }

    /// Dispatch an outgoing message
    async fn service(&mut self, instruction: Instruction) -> Result<(), ClientError> {
        match instruction {
            Instruction::Request {
                id,
                request,
                sender,
            } => self.service_request(id, request, sender).await,
            Instruction::Subscribe { id, sink } => self.service_subscribe(id, sink).await,
            Instruction::Unsubscribe { id } => self.service_unsubscribe(id).await,
        }
    }

    async fn handle_text(&mut self, inner: String) -> Result<(), ClientError> {
        match serde_json::from_str::<Incoming>(&inner) {
            Err(_) => {}
            Ok(Incoming::Response(resp)) => {
                if let Some(request) = self.pending.remove(&resp.id) {
                    request
                        .send(resp.data.into_result())
                        .map_err(to_client_error)?;
                }
            }
            Ok(Incoming::Notification(notification)) => {
                let id = notification.params.subscription;
                if let Some(stream) = self.subscriptions.get(&id) {
                    stream
                        .unbounded_send(notification.params.result)
                        .map_err(to_client_error)?;
                }
            }
        }
        Ok(())
    }

    async fn handle(&mut self, resp: Message) -> Result<(), ClientError> {
        match resp {
            Message::Text(inner) => self.handle_text(inner).await,
            Message::Binary(buf) => Err(ClientError::UnexpectedBinary(buf)),
        }
    }

    /// Processes 1 instruction or 1 incoming websocket message
    #[allow(clippy::single_match)]
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
    #[error("Websocket closed with info")]
    WsClosed,

    /// Something caused the websocket to close
    #[error("WebSocket connection closed unexpectedly")]
    UnexpectedClose,
}

impl From<ClientError> for ProviderError {
    fn from(src: ClientError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}