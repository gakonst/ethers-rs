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
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, protocol::Message},
};

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
    requests: mpsc::UnboundedSender<TransportMessage>,
}

type Pending = oneshot::Sender<serde_json::Value>;
type Subscription = mpsc::UnboundedSender<serde_json::Value>;

enum TransportMessage {
    Request {
        id: u64,
        request: String,
        sender: Pending,
    },
    Subscribe {
        id: U256,
        sink: Subscription,
    },
    Unsubscribe {
        id: U256,
    },
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
            + Stream<Item = Result<Message, tungstenite::Error>>
            + Sink<Message, Error = tungstenite::Error>
            + Unpin,
    {
        let (sink, stream) = mpsc::unbounded();

        // Spawn the server
        WsServer::new(ws, stream).spawn();

        Self {
            id: Arc::new(AtomicU64::new(0)),
            requests: sink,
        }
    }

    /// Initializes a new WebSocket Client
    pub async fn connect(
        url: impl tungstenite::client::IntoClientRequest + Unpin,
    ) -> Result<Self, ClientError> {
        let (ws, _) = connect_async(url).await?;
        Ok(Self::new(ws))
    }

    fn send(&self, msg: TransportMessage) -> Result<(), ClientError> {
        self.requests.unbounded_send(msg).map_err(to_client_error)
    }
}

#[async_trait]
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
        let payload = TransportMessage::Request {
            id: next_id,
            request: serde_json::to_string(&Request::new(next_id, method, params))?,
            sender,
        };

        // send the data
        self.send(payload).map_err(to_client_error)?;

        // wait for the response
        let res = receiver.await?;

        // parse it
        Ok(serde_json::from_value(res)?)
    }
}

impl PubsubClient for Ws {
    type NotificationStream = mpsc::UnboundedReceiver<serde_json::Value>;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, ClientError> {
        let (sink, stream) = mpsc::unbounded();
        self.send(TransportMessage::Subscribe {
            id: id.into(),
            sink,
        })?;
        Ok(stream)
    }

    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), ClientError> {
        self.send(TransportMessage::Unsubscribe { id: id.into() })
    }
}

struct WsServer<S> {
    ws: Fuse<S>,
    requests: Fuse<mpsc::UnboundedReceiver<TransportMessage>>,

    pending: BTreeMap<u64, Pending>,
    subscriptions: BTreeMap<U256, Subscription>,
}

impl<S> WsServer<S>
where
    S: Send
        + Sync
        + Stream<Item = Result<Message, tungstenite::Error>>
        + Sink<Message, Error = tungstenite::Error>
        + Unpin,
{
    /// Instantiates the Websocket Server
    fn new(ws: S, requests: mpsc::UnboundedReceiver<TransportMessage>) -> Self {
        Self {
            // Fuse the 2 steams together, so that we can `select` them in the
            // Stream implementation
            ws: ws.fuse(),
            requests: requests.fuse(),
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
                self.process().await.expect("WS Server panic");
            }
        };

        tokio::spawn(f);
    }

    /// Processes 1 item selected from the incoming `requests` or `ws`
    #[allow(clippy::single_match)]
    async fn process(&mut self) -> Result<(), ClientError> {
        futures_util::select! {
            // Handle requests
            msg = self.requests.next() => match msg {
                Some(msg) => self.handle_request(msg).await?,
                None => {},
            },
            // Handle ws messages
            msg = self.ws.next() => match msg {
                Some(Ok(msg)) => self.handle_ws(msg).await?,
                // TODO: Log the error?
                Some(Err(_)) => {},
                None => {},
            },
            // finished
            complete => {},
        };

        Ok(())
    }

    async fn handle_request(&mut self, msg: TransportMessage) -> Result<(), ClientError> {
        match msg {
            TransportMessage::Request {
                id,
                request,
                sender,
            } => {
                if self.pending.insert(id, sender).is_some() {
                    println!("Replacing a pending request with id {:?}", id);
                }

                if let Err(e) = self.ws.send(Message::Text(request)).await {
                    println!("WS connection error: {:?}", e);
                    self.pending.remove(&id);
                }
            }
            TransportMessage::Subscribe { id, sink } => {
                if self.subscriptions.insert(id, sink).is_some() {
                    println!("Replacing already-registered subscription with id {:?}", id);
                }
            }
            TransportMessage::Unsubscribe { id } => {
                if self.subscriptions.remove(&id).is_none() {
                    println!(
                        "Unsubscribing from non-existent subscription with id {:?}",
                        id
                    );
                }
            }
        };

        Ok(())
    }

    async fn handle_ws(&mut self, resp: Message) -> Result<(), ClientError> {
        match resp {
            Message::Text(inner) => self.handle_text(inner).await,
            Message::Ping(inner) => self.handle_ping(inner).await,
            Message::Pong(_) => Ok(()), // Server is allowed to send unsolicited pongs.
            _ => Err(ClientError::NoResponse),
        }
    }

    async fn handle_ping(&mut self, inner: Vec<u8>) -> Result<(), ClientError> {
        self.ws.send(Message::Pong(inner)).await?;
        Ok(())
    }

    async fn handle_text(&mut self, inner: String) -> Result<(), ClientError> {
        if let Ok(resp) = serde_json::from_str::<Response<serde_json::Value>>(&inner) {
            if let Some(request) = self.pending.remove(&resp.id) {
                request
                    .send(resp.data.into_result()?)
                    .map_err(to_client_error)?;
            }
        } else if let Ok(notification) =
            serde_json::from_str::<Notification<serde_json::Value>>(&inner)
        {
            let id = notification.params.subscription;
            if let Some(stream) = self.subscriptions.get(&id) {
                stream
                    .unbounded_send(notification.params.result)
                    .map_err(to_client_error)?;
            }
        }
        Ok(())
    }
}

// TrySendError is private :(
fn to_client_error<T: ToString>(err: T) -> ClientError {
    ClientError::ChannelError(err.to_string())
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
    #[error("Websocket connection did not respond with text data")]
    NoResponse,

    /// Thrown if there's an error over the WS connection
    #[error(transparent)]
    TungsteniteError(#[from] tungstenite::Error),

    #[error("{0}")]
    ChannelError(String),

    #[error(transparent)]
    Canceled(#[from] oneshot::Canceled),
}

impl From<ClientError> for ProviderError {
    fn from(src: ClientError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

#[cfg(test)]
#[cfg(not(feature = "celo"))]
mod tests {
    use super::*;
    use ethers_core::types::{Block, TxHash, U256};
    use ethers_core::utils::Ganache;

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
            let block = serde_json::from_value::<Block<TxHash>>(item).unwrap();
            blocks.push(block.number.unwrap_or_default().as_u64());
        }

        assert_eq!(sub_id, 1.into());
        assert_eq!(blocks, vec![1, 2, 3])
    }
}
