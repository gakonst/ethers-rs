use crate::{JsonRpcClient, Middleware, PinBoxFut, Provider};

use ethers_core::types::U256;

use futures_core::stream::Stream;
use futures_timer::Delay;
use futures_util::{stream, FutureExt, StreamExt};
use pin_project::pin_project;
use serde::Deserialize;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
    vec::IntoIter,
};

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
    R: Send + Sync + for<'de> Deserialize<'de>,
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
    R: Send + Sync + for<'de> Deserialize<'de> + 'a,
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
