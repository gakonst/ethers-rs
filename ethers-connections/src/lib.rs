pub mod batch;
pub mod connection;
pub mod types;

mod call;
mod jsonrpc;
mod pending;
mod provider;
mod sub;

use std::{future::Future, ops::Deref, pin::Pin};

use serde_json::value::RawValue;
use tokio::sync::{mpsc, oneshot};

use ethers_core::types::U256;

pub use crate::{
    call::{CallParams, RpcCall},
    pending::PendingTransaction,
    provider::{Provider, ProviderError},
    sub::SubscriptionStream,
};

use crate::{batch::BatchError, connection::ConnectionError};

#[cfg(target_arch = "wasm32")]
type DynFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
#[cfg(not(target_arch = "wasm32"))]
type DynFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The payload of a response received over a [`Connection`].
pub type ResponsePayload = Result<Box<RawValue>, ConnectionError>;
/// The payload of a batch response received over a [`Connection`].
pub type BatchResponsePayload = Result<Vec<Result<Box<RawValue>, ConnectionError>>, BatchError>;

/// The receiver channel for a [`ResponsePayload`].
pub type ResponseReceiver = oneshot::Sender<ResponsePayload>;

/// The [`Future`] resolving to a [`Connection`]'s response to a request.
pub type ResponseFuture = DynFuture<'static, ResponsePayload>;
/// The [`Future`] resolving to a [`Connection`]'s response to a batch request.
pub type BatchResponseFuture = DynFuture<'static, BatchResponsePayload>;

/// A connection allowing the exchange of Ethereum API JSON-RPC messages between
/// a local client and a remote API provider.
pub trait Connection: Send + Sync {
    /// Returns a unique request ID.
    fn request_id(&self) -> u64;

    /// Sends a JSON-RPC request to the connected API provider and returns its
    /// response.
    ///
    /// The caller has to ensure that `id` is identical to the id encoded in
    /// `request` and that the latter represents a valid JSONRPC 2.0 request
    /// whose contents match the specification defined by the Ethereum
    /// [JSON-RPC API](https://eth.wiki/json-rpc/API).
    fn send_raw_request(&self, id: u64, request: Box<RawValue>) -> ResponseFuture;

    /// Sends a JSON-RPC batch request to the connected API provider and returns
    /// its response.
    ///
    /// The caller has to ensure that for each ID in `ids` there is a
    /// corresponding valid JSON object in `request`, which must be formatted as
    /// an array.
    ///
    /// The implementation has to ensure, that the order of returned responses
    /// matches the order of the given `ids`.
    fn send_raw_batch_request(
        &self,
        ids: Box<[u64]>,
        request: Box<RawValue>,
    ) -> BatchResponseFuture;
}

// blanket impl for all types derefencing to a Connection
impl<C, D> Connection for D
where
    C: Connection + ?Sized,
    D: Deref<Target = C> + Send + Sync,
{
    fn request_id(&self) -> u64 {
        self.deref().request_id()
    }

    fn send_raw_request(&self, id: u64, request: Box<RawValue>) -> ResponseFuture {
        self.deref().send_raw_request(id, request)
    }

    fn send_raw_batch_request(
        &self,
        ids: Box<[u64]>,
        request: Box<RawValue>,
    ) -> BatchResponseFuture {
        self.deref().send_raw_batch_request(ids, request)
    }
}

/// The future returned by [`DuplexConnection::subscribe`] that resolves to the
/// ID of the subscription and the channel receiver for all notifications
/// received for this subscription.
pub type SubscribeFuture = DynFuture<'static, SubscribePayload>;

/// The payload of a response to a subscribe request.
pub type SubscribePayload = Result<Option<NotificationReceiver>, ConnectionError>;

/// The receiver channel half for subscription notifications.
pub type NotificationReceiver = mpsc::UnboundedReceiver<Box<RawValue>>;

/// A [`Connection`] that allows publish/subscribe communication with the API
/// provider.
pub trait DuplexConnection: Connection {
    /// Subscribes to all notifications received for the given `id` and returns
    /// a [`NotificationReceiver`] for them.
    ///
    /// Additionaly, a RPC call to `eth_subscribe` is necessary, otherwise, no
    /// notifications will be received.
    /// If the ID is already subscribed to, `None` is returned.
    fn subscribe(&self, id: U256) -> SubscribeFuture;

    /// Unsubscribes to all notifications received for the given `id`.
    ///
    /// A previous RPC call to `eth_unsubscribe` is necessary, otherwise, the
    /// provider will continue to send further notifications for this ID.
    fn unsubscribe(&self, id: U256) -> Result<(), ConnectionError>;
}

// blanket impl for all types derefencing to a DuplexConnection
impl<C, D> DuplexConnection for D
where
    C: DuplexConnection + ?Sized,
    D: Deref<Target = C> + Send + Sync,
{
    fn subscribe(&self, id: U256) -> SubscribeFuture {
        self.deref().subscribe(id)
    }

    fn unsubscribe(&self, id: U256) -> Result<(), ConnectionError> {
        self.deref().unsubscribe(id)
    }
}

#[cfg(test)]
fn block_on(future: impl Future<Output = ()>) {
    use tokio::runtime::Builder;
    Builder::new_current_thread().enable_all().build().unwrap().block_on(future);
}
