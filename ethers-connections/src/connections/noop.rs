use ethers_core::types::U256;
use serde_json::value::RawValue;

use crate::{err::TransportError, Connection, DuplexConnection, RequestFuture, SubscribeFuture};

/// A noop connection that does nothing and always fails immediately.
#[derive(Clone, Copy, Debug, Default)]
pub struct Noop;

impl Connection for Noop {
    fn request_id(&self) -> u64 {
        0
    }

    fn send_raw_request(&self, _: u64, request: Box<RawValue>) -> RequestFuture<'_> {
        Box::pin(async move {
            Err(TransportError::Transport(
                format!("noop connection requests always fail (request={request})").into(),
            ))
        })
    }
}

impl DuplexConnection for Noop {
    fn subscribe(&self, id: U256) -> SubscribeFuture<'_> {
        Box::pin(async move {
            Err(TransportError::Transport(
                format!("noop connection requests always fail (sub_id={id})").into(),
            ))
        })
    }

    fn unsubscribe(&self, id: U256) -> Result<(), TransportError> {
        Err(TransportError::Transport(
            format!("noop connection requests always fail (sub_id={id})").into(),
        ))
    }
}
