use std::{
    error, fmt,
    sync::atomic::{AtomicU64, Ordering},
};

use futures_util::{SinkExt, StreamExt};
use serde_json::value::RawValue;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream};

use crate::{
    err::TransportError, jsonrpc::Response, Connection, NotificationReceiver, RequestFuture, U256,
};

use super::common::{FxHashMap, PendingRequest, Request};

type WebSocketStream = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;

/// The handle for an established WebSocket connection to an Ethereum JSON-RPC
/// provider.
pub struct Ws {
    next_id: AtomicU64,
    request_tx: mpsc::UnboundedSender<Request>,
}

impl Ws {
    pub async fn connect(url: &str) -> Result<Self, WsError> {
        let next_id = AtomicU64::new(1);
        let (request_tx, request_rx) = mpsc::unbounded_channel();

        // try to open a websocket connection to `url`
        let (stream, _) = tokio_tungstenite::connect_async(url).await?;

        // spawn a WS server task
        let _ = tokio::spawn(WsServer::new(stream).run(request_rx));

        Ok(Self { next_id, request_tx })
    }
}

impl Connection for Ws {
    fn request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn send_raw_request(&self, id: u64, request: String) -> RequestFuture<'_> {
        Box::pin(async move {
            // send the request to the WS server
            let (tx, rx) = oneshot::channel();
            self.request_tx.send(Request::Call { id, tx, request }).map_err(|_| todo!()).unwrap();

            // await the server's response
            rx.await.map_err(|_| server_exit())?
        })
    }
}

type Subscription = (mpsc::UnboundedSender<Box<RawValue>>, Option<NotificationReceiver>);

struct WsServer {
    pending: FxHashMap<u64, PendingRequest>,
    subs: FxHashMap<U256, Subscription>,
    stream: WebSocketStream,
}

type WsError = Box<dyn std::error::Error + Send + Sync>;

impl WsServer {
    fn new(stream: WebSocketStream) -> Self {
        Self {
            pending: FxHashMap::with_capacity_and_hasher(64, Default::default()),
            subs: FxHashMap::with_capacity_and_hasher(64, Default::default()),
            stream,
        }
    }

    async fn run(mut self, mut rx: mpsc::UnboundedReceiver<Request>) -> Result<(), WsError> {
        let x = self.stream.next().await;
        loop {
            tokio::select! {
                biased;
                request = rx.recv() => match request {
                    Some(request) => self.handle_request(request).await?,
                    None => break, // request channel closed, handle was dropped
                },
                msg = self.stream.next() => match msg {
                    Some(res) => {
                        let msg = res?;
                        self.handle_message(msg).await?;
                    },
                    None => break,
                },
            }
        }

        Ok(())
    }

    async fn handle_request(&mut self, request: Request) -> Result<(), WsError> {
        match request {
            Request::Call { id, request, tx } => {
                let prev = self.pending.insert(id, tx);
                assert!(prev.is_none(), "replaced pending IPC request (id={})", id);
                self.stream.send(request.into()).await?;
            }
            _ => todo!(),
        };

        Ok(())
    }

    async fn handle_message(&mut self, msg: Message) -> Result<bool, WsError> {
        match msg {
            Message::Text(text) => match serde_json::from_str(&text)? {
                Response::Success { id, result } => self.handle_response(id, Ok(result.to_owned())),
                Response::Error { id, error } => {
                    self.handle_response(id, Err(TransportError::jsonrpc(error)))
                }
                Response::Notification { params, .. } => todo!(),
            },
            Message::Ping(ping) => self.handle_ping(ping).await?,
            Message::Close(_) => return Ok(true),
            Message::Frame(_) | Message::Binary(_) | Message::Pong(_) => {}
        };

        Ok(false)
    }

    fn handle_response(&mut self, id: u64, res: Result<Box<RawValue>, TransportError>) {
        match self.pending.remove(&id) {
            Some(tx) => {
                // if send fails, request has been dropped at the callsite
                let _ = tx.send(res);
            }
            None => tracing::warn!(%id, "no pending request exists for response ID"),
        };
    }

    async fn handle_ping(&mut self, ping: Vec<u8>) -> Result<(), WsError> {
        self.stream.send(Message::Pong(ping)).await.map_err(Into::into)
    }
}

#[derive(Debug)]
enum WsErrorSoonish {
    Json(serde_json::Error),
    ServerExit,
    Websocket(tokio_tungstenite::tungstenite::Error),
}

impl error::Error for WsErrorSoonish {}

impl fmt::Display for WsErrorSoonish {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl From<serde_json::Error> for WsErrorSoonish {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

fn server_exit() -> TransportError {
    TransportError::transport(WsErrorSoonish::ServerExit)
}
