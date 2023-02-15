use std::{
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::{stream::Stream, Future};
use futures_util::{
    self,
    stream::{FuturesUnordered, StreamExt},
    FutureExt,
};

use ethers_core::types::{Transaction, TxHash};

use crate::{FilterWatcher, JsonRpcClient, Middleware, Provider, ProviderError};

/// Errors `TransactionStream` can throw
#[derive(Debug, thiserror::Error)]
pub enum GetTransactionError {
    #[error("Failed to get transaction `{0}`: {1}")]
    ProviderError(TxHash, ProviderError),
    /// `get_transaction` resulted in a `None`
    #[error("Transaction `{0}` not found")]
    NotFound(TxHash),
}

impl From<GetTransactionError> for ProviderError {
    fn from(err: GetTransactionError) -> Self {
        match err {
            GetTransactionError::ProviderError(_, err) => err,
            err @ GetTransactionError::NotFound(_) => ProviderError::CustomError(err.to_string()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type TransactionFut<'a> = Pin<Box<dyn Future<Output = TransactionResult> + Send + 'a>>;

#[cfg(target_arch = "wasm32")]
pub(crate) type TransactionFut<'a> = Pin<Box<dyn Future<Output = TransactionResult> + 'a>>;

pub(crate) type TransactionResult = Result<Transaction, GetTransactionError>;

/// Drains a stream of transaction hashes and yields entire `Transaction`.
#[must_use = "streams do nothing unless polled"]
pub struct TransactionStream<'a, P, St> {
    /// Currently running futures pending completion.
    pub(crate) pending: FuturesUnordered<TransactionFut<'a>>,
    /// Temporary buffered transaction that get started as soon as another future finishes.
    pub(crate) buffered: VecDeque<TxHash>,
    /// The provider that gets the transaction
    pub(crate) provider: &'a Provider<P>,
    /// A stream of transaction hashes.
    pub(crate) stream: St,
    /// max allowed futures to execute at once.
    pub(crate) max_concurrent: usize,
}

impl<'a, P: JsonRpcClient, St> TransactionStream<'a, P, St> {
    /// Create a new `TransactionStream` instance
    pub fn new(provider: &'a Provider<P>, stream: St, max_concurrent: usize) -> Self {
        Self {
            pending: Default::default(),
            buffered: Default::default(),
            provider,
            stream,
            max_concurrent,
        }
    }

    /// Push a future into the set
    pub(crate) fn push_tx(&mut self, tx: TxHash) {
        let fut = self.provider.get_transaction(tx).then(move |res| match res {
            Ok(Some(tx)) => futures_util::future::ok(tx),
            Ok(None) => futures_util::future::err(GetTransactionError::NotFound(tx)),
            Err(err) => futures_util::future::err(GetTransactionError::ProviderError(tx, err)),
        });
        self.pending.push(Box::pin(fut));
    }
}

impl<'a, P, St> Stream for TransactionStream<'a, P, St>
where
    P: JsonRpcClient,
    St: Stream<Item = TxHash> + Unpin + 'a,
{
    type Item = TransactionResult;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // drain buffered transactions first
        while this.pending.len() < this.max_concurrent {
            if let Some(tx) = this.buffered.pop_front() {
                this.push_tx(tx);
            } else {
                break
            }
        }

        let mut stream_done = false;
        loop {
            match Stream::poll_next(Pin::new(&mut this.stream), cx) {
                Poll::Ready(Some(tx)) => {
                    if this.pending.len() < this.max_concurrent {
                        this.push_tx(tx);
                    } else {
                        this.buffered.push_back(tx);
                    }
                }
                Poll::Ready(None) => {
                    stream_done = true;
                    break
                }
                _ => break,
            }
        }

        // poll running futures
        if let tx @ Poll::Ready(Some(_)) = this.pending.poll_next_unpin(cx) {
            return tx
        }

        if stream_done && this.pending.is_empty() {
            // all done
            return Poll::Ready(None)
        }

        Poll::Pending
    }
}

impl<'a, P> FilterWatcher<'a, P, TxHash>
where
    P: JsonRpcClient,
{
    /// Returns a stream that yields the `Transaction`s for the transaction hashes this stream
    /// yields.
    ///
    /// This internally calls `Provider::get_transaction` with every new transaction.
    /// No more than n futures will be buffered at any point in time, and less than n may also be
    /// buffered depending on the state of each future.
    pub fn transactions_unordered(self, n: usize) -> TransactionStream<'a, P, Self> {
        TransactionStream::new(self.provider, self, n)
    }
}
