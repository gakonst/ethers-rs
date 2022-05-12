use std::{
    cell::RefCell,
    error, fmt,
    hash::BuildHasherDefault,
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

use bytes::{Buf as _, BytesMut};
use ethers_core::types::U256;
use hashers::fx_hash::FxHasher64;
use serde_json::{value::RawValue, Deserializer};
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _, BufReader},
    net::{
        unix::{ReadHalf, WriteHalf},
        UnixStream,
    },
    runtime,
    sync::{mpsc, oneshot},
};

use crate::{
    err::TransportError,
    jsonrpc::{Params, Response},
    Connection, DuplexConnection, RequestFuture, ResponsePayload, SubscribeFuture,
    SubscribePayload, UnsubscribeFuture, UnsubscribePayload,
};

type FxHashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<FxHasher64>>;

type PendingRequest = oneshot::Sender<ResponsePayload>;
type PendingSubscribe = oneshot::Sender<SubscribePayload>;
type PendingUnsubscribe = oneshot::Sender<UnsubscribePayload>;

enum Request {
    Call { id: u64, tx: PendingRequest, request: String },
    Subscribe { id: u64, tx: PendingSubscribe, request: String },
    Unsubscribe { tx: PendingUnsubscribe, request: Box<UnsubscribeRequest> },
}

type UnsubscribeRequest = crate::Request<'static, [U256; 1]>;

enum Pending {
    Call { tx: PendingRequest },
    Subscribe { tx: PendingSubscribe },
    Unsubscribe { id: Box<U256>, tx: PendingUnsubscribe },
}

/// The handle for an IPC connection to an Ethereum JSON-RPC provider.
///
/// **Note** Dropping an [`Ipc`] handle will invalidate all pending requests
/// that were made through it.
pub struct Ipc {
    /// The counter for unique request ids.
    next_id: AtomicU64,
    /// The instance for sending requests to the IPC request server
    request_tx: mpsc::UnboundedSender<Request>,
}

impl Ipc {
    /// Connects to the IPC socket at `path`.
    ///
    /// # Error
    ///
    /// Fails, if establishing the connection to the socket fails.
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self, IpcError> {
        let next_id = AtomicU64::new(0);
        let (request_tx, request_rx) = mpsc::unbounded_channel();

        // connect to the IPC socket at `path`
        let path = path.as_ref();
        let stream = UnixStream::connect(path)
            .await
            .map_err(|source| IpcError::InvalidSocket { path: path.into(), source })?;

        // spawn an IPC server thread with its own runtime
        spawn_ipc_server(stream, request_rx);

        Ok(Self { next_id, request_tx })
    }
}

impl Connection for Ipc {
    fn request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn send_raw_request(&self, id: u64, request: String) -> RequestFuture<'_> {
        Box::pin(async move {
            // send the request to the IPC server
            let (tx, rx) = oneshot::channel();
            self.request_tx.send(Request::Call { id, tx, request }).map_err(|_| server_exit())?;

            // await the response
            rx.await.map_err(|_| server_exit())?
        })
    }
}

impl DuplexConnection for Ipc {
    fn subscribe(&self, id: u64, request: String) -> SubscribeFuture<'_> {
        Box::pin(async move {
            // send the request to the IPC server
            let (tx, rx) = oneshot::channel();
            self.request_tx
                .send(Request::Subscribe { id, tx, request })
                .map_err(|_| server_exit())?;

            // await the response
            rx.await.map_err(|_| server_exit())?
        })
    }

    fn unsubscribe(&self, id: &U256) -> UnsubscribeFuture<'_> {
        let (tx, rx) = oneshot::channel();
        let request =
            UnsubscribeRequest { id: self.request_id(), method: "eth_unsubscribe", params: [*id] };
        // send the request BEFORE entering any async code
        let res = self.request_tx.send(Request::Unsubscribe { tx, request: request.into() });

        Box::pin(async move {
            match res {
                Ok(_) => {
                    // await the response
                    rx.await.map_err(|_| server_exit())?
                }
                Err(_) => Err(server_exit()),
            }
        })
    }
}

fn spawn_ipc_server(stream: UnixStream, request_rx: mpsc::UnboundedReceiver<Request>) {
    // 65 KiB should be more than enough for this thread, as all unbounded data
    // growth occurs on heap-allocated data structures/buffers and the call
    // stack is not going to do anything odd either
    const STACK_SIZE: usize = 1 << 16;
    let _ = thread::Builder::new()
        .name("ipc-server-thread".to_string())
        .stack_size(STACK_SIZE)
        .spawn(move || {
            let rt = runtime::Builder::new_current_thread()
                .enable_io()
                .build()
                .expect("failed to create IPC server thread async runtime");

            rt.block_on(run_ipc_server(stream, request_rx));
        })
        .expect("failed to spawn IPC server thread");
}

async fn run_ipc_server(mut stream: UnixStream, request_rx: mpsc::UnboundedReceiver<Request>) {
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

    // run both loops concurrently & abort (drop) the other once either of them finishes.
    let res = tokio::select! {
        biased;
        res = read => res,
        res = write => res,
    };

    if let Err(e) = res {
        tracing::error!(err = ?e, "exiting IPC server due to error");
    }
}

struct Shared {
    pending: RefCell<FxHashMap<u64, Pending>>,
    subs: RefCell<FxHashMap<U256, mpsc::UnboundedSender<Box<RawValue>>>>,
}

impl Shared {
    async fn handle_ipc_reads(&self, reader: ReadHalf<'_>) -> Result<(), IpcError> {
        let mut reader = BufReader::new(reader);
        let mut buf = BytesMut::with_capacity(4096);

        loop {
            // try to read the next batch of bytes into the buffer
            let read = reader.read_buf(&mut buf).await?;
            if read == 0 {
                // eof, socket was closed
                return Ok(());
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
        mut request_rx: mpsc::UnboundedReceiver<Request>,
    ) -> Result<(), IpcError> {
        use Request::*;

        while let Some(msg) = request_rx.recv().await {
            match msg {
                Call { id, tx, request } => {
                    let prev = self.pending.borrow_mut().insert(id, Pending::Call { tx });
                    assert!(prev.is_none(), "replaced pending IPC request (id={})", id);
                    writer.write_all(request.as_bytes()).await?;
                }
                Subscribe { id, tx, request } => {
                    let prev = self.pending.borrow_mut().insert(id, Pending::Subscribe { tx });
                    assert!(prev.is_none(), "replaced pending IPC subscribe request (id={})", id);
                    writer.write_all(request.as_bytes()).await?;
                }
                Unsubscribe { tx, request } => {
                    let prev = self.pending.borrow_mut().insert(
                        request.id,
                        Pending::Unsubscribe { id: request.params[0].into(), tx },
                    );
                    assert!(
                        prev.is_none(),
                        "replaced pending IPC unsubscribe request (id={})",
                        request.id
                    );
                    writer.write_all(request.to_json().as_bytes()).await?;
                }
            }
        }

        // the IPC handle has been dropped
        Ok(())
    }

    fn handle_bytes(&self, bytes: &BytesMut) -> Result<usize, IpcError> {
        // deserialize all complete jsonrpc responses in the buffer
        let mut de = Deserializer::from_slice(bytes.as_ref()).into_iter();
        while let Some(Ok(response)) = de.next() {
            match response {
                Response::Success { id, result } => self.handle_response(id, Ok(result.to_owned())),
                Response::Error { id, error } => {
                    self.handle_response(id, Err(TransportError::jsonrpc(error)))
                }
                Response::Notification { params, .. } => self.handle_notification(params),
            };
        }

        Ok(de.byte_offset())
    }

    fn handle_response(&self, id: u64, res: Result<Box<RawValue>, Box<TransportError>>) {
        match self.pending.borrow_mut().remove(&id) {
            Some(Pending::Call { tx }) => {
                // if send fails, request has been dropped at the callsite
                let _ = tx.send(res);
            }
            Some(Pending::Subscribe { tx }) => {
                let res = self.handle_subscribe(res);
                // if send fails, request has been dropped at the callsite
                let _ = tx.send(res);
            }
            Some(Pending::Unsubscribe { id, tx }) => {
                let res = self.handle_unsubscribe(&id, res);
                // if send fails, request has been dropped at the callsite
                let _ = tx.send(res);
            }
            None => tracing::warn!(%id, "no pending request exists for the response ID"),
        };
    }

    fn handle_subscribe(
        &self,
        res: Result<Box<RawValue>, Box<TransportError>>,
    ) -> SubscribePayload {
        match res {
            Ok(raw) => {
                // parse the subscription id
                let id: U256 = serde_json::from_str(raw.get())
                    .map_err(|err| TransportError::json(raw.get(), err))?;
                let (sub_tx, sub_rx) = mpsc::unbounded_channel();

                let prev = self.subs.borrow_mut().insert(id, sub_tx);
                assert!(prev.is_none(), "replaced IPC subscription (id={})", id);

                Ok((id, sub_rx))
            }
            Err(e) => Err(e),
        }
    }

    fn handle_unsubscribe(
        &self,
        id: &U256,
        res: Result<Box<RawValue>, Box<TransportError>>,
    ) -> Result<bool, Box<TransportError>> {
        match res {
            Ok(raw) => {
                let ok: bool = serde_json::from_str(raw.get())
                    .map_err(|err| TransportError::json(raw.get(), err))?;

                let tx = self.subs.borrow_mut().remove(id);
                assert!(tx.is_some(), "tried to unsubscribe non-existant subscription (id={})", id);

                Ok(ok)
            }
            Err(e) => Err(e),
        }
    }

    /// Sends notification through the channel based on the ID of the subscription.
    /// This handles streaming responses.
    fn handle_notification(&self, params: Params<'_>) {
        // retrieve the channel sender for notifying the subscription stream
        let subs = self.subs.borrow();
        let tx = match subs.get(&params.subscription) {
            Some(tx) => tx,
            None => {
                tracing::warn!(
                    id = ?params.subscription,
                    "no subscription exists for the notification ID"
                );
                return;
            }
        };

        // a failure to send the response indicates that the pending request has
        // been dropped in the mean time (and should have been unsubscribed!)
        let _ = tx.send(params.result.to_owned());
    }
}

#[derive(Debug)]
pub enum IpcError {
    InvalidSocket { path: PathBuf, source: io::Error },
    Io(io::Error),
    ServerExit,
}

impl error::Error for IpcError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidSocket { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSocket { path, .. } => write!(f, "invalid IPC socket at {path:?}"),
            Self::Io(io) => write!(f, "{io}"),
            Self::ServerExit => f.write_str("the IPC server has exited unexpectedly"),
        }
    }
}

impl From<io::Error> for IpcError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

fn server_exit() -> Box<TransportError> {
    TransportError::transport(IpcError::ServerExit)
}
