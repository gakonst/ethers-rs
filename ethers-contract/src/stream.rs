//! Contains the `EventStream` type which aids in streaming access to contract
//! events

use crate::LogMeta;
use ethers_core::types::{Log, U256};
use futures_util::{
    future::Either,
    stream::{Stream, StreamExt},
};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

type MapEvent<'a, R, E> = Box<dyn Fn(Log) -> Result<R, E> + 'a + Send + Sync>;

#[pin_project]
/// Generic wrapper around Log streams, mapping their content to a specific
/// deserialized log struct.
///
/// We use this wrapper type instead of `StreamExt::map` in order to preserve
/// information about the filter/subscription's id.
pub struct EventStream<'a, T, R, E> {
    /// The stream ID, provided by the RPC server
    pub id: U256,
    #[pin]
    stream: T,
    parse: MapEvent<'a, R, E>,
}

impl<'a, T, R, E> EventStream<'a, T, R, E> {
    /// Turns this stream of events into a stream that also yields the event's metadata
    pub fn with_meta(self) -> EventStreamMeta<'a, T, R, E> {
        EventStreamMeta(self)
    }
}

impl<'a, T, R, E> EventStream<'a, T, R, E> {
    /// Instantiate a new `EventStream`
    ///
    /// Typically users should not call this directly
    pub fn new(id: U256, stream: T, parse: MapEvent<'a, R, E>) -> Self {
        Self { id, stream, parse }
    }
}

impl<'a, T, R, E> Stream for EventStream<'a, T, R, E>
where
    T: Stream<Item = Log> + Unpin,
{
    type Item = Result<R, E>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        match futures_util::ready!(this.stream.poll_next_unpin(ctx)) {
            Some(item) => Poll::Ready(Some((this.parse)(item))),
            None => Poll::Pending,
        }
    }
}

impl<'a, T, R, E> EventStream<'a, T, R, E>
where
    T: Stream<Item = Log> + Unpin + 'a,
    R: 'a,
    E: 'a,
{
    /// This function will attempt to pull events from both event streams. Each
    /// stream will be polled in a round-robin fashion, and whenever a stream is
    /// ready to yield an event that event is yielded.
    ///
    /// After one of the two event streams completes, the remaining one will be
    /// polled exclusively. The returned stream completes when both input
    /// streams have completed.
    ///
    ///
    /// Note that this function consumes both streams and returns a wrapped
    /// version of them.
    /// The item of the wrapped stream is an `Either`, and the items that the `self` streams yields
    /// will be stored in the left-hand variant of that `Either` and the other stream's (`st`) items
    /// will be wrapped into the right-hand variant of that `Either`.
    ///
    /// # Example
    // Ignore because `ethers-contract-derive` macros do not work in doctests in `ethers-contract`.
    /// ```ignore
    /// # #[cfg(feature = "abigen")]
    /// # async fn test<M:ethers_providers::Middleware>(contract: ethers_contract::Contract<M>) {
    /// # use ethers_core::types::*;
    /// # use futures_util::stream::StreamExt;
    /// # use futures_util::future::Either;
    /// # use ethers_contract::{Contract, ContractFactory, EthEvent};
    ///
    /// #[derive(Clone, Debug, EthEvent)]
    /// pub struct Approval {
    ///     #[ethevent(indexed)]
    ///     pub token_owner: Address,
    ///     #[ethevent(indexed)]
    ///     pub spender: Address,
    ///     pub tokens: U256,
    /// }
    ///
    /// #[derive(Clone, Debug, EthEvent)]
    /// pub struct Transfer {
    ///     #[ethevent(indexed)]
    ///     pub from: Address,
    ///     #[ethevent(indexed)]
    ///     pub to: Address,
    ///     pub tokens: U256,
    /// }
    ///
    ///
    /// let ev1 = contract.event::<Approval>().from_block(1337).to_block(2000);
    /// let ev2 = contract.event::<Transfer>();
    ///
    /// let mut events = ev1.stream().await.unwrap().select(ev2.stream().await.unwrap()).ok();
    ///
    /// while let Some(either) = events.next().await {
    ///     match either {
    ///         Either::Left(approval) => { let Approval{token_owner,spender,tokens} = approval; }
    ///         Either::Right(transfer) => { let Transfer{from,to,tokens} = transfer; }
    ///     }
    /// }
    ///
    /// # }
    /// ```
    pub fn select<St>(self, st: St) -> SelectEvent<SelectEither<'a, Result<R, E>, St::Item>>
    where
        St: Stream + Unpin + 'a,
    {
        SelectEvent(Box::pin(futures_util::stream::select(
            self.map(Either::Left),
            st.map(Either::Right),
        )))
    }
}

/// A stream of two items
pub type SelectEither<'a, L, R> = Pin<Box<dyn Stream<Item = Either<L, R>> + 'a>>;

/// Stream for [`EventStream::select`]
#[pin_project]
pub struct SelectEvent<T>(#[pin] T);

impl<'a, T, L, LE, R, RE> SelectEvent<T>
where
    T: Stream<Item = Either<Result<L, LE>, Result<R, RE>>> + 'a,
    L: 'a,
    LE: 'a,
    R: 'a,
    RE: 'a,
{
    /// Turns a stream of Results to a stream of `Result::ok` for both arms
    pub fn ok(self) -> Pin<Box<dyn Stream<Item = Either<L, R>> + 'a>> {
        Box::pin(self.filter_map(|e| async move {
            match e {
                Either::Left(res) => res.ok().map(Either::Left),
                Either::Right(res) => res.ok().map(Either::Right),
            }
        }))
    }
}

impl<T: Stream> Stream for SelectEvent<T> {
    type Item = T::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.0.poll_next(cx)
    }
}

/// Wrapper around a `EventStream`, that in addition to the deserialized Event type also yields the
/// `LogMeta`.
#[pin_project]
pub struct EventStreamMeta<'a, T, R, E>(pub EventStream<'a, T, R, E>);

impl<'a, T, R, E> EventStreamMeta<'a, T, R, E>
where
    T: Stream<Item = Log> + Unpin + 'a,
    R: 'a,
    E: 'a,
{
    /// See `EventStream::select`
    #[allow(clippy::type_complexity)]
    pub fn select<St>(
        self,
        st: St,
    ) -> SelectEvent<SelectEither<'a, Result<(R, LogMeta), E>, St::Item>>
    where
        St: Stream + Unpin + 'a,
    {
        SelectEvent(Box::pin(futures_util::stream::select(
            self.map(Either::Left),
            st.map(Either::Right),
        )))
    }
}

impl<'a, T, R, E> Stream for EventStreamMeta<'a, T, R, E>
where
    T: Stream<Item = Log> + Unpin,
{
    type Item = Result<(R, LogMeta), E>;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match futures_util::ready!(this.0.stream.poll_next_unpin(ctx)) {
            Some(item) => {
                let meta = LogMeta::from(&item);
                let res = (this.0.parse)(item);
                let res = res.map(|inner| (inner, meta));
                Poll::Ready(Some(res))
            }
            None => Poll::Ready(None),
        }
    }
}
