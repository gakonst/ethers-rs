use std::{
    cell::RefCell,
    convert::Infallible,
    hash::BuildHasherDefault,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
};

use async_trait::async_trait;
use bytes::{Buf as _, BytesMut};
use ethers_core::types::U256;
use futures_channel::mpsc;
use futures_util::stream::StreamExt as _;
use hashers::fx_hash::FxHasher64;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{value::RawValue, Deserializer};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _, BufReader},
    net::{
        unix::{ReadHalf, WriteHalf},
        UnixStream,
    },
    runtime,
    sync::oneshot::{self, error::RecvError},
};

use crate::{
    provider::ProviderError,
    transports::common::{JsonRpcError, Request, Response},
    JsonRpcClient, PubsubClient,
};

use super::common::Params;

type FxHashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<FxHasher64>>;

type Pending = oneshot::Sender<Result<Box<RawValue>, JsonRpcError>>;
type Subscription = mpsc::UnboundedSender<Box<RawValue>>;

/// Unix Domain Sockets (IPC) transport.
#[derive(Debug, Clone)]
pub struct Ipc {
    id: Arc<AtomicU64>,
    request_tx: mpsc::UnboundedSender<TransportMessage>,
}

#[derive(Debug)]
enum TransportMessage {
    Request { id: u64, request: Box<[u8]>, sender: Pending },
    Subscribe { id: U256, sink: Subscription },
    Unsubscribe { id: U256 },
}

impl Ipc {
    /// Creates a new IPC transport from a given path using Unix sockets.
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self, IpcError> {
        let id = Arc::new(AtomicU64::new(1));
        let (request_tx, request_rx) = mpsc::unbounded();

        let stream = UnixStream::connect(path).await?;
        spawn_ipc_server(stream, request_rx);

        Ok(Self { id, request_tx })
    }

    fn send(&self, msg: TransportMessage) -> Result<(), IpcError> {
        self.request_tx
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
            request: serde_json::to_vec(&Request::new(next_id, method, params))?.into_boxed_slice(),
            sender,
        };

        // Send the request to the IPC server to be handled.
        self.send(payload)?;

        // Wait for the response from the IPC server.
        let res = receiver.await??;

        // Parse JSON response.
        Ok(serde_json::from_str(res.get())?)
    }
}

impl PubsubClient for Ipc {
    type NotificationStream = mpsc::UnboundedReceiver<Box<RawValue>>;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, IpcError> {
        let (sink, stream) = mpsc::unbounded();
        self.send(TransportMessage::Subscribe { id: id.into(), sink })?;
        Ok(stream)
    }

    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), IpcError> {
        self.send(TransportMessage::Unsubscribe { id: id.into() })
    }
}

fn spawn_ipc_server(stream: UnixStream, request_rx: mpsc::UnboundedReceiver<TransportMessage>) {
    // 65 KiB should be more than enough for this thread, as all unbounded data
    // growth occurs on heap-allocated data structures and buffers and the call
    // stack is not going to do anything crazy either
    const STACK_SIZE: usize = 1 << 16;
    // spawn a light-weight thread with a thread-local async runtime just for
    // sending and receiving data over the IPC socket
    let _ = thread::Builder::new()
        .name("ipc-server-thread".to_string())
        .stack_size(STACK_SIZE)
        .spawn(move || {
            let rt = runtime::Builder::new_current_thread()
                .enable_io()
                .build()
                .expect("failed to create ipc-server-thread async runtime");

            rt.block_on(run_ipc_server(stream, request_rx));
        })
        .expect("failed to spawn ipc server thread");
}

async fn run_ipc_server(
    mut stream: UnixStream,
    request_rx: mpsc::UnboundedReceiver<TransportMessage>,
) {
    // the shared state for both reads & writes
    let shared = Shared {
        pending: FxHashMap::with_capacity_and_hasher(64, BuildHasherDefault::default()).into(),
        subs: FxHashMap::with_capacity_and_hasher(64, BuildHasherDefault::default()).into(),
    };

    // split the stream and run two independent concurrently (local), thereby
    // allowing reads and writes to occurr concurrently
    let (reader, writer) = stream.split();
    let read = shared.handle_ipc_reads(reader);
    let write = shared.handle_ipc_writes(writer, request_rx);

    // run both loops concurrently, until either encounts an error
    if let Err(e) = futures_util::try_join!(read, write) {
        match e {
            IpcError::ServerExit => {}
            err => tracing::error!(?err, "exiting IPC server due to error"),
        }
    }
}

struct Shared {
    pending: RefCell<FxHashMap<u64, Pending>>,
    subs: RefCell<FxHashMap<U256, Subscription>>,
}

impl Shared {
    async fn handle_ipc_reads(&self, reader: ReadHalf<'_>) -> Result<Infallible, IpcError> {
        let mut reader = BufReader::new(reader);
        let mut buf = BytesMut::with_capacity(4096);

        loop {
            // try to read the next batch of bytes into the buffer
            let read = reader.read_buf(&mut buf).await?;
            if read == 0 {
                // eof, socket was closed
                return Err(IpcError::ServerExit)
            }

            // parse the received bytes into 0-n jsonrpc messages
            let read = self.handle_bytes(&buf)?;
            // split off all bytes that were parsed into complete messages
            // any remaining bytes that correspond to incomplete messages remain
            // in the buffer
            buf.advance(read);
        }
    }

    async fn handle_ipc_writes(
        &self,
        mut writer: WriteHalf<'_>,
        mut request_rx: mpsc::UnboundedReceiver<TransportMessage>,
    ) -> Result<Infallible, IpcError> {
        use TransportMessage::*;

        while let Some(msg) = request_rx.next().await {
            match msg {
                Request { id, request, sender } => {
                    let prev = self.pending.borrow_mut().insert(id, sender);
                    assert!(prev.is_none(), "{}", "replaced pending IPC request (id={id})");

                    if let Err(err) = writer.write_all(&request).await {
                        tracing::error!("IPC connection error: {:?}", err);
                        self.pending.borrow_mut().remove(&id);
                    }
                }
                Subscribe { id, sink } => {
                    if self.subs.borrow_mut().insert(id, sink).is_some() {
                        tracing::warn!(
                            %id,
                            "replaced already-registered subscription"
                        );
                    }
                }
                Unsubscribe { id } => {
                    if self.subs.borrow_mut().remove(&id).is_none() {
                        tracing::warn!(
                            %id,
                            "attempted to unsubscribe from non-existent subscription"
                        );
                    }
                }
            }
        }

        // the request receiver will only be closed if the sender instance
        // located within the transport handle is dropped, this is not truly an
        // error but leads to the `try_join` in `run_ipc_server` to cancel the
        // read half future
        Err(IpcError::ServerExit)
    }

    fn handle_bytes(&self, bytes: &BytesMut) -> Result<usize, IpcError> {
        // deserialize all complete jsonrpc responses in the buffer
        let mut de = Deserializer::from_slice(bytes.as_ref()).into_iter();
        while let Some(Ok(response)) = de.next() {
            match response {
                Response::Success { id, result } => self.send_response(id, Ok(result.to_owned())),
                Response::Error { id, error } => self.send_response(id, Err(error)),
                Response::Notification { params, .. } => self.send_notification(params),
            };
        }

        Ok(de.byte_offset())
    }

    fn send_response(&self, id: u64, result: Result<Box<RawValue>, JsonRpcError>) {
        // retrieve the channel sender for responding to the pending request
        let response_tx = match self.pending.borrow_mut().remove(&id) {
            Some(tx) => tx,
            None => {
                tracing::warn!(%id, "no pending request exists for the response ID");
                return
            }
        };

        // a failure to send the response indicates that the pending request has
        // been dropped in the mean time
        let _ = response_tx.send(result.map_err(Into::into));
    }

    /// Sends notification through the channel based on the ID of the subscription.
    /// This handles streaming responses.
    fn send_notification(&self, params: Params<'_>) {
        // retrieve the channel sender for notifying the subscription stream
        let subs = self.subs.borrow();
        let tx = match subs.get(&params.subscription) {
            Some(tx) => tx,
            None => {
                tracing::warn!(
                    id = ?params.subscription,
                    "no subscription exists for the notification ID"
                );
                return
            }
        };

        // a failure to send the response indicates that the pending request has
        // been dropped in the mean time (and should have been unsubscribed!)
        let _ = tx.unbounded_send(params.result.to_owned());
    }
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
    RequestCancelled(#[from] RecvError),

    #[error("The IPC server has exited")]
    ServerExit,
}

impl From<IpcError> for ProviderError {
    fn from(src: IpcError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}
#[cfg(all(test, target_family = "unix"))]
#[cfg(not(feature = "celo"))]
mod test {
    use super::*;
    use ethers_core::{
        types::{Block, TxHash, U256},
        utils::Geth,
    };
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn request() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.into_temp_path().to_path_buf();
        let _geth = Geth::new().block_time(1u64).ipc_path(&path).spawn();
        let ipc = Ipc::connect(path).await.unwrap();

        let block_num: U256 = ipc.request("eth_blockNumber", ()).await.unwrap();
        std::thread::sleep(std::time::Duration::new(3, 0));
        let block_num2: U256 = ipc.request("eth_blockNumber", ()).await.unwrap();
        assert!(block_num2 > block_num);
    }

    #[tokio::test]
    async fn subscription() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.into_temp_path().to_path_buf();
        let _geth = Geth::new().block_time(2u64).ipc_path(&path).spawn();
        let ipc = Ipc::connect(path).await.unwrap();

        let sub_id: U256 = ipc.request("eth_subscribe", ["newHeads"]).await.unwrap();
        let mut stream = ipc.subscribe(sub_id).unwrap();

        // Subscribing requires sending the sub request and then subscribing to
        // the returned sub_id
        let block_num: u64 = ipc.request::<_, U256>("eth_blockNumber", ()).await.unwrap().as_u64();
        let mut blocks = Vec::new();
        for _ in 0..3 {
            let item = stream.next().await.unwrap();
            let block: Block<TxHash> = serde_json::from_str(item.get()).unwrap();
            blocks.push(block.number.unwrap_or_default().as_u64());
        }
        let offset = blocks[0] - block_num;
        assert_eq!(blocks, &[block_num + offset, block_num + offset + 1, block_num + offset + 2])
    }
}
