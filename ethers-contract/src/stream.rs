use crate::LogMeta;
use ethers_core::types::{Log, U256};
use futures_util::stream::{Stream, StreamExt};
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

type MapEvent<'a, R, E> = Box<dyn Fn(Log) -> Result<R, E> + 'a + Send + Sync>;

#[pin_project]
/// Generic wrapper around Log streams, mapping their content to a specific
/// deserialized log struct.
///
/// We use this wrapper type instead of `StreamExt::map` in order to preserve
/// information about the filter/subscription's id.
pub struct EventStream<'a, T, R, E> {
    pub id: U256,
    #[pin]
    stream: T,
    parse: MapEvent<'a, R, E>,
}

impl<'a, T, R, E> EventStream<'a, T, R, E> {
    pub fn with_meta(self) -> EventStreamMeta<'a, T, R, E> {
        EventStreamMeta(self)
    }
}

impl<'a, T, R, E> EventStream<'a, T, R, E> {
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

#[pin_project]
pub struct EventStreamMeta<'a, T, R, E>(pub EventStream<'a, T, R, E>);

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
            None => Poll::Pending,
        }
    }
}
