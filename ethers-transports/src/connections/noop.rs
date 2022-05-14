use ethers_core::types::U256;

use crate::{err::TransportError, Connection, DuplexConnection, RequestFuture, SubscribeFuture};

/// A noop connection that does nothing and always fails immediately.
pub struct Noop;

impl Connection for Noop {
    fn request_id(&self) -> u64 {
        0
    }

    fn send_raw_request(&self, _: u64, request: String) -> RequestFuture<'_> {
        Box::pin(async move {
            Err(Box::new(TransportError::Transport(
                format!("noop connection requests always fail (request={request})").into(),
            )))
        })
    }
}

impl DuplexConnection for Noop {
    fn subscribe(&self, id: U256) -> SubscribeFuture<'_> {
        Box::pin(async move {
            Err(Box::new(TransportError::Transport(
                format!("noop connection requests always fail (sub_id={id})").into(),
            )))
        })
    }

    fn unsubscribe(&self, id: U256) -> Result<(), Box<TransportError>> {
        Err(Box::new(TransportError::Transport(
            format!("noop connection requests always fail (sub_id={id})").into(),
        )))
    }
}
