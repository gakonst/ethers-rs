use crate::ProviderError;

use ethers_core::types::U256;

use futures_core::{stream::Stream, TryFuture};
use futures_timer::Delay;
use futures_util::{stream, FutureExt, StreamExt};
use pin_project::pin_project;
use serde::Deserialize;
use std::{
    future::Future,
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

/// Trait for streaming filters.
pub trait FilterStream<R>: StreamExt + Stream<Item = R>
where
    R: for<'de> Deserialize<'de>,
{
    /// Returns the filter's ID for it to be uninstalled
    fn id(&self) -> U256;

    /// Sets the stream's polling interval
    fn interval(self, duration: Duration) -> Self;

    /// Alias for Box::pin, must be called in order to pin the stream and be able
    /// to call `next` on it.
    fn stream(self) -> Pin<Box<Self>>
    where
        Self: Sized,
    {
        Box::pin(self)
    }
}

enum FilterWatcherState<F, R> {
    WaitForInterval,
    GetFilterChanges(F),
    NextItem(IntoIter<R>),
}

#[must_use = "filters do nothing unless you stream them"]
#[pin_project]
pub(crate) struct FilterWatcher<F: FutureFactory, R> {
    id: U256,

    #[pin]
    // Future factory for generating new calls on each loop
    factory: F,

    // The polling interval
    interval: Box<dyn Stream<Item = ()> + Send + Unpin>,

    state: FilterWatcherState<F::FutureItem, R>,
}

impl<F, R> FilterWatcher<F, R>
where
    F: FutureFactory,
    R: for<'de> Deserialize<'de>,
{
    /// Creates a new watcher with the provided factory and filter id.
    pub fn new<T: Into<U256>>(id: T, factory: F) -> Self {
        Self {
            id: id.into(),
            interval: Box::new(interval(DEFAULT_POLL_INTERVAL)),
            state: FilterWatcherState::WaitForInterval,
            factory,
        }
    }
}

impl<F, R> FilterStream<R> for FilterWatcher<F, R>
where
    F: FutureFactory,
    F::FutureItem: Future<Output = Result<Vec<R>, ProviderError>>,
    R: for<'de> Deserialize<'de>,
{
    fn id(&self) -> U256 {
        self.id
    }

    fn interval(mut self, duration: Duration) -> Self {
        self.interval = Box::new(interval(duration));
        self
    }
}

// Pattern for flattening the returned Vec of filter changes taken from
// https://github.com/tomusdrw/rust-web3/blob/f043b222744580bf4be043da757ab0b300c3b2da/src/api/eth_filter.rs#L50-L67
impl<F, R> Stream for FilterWatcher<F, R>
where
    F: FutureFactory,
    F::FutureItem: Future<Output = Result<Vec<R>, ProviderError>>,
    R: for<'de> Deserialize<'de>,
{
    type Item = R;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        *this.state = match this.state {
            FilterWatcherState::WaitForInterval => {
                // Wait the polling period
                let _ready = futures_util::ready!(this.interval.poll_next_unpin(cx));

                // create a new instance of the future
                cx.waker().wake_by_ref();
                FilterWatcherState::GetFilterChanges(this.factory.as_mut().new())
            }
            FilterWatcherState::GetFilterChanges(fut) => {
                // NOTE: If the provider returns an error, this will return an empty
                // vector. Should we make this return a Result instead? Ideally if we're
                // in a streamed loop we wouldn't want the loop to terminate if an error
                // is encountered (since it might be a temporary error).
                let items: Vec<R> = futures_util::ready!(fut.poll_unpin(cx)).unwrap_or_default();
                FilterWatcherState::NextItem(items.into_iter())
            }
            // Consume 1 element from the vector. If more elements are in the vector,
            // the next call will immediately go to this branch instead of trying to get
            // filter changes again. Once the whole vector is consumed, it will poll again
            // for new logs
            FilterWatcherState::NextItem(iter) => match iter.next() {
                Some(item) => return Poll::Ready(Some(item)),
                None => {
                    cx.waker().wake_by_ref();
                    FilterWatcherState::WaitForInterval
                }
            },
        };

        Poll::Pending
    }
}

// Do not leak private trait
// Pattern for re-usable futures from: https://gitlab.com/Ploppz/futures-retry/-/blob/std-futures/src/future.rs#L13
use factory::FutureFactory;
mod factory {
    use super::*;

    /// A factory trait used to create futures.
    ///
    /// We need a factory for the stream logic because when (and if) a future
    /// is polled to completion, it can't be polled again. Hence we need to
    /// create a new one.
    ///
    /// This trait is implemented for any closure that returns a `Future`, so you don't
    /// have to write your own type and implement it to handle some simple cases.
    pub trait FutureFactory {
        /// A future type that is created by the `new` method.
        type FutureItem: TryFuture + Unpin;

        /// Creates a new future. We don't need the factory to be immutable so we
        /// pass `self` as a mutable reference.
        fn new(self: Pin<&mut Self>) -> Self::FutureItem;
    }

    impl<T, F> FutureFactory for T
    where
        T: Unpin + FnMut() -> F,
        F: TryFuture + Unpin,
    {
        type FutureItem = F;

        #[allow(clippy::new_ret_no_self)]
        fn new(self: Pin<&mut Self>) -> F {
            (*self.get_mut())()
        }
    }
}

#[cfg(test)]
mod watch {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn stream() {
        let factory = || Box::pin(async { Ok::<Vec<u64>, ProviderError>(vec![1, 2, 3]) });
        let filter = FilterWatcher::<_, u64>::new(1, factory);
        // stream combinator calls are still doable since FilterStream extends
        // Stream and StreamExt
        let mut stream = filter
            .interval(Duration::from_millis(100u64))
            .stream()
            .map(|x| 2 * x);
        assert_eq!(stream.next().await.unwrap(), 2);
        assert_eq!(stream.next().await.unwrap(), 4);
        assert_eq!(stream.next().await.unwrap(), 6);
        // this will poll the factory function again since it consumed the entire
        // vector, so it'll wrap around. Realistically, we'd then sleep for a few seconds
        // until new blocks are mined, until the call to the factory returns a non-empty
        // vector of logs
        assert_eq!(stream.next().await.unwrap(), 2);
    }
}
