mod err;
mod jsonrpc;
mod provider;
mod transports;
mod types;

use std::{future::Future, ops::Deref, pin::Pin};

use serde::Serialize;
use serde_json::value::RawValue;

#[cfg(all(unix, feature = "ipc"))]
use crate::transports::ipc::Ipc;
use crate::{err::TransportError, jsonrpc::Request};

/// ...
pub type RequestFuture = Pin<Box<dyn Future<Output = ResponsePayload>>>;
/// ...
pub type ResponsePayload = Result<Box<RawValue>, Box<TransportError>>;

pub trait Transport {
    /// Returns a unique request ID.
    fn request_id(&self) -> u64;

    /// Sends a JSON-RPC request to the underlying API provider and returns its
    /// response.
    ///
    /// The caller has to ensure, that the given `request` represents a valid
    /// JSONRPC 2.0 request whose contents match the specification defined by
    /// the Ethereum [JSON-RPC API](https://eth.wiki/json-rpc/API).
    ///
    /// # Errors
    ///
    /// ...
    fn send_raw_request(&self, request: String) -> RequestFuture;
}

// blanket impl for all types derefencing to a transport
impl<T, D> Transport for D
where
    T: Transport,
    D: Deref<Target = T>,
{
    fn request_id(&self) -> u64 {
        self.deref().request_id()
    }

    fn send_raw_request(&self, request: String) -> Pin<Box<dyn Future<Output = ResponsePayload>>> {
        self.deref().send_raw_request(request)
    }
}

pub trait TransportExt: Transport {
    fn send_request<T: Serialize>(&self, method: &str, params: T) -> RequestFuture {
        let request = Request { id: self.request_id(), method, params }.to_json();
        self.send_raw_request(request)
    }
}

impl<T: Transport> TransportExt for T {}
impl TransportExt for dyn Transport + '_ {}

pub trait BidiTransport: Transport {}
