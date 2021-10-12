use crate::{JsonRpcClient, Middleware, PinBoxFut, Provider, ProviderError};

use ethers_core::types::{Transaction, TxHash, U256};

use futures_core::{stream::Stream, Future};
use futures_util::{stream, stream::FuturesUnordered, FutureExt, StreamExt};
use pin_project::pin_project;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::VecDeque,
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
    vec::IntoIter,
};

#[cfg(not(target_arch = "wasm32"))]
use futures_timer::Delay;
#[cfg(target_arch = "wasm32")]
use wasm_timer::Delay;

// https://github.com/tomusdrw/rust-web3/blob/befcb2fb8f3ca0a43e3081f68886fa327e64c8e6/src/api/eth_filter.rs#L20
pub fn interval(duration: Duration) -> impl Stream<Item = ()> + Send + Unpin {
    stream::unfold((), move |_| Delay::new(duration).map(|_| Some(((), ())))).map(drop)
}

/// The default polling interval for filters and pending transactions
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(7000);

enum FilterWatcherState<'a, R> {
    WaitForInterval,
    GetFilterChanges(PinBoxFut<'a, Vec<R>>),
    NextItem(IntoIter<R>),
}

#[must_use = "filters do nothing unless you stream them"]
#[pin_project]
/// Streams data from an installed filter via `eth_getFilterChanges`
pub struct FilterWatcher<'a, P, R> {
    /// The filter's installed id on the ethereum node
    pub id: U256,

    provider: &'a Provider<P>,

    // The polling interval
    interval: Box<dyn Stream<Item = ()> + Send + Unpin>,

    state: FilterWatcherState<'a, R>,
}

impl<'a, P, R> FilterWatcher<'a, P, R>
where
    P: JsonRpcClient,
    R: Send + Sync + DeserializeOwned,
{
    /// Creates a new watcher with the provided factory and filter id.
    pub fn new<T: Into<U256>>(id: T, provider: &'a Provider<P>) -> Self {
        Self {
            id: id.into(),
            interval: Box::new(interval(DEFAULT_POLL_INTERVAL)),
            state: FilterWatcherState::WaitForInterval,
            provider,
        }
    }

    /// Sets the stream's polling interval
    pub fn interval(mut self, duration: Duration) -> Self {
        self.interval = Box::new(interval(duration));
        self
    }

    /// Alias for Box::pin, must be called in order to pin the stream and be able
    /// to call `next` on it.
    pub fn stream(self) -> Pin<Box<Self>> {
        Box::pin(self)
    }
}

// Pattern for flattening the returned Vec of filter changes taken from
// https://github.com/tomusdrw/rust-web3/blob/f043b222744580bf4be043da757ab0b300c3b2da/src/api/eth_filter.rs#L50-L67
impl<'a, P, R> Stream for FilterWatcher<'a, P, R>
where
    P: JsonRpcClient,
    R: Serialize + Send + Sync + DeserializeOwned + Debug + 'a,
{
    type Item = R;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let id = *this.id;

        *this.state = match this.state {
            FilterWatcherState::WaitForInterval => {
                // Wait the polling period
                let _ready = futures_util::ready!(this.interval.poll_next_unpin(cx));

                // create a new instance of the future
                cx.waker().wake_by_ref();
                let fut = Box::pin(this.provider.get_filter_changes(id));
                FilterWatcherState::GetFilterChanges(fut)
            }
            FilterWatcherState::GetFilterChanges(fut) => {
                // NOTE: If the provider returns an error, this will return an empty
                // vector. Should we make this return a Result instead? Ideally if we're
                // in a streamed loop we wouldn't want the loop to terminate if an error
                // is encountered (since it might be a temporary error).
                let items: Vec<R> = futures_util::ready!(fut.as_mut().poll(cx)).unwrap_or_default();
                cx.waker().wake_by_ref();
                FilterWatcherState::NextItem(items.into_iter())
            }
            // Consume 1 element from the vector. If more elements are in the vector,
            // the next call will immediately go to this branch instead of trying to get
            // filter changes again. Once the whole vector is consumed, it will poll again
            // for new logs
            FilterWatcherState::NextItem(iter) => {
                cx.waker().wake_by_ref();
                match iter.next() {
                    Some(item) => return Poll::Ready(Some(item)),
                    None => FilterWatcherState::WaitForInterval,
                }
            }
        };

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

type TransactionFut<'a> = Pin<Box<dyn Future<Output = TransactionResult> + 'a>>;

type TransactionResult = Result<Transaction, GetTransactionError>;

/// Drains a stream of transaction hashes and yields entire `Transaction`.
#[must_use = "streams do nothing unless polled"]
pub struct TransactionStream<'a, P, St> {
    /// Currently running futures pending completion.
    pending: FuturesUnordered<TransactionFut<'a>>,
    /// Temporary buffered transaction that get started as soon as another future finishes.
    buffered: VecDeque<TxHash>,
    /// The provider that gets the transaction
    provider: &'a Provider<P>,
    /// A stream of transaction hashes.
    stream: St,
    /// max allowed futures to execute at once.
    max_concurrent: usize,
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
    fn push_tx(&mut self, tx: TxHash) {
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

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use crate::{Http, Ws};
    use ethers_core::{
        types::{TransactionReceipt, TransactionRequest},
        utils::{Ganache, Geth},
    };
    use futures_util::{FutureExt, StreamExt};
    use std::{collections::HashSet, convert::TryFrom};

    #[tokio::test]
    async fn can_stream_pending_transactions() {
        let num_txs = 5;
        let geth = Geth::new().block_time(2u64).spawn();
        let provider = Provider::<Http>::try_from(geth.endpoint())
            .unwrap()
            .interval(Duration::from_millis(1000));
        let ws = Ws::connect(geth.ws_endpoint()).await.unwrap();
        let ws_provider = Provider::new(ws);

        let accounts = provider.get_accounts().await.unwrap();
        let tx = TransactionRequest::new().from(accounts[0]).to(accounts[0]).value(1e18 as u64);

        let mut sending = futures_util::future::join_all(
            std::iter::repeat(tx.clone()).take(num_txs).map(|tx| async {
                provider.send_transaction(tx, None).await.unwrap().await.unwrap().unwrap()
            }),
        )
        .fuse();

        let mut watch_tx_stream = provider
            .watch_pending_transactions()
            .await
            .unwrap()
            .transactions_unordered(num_txs)
            .fuse();

        let mut sub_tx_stream =
            ws_provider.subscribe_pending_txs().await.unwrap().transactions_unordered(2).fuse();

        let mut sent: Option<Vec<TransactionReceipt>> = None;
        let mut watch_received: Vec<Transaction> = Vec::with_capacity(num_txs);
        let mut sub_received: Vec<Transaction> = Vec::with_capacity(num_txs);

        loop {
            futures_util::select! {
                txs = sending => {
                    sent = Some(txs)
                },
                tx = watch_tx_stream.next() => watch_received.push(tx.unwrap().unwrap()),
                tx = sub_tx_stream.next() => sub_received.push(tx.unwrap().unwrap()),
            };
            if watch_received.len() == num_txs && sub_received.len() == num_txs {
                if let Some(ref sent) = sent {
                    assert_eq!(sent.len(), watch_received.len());
                    let sent_txs =
                        sent.iter().map(|tx| tx.transaction_hash).collect::<HashSet<_>>();
                    assert_eq!(sent_txs, watch_received.iter().map(|tx| tx.hash).collect());
                    assert_eq!(sent_txs, sub_received.iter().map(|tx| tx.hash).collect());
                    break
                }
            }
        }
    }

    #[tokio::test]
    async fn can_stream_transactions() {
        let ganache = Ganache::new().block_time(2u64).spawn();
        let provider = Provider::<Http>::try_from(ganache.endpoint())
            .unwrap()
            .with_sender(ganache.addresses()[0]);

        let accounts = provider.get_accounts().await.unwrap();

        let tx = TransactionRequest::new().from(accounts[0]).to(accounts[0]).value(1e18 as u64);

        let txs =
            futures_util::future::join_all(std::iter::repeat(tx.clone()).take(3).map(|tx| async {
                provider.send_transaction(tx, None).await.unwrap().await.unwrap()
            }))
            .await;

        let stream = TransactionStream::new(
            &provider,
            stream::iter(txs.iter().cloned().map(|tx| tx.unwrap().transaction_hash)),
            10,
        );
        let res =
            stream.collect::<Vec<_>>().await.into_iter().collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(res.len(), txs.len());
        assert_eq!(
            res.into_iter().map(|tx| tx.hash).collect::<HashSet<_>>(),
            txs.into_iter().map(|tx| tx.unwrap().transaction_hash).collect()
        );
    }
}
