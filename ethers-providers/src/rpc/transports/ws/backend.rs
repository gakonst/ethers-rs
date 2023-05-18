use futures_channel::{mpsc, oneshot};
use futures_util::{select, sink::SinkExt, stream::StreamExt, FutureExt};

use serde_json::value::RawValue;

use super::{types::*, WsClientError};
use tracing::{error, trace};

/// `BackendDriver` drives a specific `WsBackend`. It can be used to issue
/// requests, receive responses, see errors, and shut down the backend.
pub struct BackendDriver {
    // Pubsub items from the backend, received via WS
    pub to_handle: mpsc::UnboundedReceiver<PubSubItem>,
    // Notification from the backend of a terminal error
    pub error: oneshot::Receiver<()>,

    // Requests that the backend should dispatch
    pub dispatcher: mpsc::UnboundedSender<Box<RawValue>>,
    // Notify the backend of intentional shutdown
    shutdown: oneshot::Sender<()>,
}

impl BackendDriver {
    pub fn shutdown(self) {
        // don't care if it fails, as that means the backend is gone anyway
        let _ = self.shutdown.send(());
    }
}

/// `WsBackend` dispatches requests and routes responses and notifications. It
/// also has a simple ping-based keepalive (when not compiled to wasm), to
/// prevent inactivity from triggering server-side closes
///
/// The `WsBackend` shuts down when instructed to by the `RequestManager` or
/// when the `RequestManager` drops (because the inbound channel will close)
pub struct WsBackend {
    server: InternalStream,

    // channel to the manager, through which to send items received via WS
    handler: mpsc::UnboundedSender<PubSubItem>,
    // notify manager of an error causing this task to halt
    error: oneshot::Sender<()>,

    // channel of inbound requests to dispatch
    to_dispatch: mpsc::UnboundedReceiver<Box<RawValue>>,
    // notification from manager of intentional shutdown
    shutdown: oneshot::Receiver<()>,
}

impl WsBackend {
    #[cfg(target_arch = "wasm32")]
    pub async fn connect(
        details: ConnectionDetails,
    ) -> Result<(Self, BackendDriver), WsClientError> {
        let wsio = WsMeta::connect(details.url, None)
            .await
            .expect_throw("Could not create websocket")
            .1
            .fuse();

        Ok(Self::new(wsio))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn connect(
        details: ConnectionDetails,
    ) -> Result<(Self, BackendDriver), WsClientError> {
        let ws = connect_async(details).await?.0.fuse();
        Ok(Self::new(ws))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn connect_with_config(
        details: ConnectionDetails,
        config: WebSocketConfig,
        disable_nagle: bool,
    ) -> Result<(Self, BackendDriver), WsClientError> {
        let ws = connect_async_with_config(details, Some(config), disable_nagle).await?.0.fuse();
        Ok(Self::new(ws))
    }

    pub fn new(server: InternalStream) -> (Self, BackendDriver) {
        let (handler, to_handle) = mpsc::unbounded();
        let (dispatcher, to_dispatch) = mpsc::unbounded();
        let (error_tx, error_rx) = oneshot::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        (
            WsBackend { server, handler, error: error_tx, to_dispatch, shutdown: shutdown_rx },
            BackendDriver { to_handle, error: error_rx, dispatcher, shutdown: shutdown_tx },
        )
    }

    pub async fn handle_text(&mut self, t: String) -> Result<(), WsClientError> {
        trace!(text = t, "Received message");
        match serde_json::from_str(&t) {
            Ok(item) => {
                trace!(%item, "Deserialized message");
                let res = self.handler.unbounded_send(item);
                if res.is_err() {
                    return Err(WsClientError::DeadChannel)
                }
            }
            Err(e) => {
                error!(e = %e, "Failed to deserialize message");
            }
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn handle(&mut self, item: WsStreamItem) -> Result<(), WsClientError> {
        match item {
            Ok(item) => match item {
                Message::Text(t) => self.handle_text(t).await,
                // https://github.com/snapview/tungstenite-rs/blob/42b8797e8b7f39efb7d9322dc8af3e9089db4f7d/src/protocol/mod.rs#L172-L175
                Message::Ping(_) => Ok(()),
                Message::Pong(_) => Ok(()),
                Message::Frame(_) => Ok(()),

                Message::Binary(buf) => Err(WsClientError::UnexpectedBinary(buf)),
                Message::Close(frame) => {
                    if frame.is_some() {
                        error!("Close frame: {}", frame.unwrap());
                    }
                    Err(WsClientError::UnexpectedClose)
                }
            },
            Err(e) => {
                error!(err = %e, "Error response from WS");
                Err(e.into())
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn handle(&mut self, item: WsStreamItem) -> Result<(), WsClientError> {
        match item {
            Message::Text(inner) => self.handle_text(inner).await,
            Message::Binary(buf) => Err(WsClientError::UnexpectedBinary(buf)),
        }
    }

    pub fn spawn(mut self) {
        let fut = async move {
            let mut err = false;
            loop {
                #[cfg(not(target_arch = "wasm32"))]
                let keepalive = tokio::time::sleep(std::time::Duration::from_secs(10)).fuse();
                #[cfg(not(target_arch = "wasm32"))]
                tokio::pin!(keepalive);

                // in wasm, we don't ping. as ping doesn't exist in our wasm lib
                #[cfg(target_arch = "wasm32")]
                let mut keepalive = futures_util::future::pending::<()>().fuse();

                select! {
                    _ = keepalive => {
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Err(e) = self.server.send(Message::Ping(vec![])).await {
                            error!(err = %e, "WS connection error");
                            err = true;
                            break
                        }
                        #[cfg(target_arch = "wasm32")]
                        unreachable!();
                    }
                    resp = self.server.next() => {
                        match resp {
                            Some(item) => {
                                err = self.handle(item).await.is_err();
                                if err { break }
                            },
                            None => {
                                error!("WS server has gone away");
                                err = true;
                                break
                            },
                        }
                    }
                    // we've received a new dispatch, so we send it via
                    // websocket
                    inst = self.to_dispatch.next() => {
                        match inst {
                            Some(msg) => {
                                if let Err(e) = self.server.send(Message::Text(msg.to_string())).await {
                                    error!(err = %e, "WS connection error");
                                    err = true;
                                    break
                                }
                            },
                            // dispatcher has gone away
                            None => {
                                break
                            },
                        }
                    },
                    // break on shutdown recv, or on shutdown recv error
                    _ = &mut self.shutdown => {
                        break
                    },
                }
            }
            if err {
                let _ = self.error.send(());
            }
        };

        #[cfg(target_arch = "wasm32")]
        super::spawn_local(fut);

        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(fut);
    }
}
