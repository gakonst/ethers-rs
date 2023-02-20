use crate::{
    utils::{interval, PinBoxFut},
    JsonRpcClient, Middleware, Provider,
};
use ethers_core::types::U256;
use futures_core::stream::Stream;
use futures_util::StreamExt;
use pin_project::pin_project;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
    vec::IntoIter,
};

/// The default polling interval for filters and pending transactions
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(7000);

/// The polling interval to use for local endpoints, See [`crate::is_local_endpoint()`]
pub const DEFAULT_LOCAL_POLL_INTERVAL: Duration = Duration::from_millis(100);

enum FilterWatcherState<'a, R> {
    WaitForInterval,
    GetFilterChanges(PinBoxFut<'a, Vec<R>>),
    NextItem(IntoIter<R>),
}

#[must_use = "filters do nothing unless you stream them"]
/// Streams data from an installed filter via `eth_getFilterChanges`
#[pin_project]
pub struct FilterWatcher<'a, P, R> {
    /// The filter's installed id on the ethereum node
    pub id: U256,

    pub(crate) provider: &'a Provider<P>,

    // The polling interval
    interval: Box<dyn Stream<Item = ()> + Send + Unpin>,
    /// statemachine driven by the Stream impl
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

// Advances the filter's state machine
impl<'a, P, R> Stream for FilterWatcher<'a, P, R>
where
    P: JsonRpcClient,
    R: Serialize + Send + Sync + DeserializeOwned + Debug + 'a,
{
    type Item = R;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let id = *this.id;

        loop {
            *this.state = match &mut this.state {
                FilterWatcherState::WaitForInterval => {
                    // Wait the polling period
                    let _ready = futures_util::ready!(this.interval.poll_next_unpin(cx));
                    let fut = Box::pin(this.provider.get_filter_changes(id));
                    FilterWatcherState::GetFilterChanges(fut)
                }
                FilterWatcherState::GetFilterChanges(fut) => {
                    // NOTE: If the provider returns an error, this will return an empty
                    // vector. Should we make this return a Result instead? Ideally if we're
                    // in a streamed loop we wouldn't want the loop to terminate if an error
                    // is encountered (since it might be a temporary error).
                    let items: Vec<R> =
                        futures_util::ready!(fut.as_mut().poll(cx)).unwrap_or_default();
                    FilterWatcherState::NextItem(items.into_iter())
                }
                // Consume 1 element from the vector. If more elements are in the vector,
                // the next call will immediately go to this branch instead of trying to get
                // filter changes again. Once the whole vector is consumed, it will poll again
                // for new logs
                FilterWatcherState::NextItem(iter) => {
                    if let item @ Some(_) = iter.next() {
                        return Poll::Ready(item)
                    }
                    FilterWatcherState::WaitForInterval
                }
            };
        }
    }
}
