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
    err::TransportError,
    jsonrpc::{Params, Response},
    Connection, NotificationReceiver, RequestFuture, U256,
};

use super::common::{FxHashMap, PendingRequest, Request};

type WebSocketStream = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;

/// The handle for an established WebSocket connection to an Ethereum JSON-RPC
/// provider.
pub struct WebSocket {
    next_id: AtomicU64,
    request_tx: mpsc::UnboundedSender<Request>,
}

impl WebSocket {
    /// Connects to the WS provider at `url`.
    pub async fn connect(url: &str) -> Result<Self, WsError> {
        let next_id = AtomicU64::new(1);
        let (request_tx, request_rx) = mpsc::unbounded_channel();

        // try to open a websocket connection to `url`
        let (stream, _) = tokio_tungstenite::connect_async(url).await?;

        // spawn a WS server task
        let _ = tokio::spawn(WsServer::new(stream).run(request_rx));

        Ok(Self { next_id, request_tx })
    }

    /// Returns `true` if the connetion is active.
    pub fn ready(&self) -> bool {
        !self.request_tx.is_closed()
    }
}

impl Connection for WebSocket {
    fn request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn send_raw_request(&self, id: u64, request: Box<RawValue>) -> RequestFuture<'_> {
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

impl WsServer {
    fn new(stream: WebSocketStream) -> Self {
        Self {
            pending: FxHashMap::with_capacity_and_hasher(64, Default::default()),
            subs: FxHashMap::with_capacity_and_hasher(64, Default::default()),
            stream,
        }
    }

    async fn run(mut self, mut rx: mpsc::UnboundedReceiver<Request>) {
        let res = loop {
            tokio::select! {
                biased;
                // 1) receive next request
                request = rx.recv() => match request {
                    Some(request) => {
                        if let Err(e) = self.handle_request(request).await {
                            break Err(e);
                        }
                    },
                    None => break Ok(()), // request channel closed, handle was dropped
                },
                // 2) receive & handle incoming responses
                msg = self.stream.next() => match msg {
                    Some(Ok(msg)) => {
                        if let Err(e) = self.handle_message(msg).await {
                            break Err(e);
                        }
                    }
                    Some(Err(e)) => break Err(e.into()),
                    None => break Ok(()),
                },
            }
        };

        if let Err(e) = res {
            tracing::error!(err = ?e, "exiting WS server due to error");
        }
    }

    async fn handle_request(&mut self, request: Request) -> Result<(), WsError> {
        match request {
            // RPC call requests are inserted into the `pending` map and their
            // payload is extracted to be written out
            Request::Call { id, request, tx } => {
                let prev = self.pending.insert(id, tx);
                assert!(prev.is_none(), "replaced pending IPC request (id={})", id);
                self.stream.send(request.into()).await?;
            }
            Request::Subscribe { id, tx } => {
                use std::collections::hash_map::Entry::*;
                let res = match self.subs.entry(id) {
                    // the entry already exists, e.g., because it was
                    // earlier instantiated by an incoming notification
                    Occupied(mut occ) => {
                        // take the receiver half, which is `None` if a
                        // subscription stream has already been created for
                        // this ID.
                        let (_, rx) = occ.get_mut();
                        rx.take()
                    }
                    Vacant(vac) => {
                        // insert a new channel tx/rx pair
                        let (sub_tx, sub_rx) = mpsc::unbounded_channel();
                        vac.insert((sub_tx, None));
                        Some(sub_rx)
                    }
                };

                let _ = tx.send(res);
            }
            Request::Unsubscribe { id } => {
                // removes the subscription entry and drops the sender half,
                // ending the registered subscription stream (if any)
                // NOTE: if the subscription has not been removed at the
                // provider side as well, it will keep sending further
                // notifications, which will re-create the entry
                let _ = self.subs.remove(&id);
            }
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
                Response::Notification { params, .. } => self.handle_notification(params),
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

    fn handle_notification(&mut self, params: Params<'_>) {
        todo!()
    }

    async fn handle_ping(&mut self, ping: Vec<u8>) -> Result<(), WsError> {
        self.stream.send(Message::Pong(ping)).await.map_err(Into::into)
    }
}

#[derive(Debug)]
pub enum WsError {
    Json(serde_json::Error),
    ServerExit,
    Websocket(tokio_tungstenite::tungstenite::Error),
}

impl error::Error for WsError {}

impl fmt::Display for WsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl From<serde_json::Error> for WsError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for WsError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::Websocket(err)
    }
}

fn server_exit() -> TransportError {
    TransportError::transport(WsError::ServerExit)
}
