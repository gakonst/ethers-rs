use super::common::Params;
use async_trait::async_trait;
use bytes::{Buf, BytesMut};
use ethers_core::types::U256;
use futures_channel::mpsc;
use futures_util::stream::StreamExt;
use hashers::fx_hash::FxHasher64;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{value::RawValue, Deserializer};
use std::{
    cell::RefCell,
    convert::Infallible,
    hash::BuildHasherDefault,
    io,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    runtime,
    sync::oneshot::{self, error::RecvError},
};

use super::common::{JsonRpcError, Request, Response};
use crate::{errors::ProviderError, JsonRpcClient, PubsubClient};

type FxHashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<FxHasher64>>;

type Pending = oneshot::Sender<Result<Box<RawValue>, JsonRpcError>>;
type Subscription = mpsc::UnboundedSender<Box<RawValue>>;

#[cfg(unix)]
#[doc(hidden)]
mod imp {
    pub(super) use tokio::net::{
        unix::{ReadHalf, WriteHalf},
        UnixStream as Stream,
    };
}

#[cfg(windows)]
#[doc(hidden)]
mod imp {
    use super::*;
    use std::{
        ops::{Deref, DerefMut},
        pin::Pin,
        task::{Context, Poll},
        time::Duration,
    };
    use tokio::{
        io::{AsyncRead, AsyncWrite, ReadBuf},
        net::windows::named_pipe::{ClientOptions, NamedPipeClient},
        time::sleep,
    };
    use winapi::shared::winerror;

    /// Wrapper around [NamedPipeClient] to have the same methods as a UnixStream.
    ///
    /// Should not be exported.
    #[repr(transparent)]
    pub(super) struct Stream(pub NamedPipeClient);

    impl Deref for Stream {
        type Target = NamedPipeClient;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for Stream {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl Stream {
        pub async fn connect(addr: impl AsRef<Path>) -> Result<Self, io::Error> {
            let addr = addr.as_ref().as_os_str();
            loop {
                match ClientOptions::new().open(addr) {
                    Ok(client) => break Ok(Self(client)),
                    Err(e) if e.raw_os_error() == Some(winerror::ERROR_PIPE_BUSY as i32) => (),
                    Err(e) => break Err(e),
                }

                sleep(Duration::from_millis(50)).await;
            }
        }

        #[allow(unsafe_code)]
        pub fn split(&mut self) -> (ReadHalf, WriteHalf) {
            // SAFETY: ReadHalf cannot write but still needs a mutable reference for polling.
            // NamedPipeClient calls its `io` using immutable references, but it's private.
            let self1 = unsafe { &mut *(self as *mut Self) };
            let self2 = self;
            (ReadHalf(self1), WriteHalf(self2))
        }
    }

    impl AsyncRead for Stream {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            let this = Pin::new(&mut self.get_mut().0);
            this.poll_read(cx, buf)
        }
    }

    impl AsyncWrite for Stream {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            let this = Pin::new(&mut self.get_mut().0);
            this.poll_write(cx, buf)
        }

        fn poll_write_vectored(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &[io::IoSlice<'_>],
        ) -> Poll<io::Result<usize>> {
            let this = Pin::new(&mut self.get_mut().0);
            this.poll_write_vectored(cx, bufs)
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.poll_flush(cx)
        }
    }

    pub(super) struct ReadHalf<'a>(pub &'a mut Stream);

    pub(super) struct WriteHalf<'a>(pub &'a mut Stream);

    impl AsyncRead for ReadHalf<'_> {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            let this = Pin::new(&mut self.get_mut().0 .0);
            this.poll_read(cx, buf)
        }
    }

    impl AsyncWrite for WriteHalf<'_> {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            let this = Pin::new(&mut self.get_mut().0 .0);
            this.poll_write(cx, buf)
        }

        fn poll_write_vectored(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &[io::IoSlice<'_>],
        ) -> Poll<io::Result<usize>> {
            let this = Pin::new(&mut self.get_mut().0 .0);
            this.poll_write_vectored(cx, bufs)
        }

        fn is_write_vectored(&self) -> bool {
            self.0.is_write_vectored()
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            let this = Pin::new(&mut self.get_mut().0 .0);
            this.poll_flush(cx)
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.poll_flush(cx)
        }
    }
}

use self::imp::*;

#[cfg_attr(unix, doc = "A JSON-RPC Client over Unix IPC.")]
#[cfg_attr(windows, doc = "A JSON-RPC Client over named pipes.")]
///
/// # Example
///
/// ```no_run
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// use ethers_providers::Ipc;
///
/// // the ipc's path
#[cfg_attr(unix, doc = r#"let path = "/home/user/.local/share/reth/reth.ipc";"#)]
#[cfg_attr(windows, doc = r#"let path = r"\\.\pipe\reth.ipc";"#)]
/// let ipc = Ipc::connect(path).await?;
/// # Ok(())
/// # }
/// ```
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
    #[cfg_attr(unix, doc = "Connects to the Unix socket at the provided path.")]
    #[cfg_attr(windows, doc = "Connects to the named pipe at the provided path.\n")]
    #[cfg_attr(
        windows,
        doc = r"Note: the path must be the fully qualified, like: `\\.\pipe\<name>`."
    )]
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self, IpcError> {
        let id = Arc::new(AtomicU64::new(1));
        let (request_tx, request_rx) = mpsc::unbounded();

        let stream = Stream::connect(path).await?;
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

fn spawn_ipc_server(stream: Stream, request_rx: mpsc::UnboundedReceiver<TransportMessage>) {
    // 256 Kb should be more than enough for this thread, as all unbounded data
    // growth occurs on heap-allocated data structures and buffers and the call
    // stack is not going to do anything crazy either
    const STACK_SIZE: usize = 1 << 18;
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

async fn run_ipc_server(mut stream: Stream, request_rx: mpsc::UnboundedReceiver<TransportMessage>) {
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

/// Error thrown when sending or receiving an IPC message.
#[derive(Debug, Error)]
pub enum IpcError {
    /// Thrown if deserialization failed
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    /// std IO error forwarding.
    #[error(transparent)]
    IoError(#[from] io::Error),

    /// Server responded to the request with a valid JSON-RPC error response
    #[error(transparent)]
    JsonRpcError(#[from] JsonRpcError),

    /// Internal channel failed
    #[error("{0}")]
    ChannelError(String),

    /// Listener for request result is gone
    #[error(transparent)]
    RequestCancelled(#[from] RecvError),

    /// IPC server exited
    #[error("The IPC server has exited")]
    ServerExit,
}

impl From<IpcError> for ProviderError {
    fn from(src: IpcError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

impl crate::RpcError for IpcError {
    fn as_error_response(&self) -> Option<&super::JsonRpcError> {
        if let IpcError::JsonRpcError(err) = self {
            Some(err)
        } else {
            None
        }
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        match self {
            IpcError::JsonError(err) => Some(err),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers_core::utils::{Geth, GethInstance};
    use std::time::Duration;
    use tempfile::NamedTempFile;

    async fn connect() -> (Ipc, GethInstance) {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.into_temp_path().to_path_buf();
        let geth = Geth::new().block_time(1u64).ipc_path(&path).spawn();

        // [Windows named pipes](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipes)
        // are located at `\\<machine_address>\pipe\<pipe_name>`.
        #[cfg(windows)]
        let path = format!(r"\\.\pipe\{}", path.display());
        let ipc = Ipc::connect(path).await.unwrap();

        (ipc, geth)
    }

    #[tokio::test]
    async fn request() {
        let (ipc, _geth) = connect().await;

        let block_num: U256 = ipc.request("eth_blockNumber", ()).await.unwrap();
        tokio::time::sleep(Duration::from_secs(2)).await;
        let block_num2: U256 = ipc.request("eth_blockNumber", ()).await.unwrap();
        assert!(block_num2 > block_num);
    }

    #[tokio::test]
    #[cfg(not(feature = "celo"))]
    async fn subscription() {
        use ethers_core::types::{Block, TxHash};

        let (ipc, _geth) = connect().await;

        // Subscribing requires sending the sub request and then subscribing to
        // the returned sub_id
        let sub_id: U256 = ipc.request("eth_subscribe", ["newHeads"]).await.unwrap();
        let stream = ipc.subscribe(sub_id).unwrap();

        let blocks: Vec<u64> = stream
            .take(3)
            .map(|item| {
                let block: Block<TxHash> = serde_json::from_str(item.get()).unwrap();
                block.number.unwrap_or_default().as_u64()
            })
            .collect()
            .await;
        // `[1, 2, 3]` or `[2, 3, 4]` etc, depending on test latency
        assert_eq!(blocks[2], blocks[1] + 1);
        assert_eq!(blocks[1], blocks[0] + 1);
    }
}
