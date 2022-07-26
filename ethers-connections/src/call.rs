use std::{
    future::Future,
    marker::PhantomData,
    mem,
    pin::Pin,
    task::{Context, Poll},
};

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::{
    connection::ConnectionError, jsonrpc as rpc, Connection, ProviderError, ResponseFuture,
};

/// A [`Future`] that resolves to the result of an JSONRPC call.
pub struct RpcCall<C, R> {
    state: CallState<C>,
    _marker: PhantomData<fn() -> R>,
}

impl<C, R> RpcCall<C, R> {
    pub(crate) fn new(connection: C, params: CallParams) -> Self {
        Self { state: CallState::Prepared { connection, params }, _marker: PhantomData }
    }

    /// Converts the RPC call into its request parameters.
    ///
    /// # Panics
    ///
    /// Panics, if the call has already been polled at least once.
    pub fn to_params(self) -> CallParams {
        match self.state {
            CallState::Prepared { params, .. } => params,
            _ => panic!("rpc call future has already been polled"),
        }
    }
}

impl<C, R> RpcCall<C, R>
where
    C: Connection + ToOwned,
    C::Owned: Connection,
{
    /// ```
    /// # use std::{thread, sync::Arc};
    /// use ethers_connections::{Connection, Provider, connection::noop};
    /// let provider: Provider<Arc<dyn Connection>> = Provider { connection: Arc::new(noop::Noop) };
    ///
    /// // call borrows the underlying connection and can, e.g., not be moved to
    /// // a different task or thread
    /// let call = provider.get_block_number();
    /// let call = call.to_owned();
    /// # thread::spawn(move || { let _ = call; });
    /// ```
    pub fn to_owned(self) -> RpcCall<C::Owned, R> {
        match self.state {
            CallState::Prepared { connection, params } => {
                let connection = connection.to_owned();
                RpcCall { state: CallState::Prepared { connection, params }, _marker: PhantomData }
            }
            _ => panic!("rpc call future has already been polled"),
        }
    }
}

impl<C, R> RpcCall<C, R>
where
    R: for<'de> Deserialize<'de>,
{
    fn handle_poll(
        poll: Poll<(&'static str, Result<Box<RawValue>, ConnectionError>)>,
    ) -> Poll<Result<R, Box<ProviderError>>> {
        match poll {
            Poll::Ready((method, Ok(response))) => {
                let res = serde_json::from_str(response.get()).map_err(|err| {
                    ProviderError::json(err).with_ctx(format!(
                        "failed RPC call to `{method}` (response deserialization failed)",
                    ))
                });

                Poll::Ready(res)
            }
            Poll::Ready((method, Err(e))) => Poll::Ready(Err(e
                .to_provider_err()
                .with_ctx(format!("failed RPC call to `{method}` (rpc request failed)")))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<C: Connection + Unpin, R: for<'de> Deserialize<'de>> Future for RpcCall<C, R> {
    type Output = Result<R, Box<ProviderError>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = Pin::new(&mut self.get_mut().state);
        let poll = state.poll(cx);
        Self::handle_poll(poll)
    }
}

/// The parameters for a JSON-RPC call.
#[derive(Clone, Debug)]
pub struct CallParams {
    pub id: u64,
    pub method: &'static str,
    pub request: Box<RawValue>,
}

impl CallParams {
    pub(crate) fn new<T: Serialize>(id: u64, method: &'static str, params: T) -> Self {
        debug_assert!(id != 0);
        let request = rpc::Request { id, method, params }.to_json();
        Self { id, method, request }
    }
}

/// The current poll state of an [`RpcCall`] future.
enum CallState<C> {
    /// All call parameters are prepared and the future has never been polled.
    Prepared { connection: C, params: CallParams },
    /// The future has been polled at least once and the initial call parameters
    /// have been consumed.
    Polled { future: ResponseFuture, method: &'static str },
    /// The future has been polled to completion.
    Completed,
}

impl<C> Future for CallState<C>
where
    C: Connection + Unpin,
{
    type Output = (&'static str, Result<Box<RawValue>, ConnectionError>);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.get_mut();
        match state {
            Self::Prepared { connection, params } => {
                let method = params.method;
                let request = mem::replace(&mut params.request, Box::default());

                let mut future = connection.send_raw_request(params.id, request);
                match future.as_mut().poll(cx) {
                    Poll::Ready(res) => {
                        *state = Self::Completed;
                        Poll::Ready((method, res))
                    }
                    Poll::Pending => {
                        *state = Self::Polled { future, method };
                        Poll::Pending
                    }
                }
            }
            Self::Polled { future, method } => {
                let mut future = future.as_mut();
                let method = *method;

                match future.as_mut().poll(cx) {
                    Poll::Ready(res) => {
                        *state = Self::Completed;
                        Poll::Ready((method, res))
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
            Self::Completed => panic!("rpc call future already completed"),
        }
    }
}
