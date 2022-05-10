pub mod connections {
    //! The parent module containing all [`Connection`](crate::Connection)
    //! implementations.

    #[cfg(feature = "http")]
    pub mod http;
    #[cfg(all(unix, feature = "ipc"))]
    pub mod ipc;
    #[cfg(feature = "ws")]
    pub mod ws;

    pub mod noop;
}

mod err;
mod jsonrpc;
mod provider;
mod sub;
mod types;

use std::{future::Future, ops::Deref, pin::Pin};

use ethers_core::types::U256;
use serde::Serialize;
use serde_json::value::RawValue;
use tokio::sync::mpsc;

pub use crate::provider::{Provider, ProviderError};
pub use crate::sub::SubscriptionStream;

#[cfg(all(unix, feature = "ipc"))]
pub use crate::connections::ipc::Ipc;

use crate::{err::TransportError, jsonrpc::Request};

#[cfg(target_arch = "wasm32")]
type DynFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
#[cfg(not(target_arch = "wasm32"))]
type DynFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The future returned by [`Transport::send_raw_request`] that resolves to the
/// JSON value returned by the transport.
pub type RequestFuture<'a> = DynFuture<'a, ResponsePayload>;

/// The payload of a request response from a transport.
pub type ResponsePayload = Result<Box<RawValue>, Box<TransportError>>;

pub trait Connection {
    /// Returns a unique request ID.
    fn request_id(&self) -> u64;

    /// Sends a JSON-RPC request to the underlying API provider and returns its
    /// response.
    ///
    /// The caller has to ensure that `id` is identical to the id encoded in
    /// `request` and that the latter represents a valid JSONRPC 2.0 request
    /// whose contents match the specification defined by the Ethereum
    /// [JSON-RPC API](https://eth.wiki/json-rpc/API).
    fn send_raw_request(&self, id: u64, request: String) -> RequestFuture<'_>;
}

// blanket impl for all types derefencing to a transport (but not nested refs)
impl<T, D> Connection for D
where
    T: Connection + ?Sized + 'static,
    D: Deref<Target = T>,
{
    fn request_id(&self) -> u64 {
        self.deref().request_id()
    }

    fn send_raw_request(&self, id: u64, request: String) -> RequestFuture<'_> {
        self.deref().send_raw_request(id, request)
    }
}

/// A trait providing additional convenience methods for the [`Transport`] trait.
pub trait ConnectionExt: Connection {
    /// Serializes and sends an RPC request for `method` and using `params`.
    ///
    /// In order to match the JSON-RPC specification, `params` must serialize
    /// either to `null` (e.g., with `()`), an array or a map.
    fn send_request<T: Serialize>(&self, method: &str, params: T) -> RequestFuture<'_> {
        let id = self.request_id();
        let request = Request { id, method, params }.to_json();
        self.send_raw_request(id, request)
    }
}

impl<T: Connection> ConnectionExt for T {}
impl ConnectionExt for dyn Connection + '_ {}

/// The future returned by [`DuplexTransport::subscribe`] that resolves to the
/// ID of the subscription and the channel receiver for all notifications
/// received for this subscription.
pub type SubscribeFuture<'a> = DynFuture<'a, SubscribePayload>;

/// ...
pub type UnsubscribeFuture<'a> = DynFuture<'a, Result<bool, Box<TransportError>>>;

/// ...
pub type SubscribePayload = Result<(U256, NotificationReceiver), Box<TransportError>>;

/// ...
pub type UnsubscribePayload = Result<bool, Box<TransportError>>;

/// ...
pub type NotificationReceiver = mpsc::UnboundedReceiver<Box<RawValue>>;

/// A [`Connection`] that allows publish/subscribe communication.
pub trait DuplexConnection: Connection {
    /// Sends a JSON-RPC subscribe request to the transport and returns
    /// the resulting subscription ID and a receiver for all notifications
    /// associated with that ID, if successful.
    ///
    /// The caller has to ensure that `id` is identical to the id encoded in
    /// `request` and that the latter represents a valid JSONRPC 2.0 request
    /// whose contents match the specification defined by the Ethereum
    /// [JSON-RPC Publish/Subscribe API](https://geth.ethereum.org/docs/rpc/pubsub).
    fn subscribe(&self, id: u64, request: String) -> SubscribeFuture<'_>;

    /// Sends a JSON-RPC unsubscribe request to the transport and returns its
    /// result, if successful.
    ///
    /// The implementation has to ensure, that the request can be sent out
    /// *before* the [`Future`] returned by this method is polled.
    fn unsubscribe(&self, id: &U256) -> UnsubscribeFuture<'_>;
}
