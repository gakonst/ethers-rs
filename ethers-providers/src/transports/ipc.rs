// use crate::{helpers, BatchTransport, DuplexTransport, Error, U256, Result, Transport};
use crate::{
    provider::ProviderError,
    transports::common::{JsonRpcError, Notification, Request, Response},
    JsonRpcClient, PubsubClient,
};
use ethers_core::types::U256;

use async_trait::async_trait;
use futures_util::stream::StreamExt;
use futures_channel::mpsc;
use oneshot::error::RecvError;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{atomic::AtomicU64, Arc},
};
use thiserror::Error;
use tokio::{io::AsyncWriteExt, net::UnixStream, sync::oneshot};
use tokio_util::io::ReaderStream;

type Result<T> = std::result::Result<T, IpcError>;

/// Unix Domain Sockets (IPC) transport.
#[derive(Debug, Clone)]
pub struct Ipc {
    id: Arc<AtomicU64>,
    messages_tx: mpsc::UnboundedSender<TransportMessage>,
}

#[cfg(unix)]
impl Ipc {
    /// Creates a new IPC transport from a given path.
    ///
    /// IPC is only available on Unix.
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let stream = UnixStream::connect(path).await?;

        Ok(Self::with_stream(stream))
    }

    fn with_stream(stream: UnixStream) -> Self {
        let id = Arc::new(AtomicU64::new(1));
        let (messages_tx, messages_rx) = mpsc::unbounded();

        tokio::spawn(run_server(stream, messages_rx));

        Ipc { id, messages_tx }
    }

    fn send(&self, msg: TransportMessage) -> Result<()> {
        self.messages_tx
            .unbounded_send(msg)
            .map_err(|_| IpcError::ChannelError("IPC server receiver dropped".to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl JsonRpcClient for Ipc {
    type Error = IpcError;

    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R> {
        let next_id = self.id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // send the message
        let (sender, receiver) = oneshot::channel();
        let payload = TransportMessage::Request {
            id: next_id,
            request: serde_json::to_string(&Request::new(next_id, method, params))?,
            sender,
        };

        // send the data
        self.send(payload)?;

        // wait for the response
        let res = receiver.await?;

        // parse it
        Ok(serde_json::from_value(res)?)
    }
}

impl PubsubClient for Ipc {
    type NotificationStream = mpsc::UnboundedReceiver<serde_json::Value>;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream> {
        let (sink, stream) = mpsc::unbounded();
        self.send(TransportMessage::Subscribe {
            id: id.into(),
            sink,
        })?;
        Ok(stream)
    }

    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<()> {
        self.send(TransportMessage::Unsubscribe { id: id.into() })
    }
}

#[derive(Debug)]
enum TransportMessage {
    Request {
        id: u64,
        request: String,
        sender: oneshot::Sender<serde_json::Value>,
    },
    Subscribe {
        id: U256,
        sink: mpsc::UnboundedSender<serde_json::Value>,
    },
    Unsubscribe {
        id: U256,
    },
}

#[cfg(unix)]
async fn run_server(
    unix_stream: UnixStream,
    messages_rx: mpsc::UnboundedReceiver<TransportMessage>,
) -> Result<()> {
    let (socket_reader, mut socket_writer) = unix_stream.into_split();
    let mut pending_response_txs = BTreeMap::default();
    let mut subscription_txs = BTreeMap::default();

    let mut socket_reader = ReaderStream::new(socket_reader);
    let mut messages_rx = messages_rx.fuse();
    let mut read_buffer = vec![];
    let mut closed = false;

    while !closed || !pending_response_txs.is_empty() {
        tokio::select! {
            message = messages_rx.next() => match message {
                None => closed = true,
                Some(TransportMessage::Subscribe{ id, sink }) => {
                    if subscription_txs.insert(id, sink).is_some() {
                        // log::warn!("Replacing a subscription with id {:?}", id);
                    }
                },
                Some(TransportMessage::Unsubscribe{ id }) => {
                    if  subscription_txs.remove(&id).is_none() {
                        // log::warn!("Unsubscribing not subscribed id {:?}", id);
                    }
                },
                Some(TransportMessage::Request{ id, request, sender }) => {
                    if pending_response_txs.insert(id, sender).is_some() {
                        // log::warn!("Replacing a pending request with id {:?}", id);
                    }

                    if socket_writer.write(&request.as_bytes()).await.is_err() {
                        pending_response_txs.remove(&id);
                        // log::error!("IPC write error: {:?}", err);
                    }
                }
            },
            bytes = socket_reader.next() => match bytes {
                Some(Ok(bytes)) => {
                    read_buffer.extend_from_slice(&bytes);

                    let read_len = {
                        let mut de: serde_json::StreamDeserializer<_, serde_json::Value> =
                            serde_json::Deserializer::from_slice(&read_buffer).into_iter();

                        while let Some(Ok(value)) = de.next() {
                            if let Ok(notification) = serde_json::from_value::<Notification<serde_json::Value>>(value.clone()) {
                                let _ = notify(&mut subscription_txs, notification);
                            } else if let Ok(response) = serde_json::from_value::<Response<serde_json::Value>>(value) {
                                let _ = respond(&mut pending_response_txs, response);
                            }

                            // log::warn!("JSON is not a response or notification");
                        }

                        de.byte_offset()
                    };

                    read_buffer.copy_within(read_len.., 0);
                    read_buffer.truncate(read_buffer.len() - read_len);
                },
                Some(Err(err)) => {
                    // log::error!("IPC read error: {:?}", err);
                    return Err(err.into());
                },
                None => break,
            }
        };
    }

    Ok(())
}

fn notify(
    subscription_txs: &mut BTreeMap<U256, mpsc::UnboundedSender<serde_json::Value>>,
    notification: Notification<serde_json::Value>,
) -> std::result::Result<(), IpcError> {
    let id = notification.params.subscription;
    if let Some(tx) = subscription_txs.get(&id) {
        tx.unbounded_send(notification.params.result)
            .map_err(|_| IpcError::ChannelError(format!("Subscription receiver {} dropped", id)))?;
    }

    Ok(())
}

fn respond(
    pending_response_txs: &mut BTreeMap<u64, oneshot::Sender<serde_json::Value>>,
    output: Response<serde_json::Value>,
) -> std::result::Result<(), IpcError> {
    let id = output.id;

    // Assuming results are always okay,
    let value = output.data.into_result()?;

    let response_tx = pending_response_txs.remove(&id).ok_or_else(|| {
        IpcError::ChannelError("No response channel exists for the response ID".to_string())
    })?;

    response_tx.send(value).map_err(|_| {
        IpcError::ChannelError("Receiver channel for response has been dropped".to_string())
    })?;

    Ok(())
}

#[derive(Error, Debug)]
/// Error thrown when sending or receiving an IPC message.
pub enum IpcError {
    /// Thrown if deserialization failed
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    /// std IO error forwarding.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    /// Thrown if the response could not be parsed
    JsonRpcError(#[from] JsonRpcError),

    /// Thrown if the websocket didn't respond to our message
    #[error("IPC connection did not respond with text data")]
    NoResponse,

    #[error("{0}")]
    ChannelError(String),

    #[error(transparent)]
    Canceled(#[from] RecvError),
}

impl From<IpcError> for ProviderError {
    fn from(src: IpcError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

#[cfg(all(test, unix))]
mod test {
    use super::*;

    // TODO write tests
}
