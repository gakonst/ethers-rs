mod err;
mod jsonrpc;
mod provider;
mod sub;
mod transports;
mod types;

use std::{future::Future, ops::Deref, pin::Pin};

use ethers_core::types::U256;
use serde::Serialize;
use serde_json::value::RawValue;
use tokio::sync::mpsc;

pub use crate::provider::{Provider, ProviderError};

#[cfg(all(unix, feature = "ipc"))]
pub use crate::transports::ipc::Ipc;

use crate::{err::TransportError, jsonrpc::Request};

/// ...
pub type RequestFuture<'a> = Pin<Box<dyn Future<Output = ResponsePayload> + 'a>>;
/// ...
pub type ResponsePayload = Result<Box<RawValue>, Box<TransportError>>;

pub trait Transport {
    /// Returns a unique request ID.
    fn request_id(&self) -> u64;

    /// Sends a JSON-RPC request to the underlying API provider and returns its
    /// response.
    ///
    /// The caller has to ensure that `id` is identical to the id encoded in
    /// `request` and that the latter represents a valid JSONRPC 2.0 request
    /// whose contents match the specification defined by the Ethereum
    /// [JSON-RPC API](https://eth.wiki/json-rpc/API).
    ///
    /// # Errors
    ///
    /// ...
    fn send_raw_request(&self, id: u64, request: String) -> RequestFuture<'_>;
}

// blanket impl for all types derefencing to a transport (but not nested refs)
impl<T, D> Transport for D
where
    T: Transport + 'static,
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
pub trait TransportExt: Transport {
    /// Serializes and sends an RPC request for `method` and using `params`.
    ///
    /// In order to match the JSON-RPC specification, `params` must serialize
    /// either to `null` (e.g., with `()`), an array or a map.
    fn send_request<T: Serialize>(&self, method: &str, params: T) -> RequestFuture {
        let id = self.request_id();
        let request = Request { id, method, params }.to_json();
        self.send_raw_request(id, request)
    }
}

impl<T: Transport> TransportExt for T {}
impl TransportExt for dyn Transport + '_ {}

pub trait BidiTransport: Transport {
    fn subscribe(
        &self,
        id: u64,
        request: Box<[u8]>,
    ) -> (U256, mpsc::UnboundedReceiver<ResponsePayload>);
}
