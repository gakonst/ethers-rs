use std::{
    error, fmt, io, mem,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use bytes::{Buf as _, BytesMut};
use serde_json::{value::RawValue, Deserializer};
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::{
        unix::{ReadHalf, WriteHalf},
        UnixStream,
    },
    sync::{mpsc, oneshot},
    task,
};

use ethers_core::types::U256;

use crate::{
    batch::BatchError, jsonrpc as rpc, BatchResponseFuture, Connection, DuplexConnection,
    ResponseFuture, ResponseReceiver, SubscribeFuture,
};

use super::{
    common::{self, FxHashMap, PendingBatchCall},
    ConnectionError,
};

/// The handle for an IPC connection to an Ethereum JSON-RPC provider.
///
/// **Note** Dropping an [`Ipc`] handle will invalidate all pending requests
/// that were made through it.
pub struct Ipc {
    /// The counter for unique request ids.
    next_id: AtomicU64,
    /// The instance for sending requests to the IPC request server
    request_tx: mpsc::UnboundedSender<common::Request>,
}

impl Ipc {
    /// Connects to the IPC socket at `path`.
    ///
    /// # Errors
    ///
    /// Fails, if establishing the connection to the socket fails.
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self, IpcError> {
        let next_id = AtomicU64::new(1);
        let (request_tx, request_rx) = mpsc::unbounded_channel();

        // try to connect to the IPC socket at `path`
        let path = path.as_ref();
        let stream = UnixStream::connect(path)
            .await
            .map_err(|source| IpcError::InvalidSocket { path: path.into(), source })?;

        // spawn an independent IPC server task
        let _ = task::spawn(run_ipc_server(stream, request_rx));

        Ok(Self { next_id, request_tx })
    }
}

impl Connection for Ipc {
    fn request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn send_raw_request(&self, id: u64, request: Box<RawValue>) -> ResponseFuture {
        // send the request to the IPC server
        let (tx, rx) = oneshot::channel();
        let res = self.request_tx.send(common::Request::Call { id, tx, request });

        Box::pin(async move {
            // the send result must be handled WITHIN the future
            res.map_err(|_| server_exit())?;
            // ..., then await the server's response
            rx.await.map_err(|_| server_exit())?
        })
    }

    fn send_raw_batch_request(
        &self,
        ids: Box<[u64]>,
        request: Box<RawValue>,
    ) -> BatchResponseFuture {
        let (tx, rx) = oneshot::channel();
        let res = self.request_tx.send(common::Request::BatchCall { ids, tx, request });

        Box::pin(async move {
            // the send result must be handled WITHIN the future
            res.map_err(|_| server_exit())?;
            // ..., then await the server's response
            rx.await.map_err(|_| server_exit())?
        })
    }
}

impl DuplexConnection for Ipc {
    fn subscribe(&self, id: U256) -> SubscribeFuture {
        // send the subscribe request to the IPC server
        let (tx, rx) = oneshot::channel();
        let res = self.request_tx.send(common::Request::Subscribe { id, tx });

        Box::pin(async move {
            // handle the result & await the response
            res.map_err(|_| server_exit())?;
            let res = rx.await.map_err(|_| server_exit())?;
            Ok(res)
        })
    }

    fn unsubscribe(&self, id: U256) -> Result<(), ConnectionError> {
        self.request_tx.send(common::Request::Unsubscribe { id }).map_err(|_| server_exit())
    }
}

async fn run_ipc_server(mut stream: UnixStream, mut rx: mpsc::UnboundedReceiver<common::Request>) {
    // split stream into read/write halves
    let (mut reader, mut writer) = stream.split();
    let mut shared = Shared::default();

    // create read buffer and next request
    let mut buf = BytesMut::with_capacity(4096);
    let mut next: Option<Box<RawValue>> = None;

    let res = loop {
        tokio::select! {
            // NOTE: writing requests is prioritized over reading incoming msgs
            biased;
            // 1) receive next request (only if there is no previous request)
            msg = rx.recv(), if next.is_none() => match msg {
                // handle the request and set the next request, if necessary
                Some(request) => next = shared.handle_request(request),
                // request channel is closed, i.e., the IPC handle was dropped
                None => break Ok(()),
            },
            // 2) if a request was received, write it out to the IPC socket
            res = shared.handle_writes(&mut writer, &next), if next.is_some() => {
                if res.is_err() {
                    break res;
                }

                // once write is complete & was successful, clear next request
                next = None;
            }
            // 3) receive & handle any incoming response/notification messages
            res = shared.handle_reads(&mut reader, &mut buf) => match res {
                Ok(true) => {
                    // parse the received bytes into 0-n jsonrpc messages
                    let read = match shared.handle_bytes(&buf) {
                        Ok(read) => read,
                        Err(e) => break Err(e),
                    };

                    // split off all bytes that were parsed into complete messages
                    // any remaining bytes that correspond to incomplete messages remain
                    // in the buffer
                    buf.advance(read);
                    continue
                },
                // exit task if the connection was closed or an error occurred
                res => break res.map(|_| ())
            }
        }
    };

    if let Err(e) = res {
        tracing::error!(err = ?e, "exiting IPC server due to error");
    }
}

/// The shared state for the IPC server task.
struct Shared {
    /// The map of pending requests.
    pending: FxHashMap<u64, ResponseReceiver>,
    /// The set of pending batch requests.
    pending_batches: FxHashMap<Box<[u64]>, PendingBatchCall>,
    /// The map of registered subscriptions.
    subs: FxHashMap<U256, common::Subscription>,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            pending: FxHashMap::with_capacity_and_hasher(64, Default::default()),
            pending_batches: FxHashMap::with_capacity_and_hasher(64, Default::default()),
            subs: FxHashMap::with_capacity_and_hasher(64, Default::default()),
        }
    }
}

impl Shared {
    /// Handles a received incoming requests and returns a raw byte buffer, if
    /// the request requires bytes to be written out over the IPC connetion.
    fn handle_request(&mut self, request: common::Request) -> Option<Box<RawValue>> {
        use common::Request::*;
        match request {
            // RPC call requests are inserted into the `pending` map and their
            // payload is extracted to be written out
            Call { id, tx, request } => {
                let prev = self.pending.insert(id, tx);
                assert!(prev.is_none(), "replaced pending IPC request (id={})", id);
                Some(request)
            }
            BatchCall { ids, tx, request } => {
                // using the sorted IDs as key allows checking for presence and
                // completeness in one step
                let mut ids_sorted = ids.clone();
                ids_sorted.sort_unstable();

                let prev = self.pending_batches.insert(ids_sorted, PendingBatchCall { ids, tx });
                if let Some(prev) = prev {
                    panic!("replaced pending IPC batch request (ids={:?})", prev.ids);
                }

                Some(request)
            }
            Subscribe { id, tx } => {
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
                None
            }
            Unsubscribe { id } => {
                // removes the subscription entry and drops the sender half,
                // ending the registered subscription stream (if any)
                // NOTE: if the subscription has not been removed at the
                // provider side as well, it will keep sending further
                // notifications, which will re-create the entry
                let _ = self.subs.remove(&id);
                None
            }
        }
    }

    /// Writes the currently queued request.
    async fn handle_writes(
        &self,
        writer: &mut WriteHalf<'_>,
        next_request: &Option<Box<RawValue>>,
    ) -> Result<(), IpcError> {
        // NOTE: must only be called if `next_request` is set
        let buf = next_request.as_deref().unwrap().get();
        writer.write_all(buf.as_bytes()).await.map_err(Into::into)
    }

    /// Receives a batch
    async fn handle_reads(
        &self,
        reader: &mut ReadHalf<'_>,
        buf: &mut BytesMut,
    ) -> Result<bool, IpcError> {
        // try to read the next batch of bytes into the buffer
        let read = reader.read_buf(buf).await?;
        if read == 0 {
            // eof, socket was closed
            return Ok(false);
        }

        Ok(true)
    }

    fn handle_bytes(&mut self, bytes: &BytesMut) -> Result<usize, IpcError> {
        // deserialize all complete jsonrpc responses contained in the buffer
        let mut de = Deserializer::from_slice(bytes.as_ref()).into_iter::<&RawValue>();
        while let Some(Ok(response)) = de.next() {
            // most likely, the received message is a regular response, so try
            // parsing this first
            if let Ok(rpc::Response { id, result, .. }) = serde_json::from_str(response.get()) {
                self.handle_response(id, Ok(result.to_owned()));
                continue;
            }

            if let Ok(rpc::Notification { params, .. }) = serde_json::from_str(response.get()) {
                self.handle_notification(params);
                continue;
            }

            if let Ok(batch) = rpc::deserialize_batch_response(response.get()) {
                self.handle_batch(batch);
                continue;
            }

            if let Ok(rpc::Error { id, error, .. }) = serde_json::from_str(response.get()) {
                self.handle_response(id, Err(error));
                continue;
            }

            tracing::error!(?response, "received RPC response that matches no expected value");
        }

        Ok(de.byte_offset())
    }

    fn handle_response(&mut self, id: u64, res: Result<Box<RawValue>, rpc::JsonRpcError>) {
        match self.pending.remove(&id) {
            Some(tx) => {
                // if send fails, request has been dropped at the callsite
                let _ = tx.send(res.map_err(ConnectionError::jsonrpc));
            }
            None => tracing::warn!(%id, "no pending request exists for response ID"),
        };
    }

    /// Sends notification through the channel based on the ID of the subscription.
    /// This handles streaming responses.
    fn handle_notification(&mut self, params: rpc::Params<'_>) {
        use std::collections::hash_map::Entry;
        let notification = params.result.to_owned();

        let ok = match self.subs.entry(params.subscription) {
            // the subscription entry has already been inserted (e.g., if the
            // sub has already been registered)
            Entry::Occupied(occ) => {
                let (tx, _) = occ.get();
                tx.send(notification).is_ok()
            }
            // the subscription has not yet been registered, insert a new tx/rx
            // pair and push the current notification to ensure that none get
            // lost
            Entry::Vacant(vac) => {
                let (tx, rx) = mpsc::unbounded_channel();
                // insert the tx/rx pair, which can be taken by the first
                // arriving registration
                let (tx, _) = vac.insert((tx, Some(rx)));
                tx.send(notification).is_ok()
            }
        };

        if !ok {
            // the channel has been dropped without unsubscribing
            let _ = self.subs.remove(&params.subscription);
        }
    }

    fn handle_batch(&mut self, mut batch: Vec<rpc::ResponseOrError<'_>>) {
        let mut ids_sorted = batch.iter().map(|response| response.id()).collect::<Box<[_]>>();
        ids_sorted.sort_unstable();

        if let Some(PendingBatchCall { ids, tx }) = self.pending_batches.remove(&*ids_sorted) {
            // all request IDs exist in the response slice, but not necessarily
            // in the same order
            debug_assert_eq!(batch.len(), ids.len());
            let len = ids.len();
            for i in 0..len {
                for j in i..len {
                    if ids[i] == batch[j].id() && i != j {
                        batch.swap(i, j);
                    }
                }
            }

            // if send fails, the request has been dropped at the callsite
            let responses = batch.into_iter().map(rpc::ResponseOrError::to_result).collect();
            let _ = tx.send(Ok(responses));
            return;
        }

        // no batch exists for the exact set of received response ids,
        // check if one exists for a subset and invalidate it
        self.pending_batches.retain(|key, pending| {
            if ids_sorted.iter().any(|id| key.contains(&id)) {
                let (tx, _) = oneshot::channel();
                let _ = mem::replace(&mut pending.tx, tx).send(Err(BatchError::IncompleteBatch));

                false
            } else {
                true
            }
        });
    }
}

/// An error that occurred when interacting with an IPC server task.
#[derive(Debug)]
pub enum IpcError {
    /// The file at `path` is not a valid IPC socket.
    InvalidSocket { path: PathBuf, source: io::Error },
    /// A generic I/O error while reading from or writing to the socket.
    Io(io::Error),
    /// The IPC server has exited unexpectedly.
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
            Self::Io(err) => err.fmt(f),
            Self::ServerExit => f.write_str("the IPC server has exited unexpectedly"),
        }
    }
}

impl From<io::Error> for IpcError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Wraps a [`ServerExit`](IpcError::ServerExit) error in a [`ConnectionError`].
fn server_exit() -> ConnectionError {
    ConnectionError::Connection(IpcError::ServerExit.into())
}
