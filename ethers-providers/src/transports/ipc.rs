use crate::{
    provider::ProviderError,
    transports::common::{JsonRpcError, Notification, Request, Response},
    JsonRpcClient, PubsubClient,
};
use ethers_core::types::U256;

use async_trait::async_trait;
use futures_channel::mpsc;
use futures_util::stream::StreamExt;
use oneshot::error::RecvError;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::atomic::Ordering;
use std::{
    collections::HashMap,
    path::Path,
    sync::{atomic::AtomicU64, Arc},
};
use thiserror::Error;
use tokio::{io::AsyncWriteExt, net::UnixStream, sync::oneshot};
use tokio_util::io::ReaderStream;
use tracing::{error, warn};

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
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self, IpcError> {
        let stream = UnixStream::connect(path).await?;

        Ok(Self::with_stream(stream))
    }

    fn with_stream(stream: UnixStream) -> Self {
        let id = Arc::new(AtomicU64::new(1));
        let (messages_tx, messages_rx) = mpsc::unbounded();

        tokio::spawn(run_server(stream, messages_rx));

        Ipc { id, messages_tx }
    }

    fn send(&self, msg: TransportMessage) -> Result<(), IpcError> {
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
    ) -> Result<R, IpcError> {
        let next_id = self.id.fetch_add(1, Ordering::SeqCst);

        // Create the request and initialize the response channel
        let (sender, receiver) = oneshot::channel();
        let payload = TransportMessage::Request {
            id: next_id,
            request: serde_json::to_string(&Request::new(next_id, method, params))?,
            sender,
        };

        // Send the request to the IPC server to be handled.
        self.send(payload)?;

        // Wait for the response from the IPC server.
        let res = receiver.await?;

        // Parse JSON response.
        Ok(serde_json::from_value(res)?)
    }
}

impl PubsubClient for Ipc {
    type NotificationStream = mpsc::UnboundedReceiver<serde_json::Value>;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, IpcError> {
        let (sink, stream) = mpsc::unbounded();
        self.send(TransportMessage::Subscribe {
            id: id.into(),
            sink,
        })?;
        Ok(stream)
    }

    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), IpcError> {
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
) -> Result<(), IpcError> {
    let (socket_reader, mut socket_writer) = unix_stream.into_split();
    let mut pending_response_txs = HashMap::default();
    let mut subscription_txs = HashMap::default();

    let mut socket_reader = ReaderStream::new(socket_reader);
    let mut messages_rx = messages_rx.fuse();
    let mut read_buffer = vec![];
    let mut closed = false;

    while !closed || !pending_response_txs.is_empty() {
        tokio::select! {
            message = messages_rx.next() => match message {
                Some(TransportMessage::Subscribe{ id, sink }) => {
                    if subscription_txs.insert(id, sink).is_some() {
                        warn!("Replacing a subscription with id {:?}", id);
                    }
                },
                Some(TransportMessage::Unsubscribe{ id }) => {
                    if subscription_txs.remove(&id).is_none() {
                        warn!("Unsubscribing not subscribed id {:?}", id);
                    }
                },
                Some(TransportMessage::Request{ id, request, sender }) => {
                    if pending_response_txs.insert(id, sender).is_some() {
                        warn!("Replacing a pending request with id {:?}", id);
                    }

                    if let Err(err) = socket_writer.write(&request.as_bytes()).await {
                        pending_response_txs.remove(&id);
                        error!("IPC write error: {:?}", err);
                    }
                },
                None => closed = true,
            },
            bytes = socket_reader.next() => match bytes {
                Some(Ok(bytes)) => {
                    // Extend buffer of previously unread with the new read bytes
                    read_buffer.extend_from_slice(&bytes);

                    let read_len = {
                        // Deserialize as many full elements from the stream as exists
                        let mut de: serde_json::StreamDeserializer<_, serde_json::Value> =
                            serde_json::Deserializer::from_slice(&read_buffer).into_iter();

                        // Iterate through these elements, and handle responses/notifications
                        while let Some(Ok(value)) = de.next() {
                            if let Ok(notification) = serde_json::from_value::<Notification<serde_json::Value>>(value.clone()) {
                                // Send notify response if okay.
                                if let Err(e) = notify(&mut subscription_txs, notification) {
                                    error!("Failed to send IPC notification: {}", e)
                                }
                            } else if let Ok(response) = serde_json::from_value::<Response<serde_json::Value>>(value) {
                                if let Err(e) = respond(&mut pending_response_txs, response) {
                                    error!("Failed to send IPC response: {}", e)
                                }
                            } else {
                                warn!("JSON from IPC stream is not a response or notification");
                            }
                        }

                        // Get the offset of bytes to handle partial buffer reads
                        de.byte_offset()
                    };

                    // Reset buffer to just include the partial value bytes.
                    read_buffer.copy_within(read_len.., 0);
                    read_buffer.truncate(read_buffer.len() - read_len);
                },
                Some(Err(err)) => {
                    error!("IPC read error: {:?}", err);
                    return Err(err.into());
                },
                None => break,
            }
        };
    }

    Ok(())
}

/// Sends notification through the channel based on the ID of the subscription.
/// This handles streaming responses.
fn notify(
    subscription_txs: &mut HashMap<U256, mpsc::UnboundedSender<serde_json::Value>>,
    notification: Notification<serde_json::Value>,
) -> Result<(), IpcError> {
    let id = notification.params.subscription;
    if let Some(tx) = subscription_txs.get(&id) {
        tx.unbounded_send(notification.params.result)
            .map_err(|_| IpcError::ChannelError(format!("Subscription receiver {} dropped", id)))?;
    }

    Ok(())
}

/// Sends JSON response through the channel based on the ID in that response.
/// This handles RPC calls with only one response, and the channel entry is dropped after sending.
fn respond(
    pending_response_txs: &mut HashMap<u64, oneshot::Sender<serde_json::Value>>,
    output: Response<serde_json::Value>,
) -> Result<(), IpcError> {
    let id = output.id;

    // Converts output into result, to send data if valid response.
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
#[cfg(not(feature = "celo"))]
mod test {
    use super::*;
    use ethers::utils::Geth;
    use ethers_core::types::{Block, TxHash, U256};
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn request() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.into_temp_path().to_path_buf();
        let _geth = Geth::new().block_time(1u64).ipc_path(&path).spawn();
        let ipc = Ipc::new(path).await.unwrap();

        let block_num: U256 = ipc.request("eth_blockNumber", ()).await.unwrap();
        std::thread::sleep(std::time::Duration::new(3, 0));
        let block_num2: U256 = ipc.request("eth_blockNumber", ()).await.unwrap();
        assert!(block_num2 > block_num);
    }

    #[tokio::test]
    async fn subscription() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.into_temp_path().to_path_buf();
        let _geth = Geth::new().block_time(1u64).ipc_path(&path).spawn();
        let ipc = Ipc::new(path).await.unwrap();

        // Subscribing requires sending the sub request and then subscribing to
        // the returned sub_id
        let block_num: u64 = ipc
            .request::<_, U256>("eth_blockNumber", ())
            .await
            .unwrap()
            .as_u64();
        let sub_id: U256 = ipc.request("eth_subscribe", ["newHeads"]).await.unwrap();
        let mut stream = ipc.subscribe(sub_id).unwrap();

        let mut blocks = Vec::new();
        for _ in 0..3 {
            let item = stream.next().await.unwrap();
            let block = serde_json::from_value::<Block<TxHash>>(item).unwrap();
            blocks.push(block.number.unwrap_or_default().as_u64());
        }

        assert_eq!(blocks, &[block_num + 1, block_num + 2, block_num + 3])
    }
}
