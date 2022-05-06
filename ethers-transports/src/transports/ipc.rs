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
    jsonrpc::{JsonRpcError, Params, Response},
    RequestFuture, Transport,
};

type FxHashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<FxHasher64>>;
type PendingRequest = oneshot::Sender<Result<Box<RawValue>, JsonRpcError>>;
type PendingSubscription = oneshot::Sender<(U256, mpsc::UnboundedReceiver<Box<RawValue>>)>;

enum IpcRequest {
    Call { id: u64, tx: PendingRequest, request: String },
}

/// The handle for an IPC connection to an Ethereum JSON-RPC provider.
///
/// **Note** Dropping an [`Ipc`] handle will invalidate all pending requests
/// that were made through it.
pub struct Ipc {
    /// The counter for unique request ids.
    next_id: AtomicU64,
    /// The instance for sending requests to the IPC request server
    request_tx: mpsc::UnboundedSender<IpcRequest>,
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

impl Transport for Ipc {
    fn request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn send_raw_request(&self, id: u64, request: String) -> RequestFuture<'_> {
        Box::pin(async move {
            let (tx, mut rx) = oneshot::channel();
            self.request_tx
                .send(IpcRequest::Call { id, tx, request })
                .map_err(|_| TransportError::transport(IpcError::ServerExit))?;

            let response = rx
                .await
                .map_err(|_| TransportError::transport(IpcError::ServerExit))?
                .map_err(|err| TransportError::jsonrpc(err))?;

            Ok(response)
        })
    }
}

fn spawn_ipc_server(stream: UnixStream, request_rx: mpsc::UnboundedReceiver<IpcRequest>) {
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

async fn run_ipc_server(mut stream: UnixStream, request_rx: mpsc::UnboundedReceiver<IpcRequest>) {
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
    pending: RefCell<FxHashMap<u64, PendingRequest>>,
    subs: RefCell<FxHashMap<U256, PendingSubscription>>,
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
        mut request_rx: mpsc::UnboundedReceiver<IpcRequest>,
    ) -> Result<(), IpcError> {
        use IpcRequest::*;

        while let Some(msg) = request_rx.recv().await {
            match msg {
                Call { id, tx, request } => {
                    let prev = self.pending.borrow_mut().insert(id, tx);
                    assert!(prev.is_none(), "replaced pending IPC request (id={})", id);

                    if let Err(err) = writer.write_all(request.as_bytes()).await {
                        tracing::error!("IPC connection error: {:?}", err);
                        self.pending.borrow_mut().remove(&id);
                    }
                } /*Subscribe { id, sink } => {
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
                  }*/
            }
        }

        // if the request receiver is closed, the IPC handle must have been
        // dropped, ...
        Ok(())
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
                return;
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
                return;
            }
        };

        // a failure to send the response indicates that the pending request has
        // been dropped in the mean time (and should have been unsubscribed!)
        let _ = tx.send((params.subscription, params.result.to_owned()));
    }
}

#[derive(Debug)]
pub enum IpcError {
    InvalidSocket { path: PathBuf, source: io::Error },
    Io(io::Error),
    ServerExit,
    //Other(&'static str),
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
        todo!()
    }
}

impl From<io::Error> for IpcError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
