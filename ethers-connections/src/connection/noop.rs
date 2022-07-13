use serde_json::value::RawValue;

use ethers_core::types::U256;

use crate::{BatchResponseFuture, Connection, DuplexConnection, ResponseFuture, SubscribeFuture};

use super::ConnectionError;

/// A noop connection that does nothing and always fails immediately.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Noop;

impl Connection for Noop {
    fn request_id(&self) -> u64 {
        1
    }

    fn send_raw_request(&self, _: u64, request: Box<RawValue>) -> ResponseFuture {
        Box::pin(async move {
            Err(ConnectionError::Connection(
                format!("noop connection requests always fail (request={request})").into(),
            ))
        })
    }

    fn send_raw_batch_request(&self, _: Box<[u64]>, request: Box<RawValue>) -> BatchResponseFuture {
        Box::pin(async move {
            Err(ConnectionError::Connection(
                format!("noop connection requests always fail (request={request})").into(),
            )
            .into())
        })
    }
}

impl DuplexConnection for Noop {
    fn subscribe(&self, id: U256) -> SubscribeFuture {
        Box::pin(async move {
            Err(ConnectionError::Connection(
                format!("noop connection requests always fail (sub_id={id})").into(),
            ))
        })
    }

    fn unsubscribe(&self, id: U256) -> Result<(), ConnectionError> {
        Err(ConnectionError::Connection(
            format!("noop connection requests always fail (sub_id={id})").into(),
        ))
    }
}
