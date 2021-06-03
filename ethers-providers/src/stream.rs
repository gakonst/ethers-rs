use crate::{JsonRpcClient, Middleware, PinBoxFut, Provider, ProviderError};

use ethers_core::types::{U256, Transaction, TxHash};

use futures_core::stream::Stream;
use futures_timer::Delay;
use futures_util::{stream, FutureExt, StreamExt, TryFutureExt};
use pin_project::pin_project;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
    vec::IntoIter,
};
use futures_core::Future;
use futures_util::stream::FuturesUnordered;
use std::collections::VecDeque;

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



type TransactionFut<'a> = Pin<Box<dyn Future<Output = TransactionResult> + 'a>>;

type TransactionResult = Result<Transaction, ProviderError>;

struct TxStream<'a, P> {
    pending: FuturesUnordered<TransactionFut<'a>>,
    buffered: VecDeque<TxHash>,
    provider: &'a Provider<P>,
    watcher: FilterWatcher<'a, P, TxHash>,
    max_concurrent: usize,
}

impl<'a, P: JsonRpcClient> Stream for TxStream<'a, P> {
    type Item = TransactionResult;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if let tx @ Poll::Ready(Some(_)) = this.pending.poll_next_unpin(cx) {
            return tx;
        }

        while this.max_concurrent < this.max_concurrent {
            if let Some(_) = this.buffered.pop_front() {
                // TODO add get_tx again
            } else {
                break;
            }
        }

        let mut watcher_done = false;
        loop {
            match Stream::poll_next(Pin::new(&mut this.watcher), cx) {
                Poll::Ready(Some(tx)) => {
                    if this.pending.len() < this.max_concurrent {
                        this.pending
                            .push(Box::pin(this.provider.get_transaction(tx).and_then(
                                |res| {
                                    if let Some(tx) = res {
                                        futures_util::future::ok(tx)
                                    } else {
                                        futures_util::future::err(ProviderError::CustomError(
                                            "Not found".to_string(),
                                        ))
                                    }
                                },
                            )));
                    } else {
                        this.buffered.push_back(tx);
                    }
                }
                Poll::Ready(None) => {
                    watcher_done = true;
                    break;
                }
                _ => break,
            }
        }

        if watcher_done && this.pending.is_empty() {
            // all done
            return Poll::Ready(None);
        }

        Poll::Pending
    }
}
