use futures_channel::{mpsc, oneshot};
use futures_util::{select, sink::SinkExt, stream::StreamExt, FutureExt};

use serde_json::value::RawValue;

use super::{types::*, WsClientError};
use crate::{ws_error, ws_trace};

pub struct WsBackend {
    server: InternalStream,

    handler: mpsc::UnboundedSender<WsItem>,
    error: oneshot::Sender<()>,

    to_dispatch: mpsc::UnboundedReceiver<Box<RawValue>>,
    shutdown: oneshot::Receiver<()>,
}

pub struct Backend {
    pub to_handle: mpsc::UnboundedReceiver<WsItem>,
    pub error: oneshot::Receiver<()>,

    pub dispatcher: mpsc::UnboundedSender<Box<RawValue>>,
    shutdown: oneshot::Sender<()>,
}

impl Backend {
    pub fn shutdown(self) {
        // don't care if it fails, as that means the backend is gone anyway
        let _ = self.shutdown.send(());
    }
}

impl WsBackend {
    #[cfg(target_arch = "wasm32")]
    pub async fn connect(details: ConnectionDetails) -> Result<(Self, Backend), WsClientError> {
        let wsio = WsMeta::connect(details.url, None)
            .await
            .expect_throw("Could not create websocket")
            .1
            .fuse();

        Ok(Self::new(wsio))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn connect(details: ConnectionDetails) -> Result<(Self, Backend), WsClientError> {
        let ws = connect_async(details).await?.0.fuse();
        Ok(Self::new(ws))
    }

    pub fn new(server: InternalStream) -> (Self, Backend) {
        let (handler, to_handle) = mpsc::unbounded();
        let (dispatcher, to_dispatch) = mpsc::unbounded();
        let (error_tx, error_rx) = oneshot::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        (
            WsBackend { server, handler, error: error_tx, to_dispatch, shutdown: shutdown_rx },
            Backend { to_handle, error: error_rx, dispatcher, shutdown: shutdown_tx },
        )
    }

    pub async fn handle_text(&mut self, t: String) -> Result<(), WsClientError> {
        ws_trace!("received message {t:?}");
        if let Ok(item) = serde_json::from_str(&t) {
            let res = self.handler.unbounded_send(item);
            if res.is_err() {
                return Err(WsClientError::DeadChannel)
            }
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn handle(&mut self, item: WsStreamItem) -> Result<(), WsClientError> {
        match item {
            Ok(item) => match item {
                Message::Text(t) => self.handle_text(t).await,
                Message::Ping(data) => {
                    if self.server.send(Message::Pong(data)).await.is_err() {
                        return Err(WsClientError::UnexpectedClose)
                    }
                    Ok(())
                }

                Message::Pong(_) => Ok(()),
                Message::Frame(_) => Ok(()),

                Message::Binary(buf) => Err(WsClientError::UnexpectedBinary(buf)),
                Message::Close(frame) => {
                    if frame.is_some() {
                        ws_error!("Close frame: {}", frame.unwrap());
                    }
                    Err(WsClientError::UnexpectedClose)
                }
            },
            Err(e) => {
                ws_error!(err = %e, "Error response from WS");
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
                            ws_error!("WS connection error {e}");
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
                                ws_error!("WS server has gone away");
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
                                    ws_error!("WS connection error {e}");
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
