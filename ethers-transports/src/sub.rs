use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use ethers_core::types::U256;
use serde::Deserialize;
use serde_json::value::RawValue;
use tokio::sync::mpsc;
use tokio_stream::Stream;

use crate::{err::TransportError, BidiTransport, Provider, ProviderError, ResponsePayload};

pub struct SubscriptionStream<T, I> {
    provider: Provider<T>,
    rx: mpsc::UnboundedReceiver<ResponsePayload>,
    id: U256,
    _marker: PhantomData<fn() -> I>,
}

impl<T, I> SubscriptionStream<T, I>
where
    T: BidiTransport + Clone,
    I: for<'de> Deserialize<'de>,
{
    pub async fn recv(&mut self) -> Option<Result<I, Box<TransportError>>> {
        let raw = match self.rx.recv().await? {
            Ok(raw) => raw,
            Err(err) => return Some(Err(err)),
        };

        match serde_json::from_str(raw.get()) {
            Ok(item) => Some(Ok(item)),
            Err(source) => Some(Err(TransportError::json(raw.get(), source))),
        }
    }

    pub async fn unsubscribe(self) -> Result<(), Box<ProviderError>> {
        todo!()
    }

    fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<I, Box<TransportError>>>> {
        match self.rx.poll_recv(cx) {
            Poll::Ready(Some(next)) => Poll::Ready(Some(self.parse_next(next))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn parse_next(
        &self,
        next: Result<Box<RawValue>, Box<TransportError>>,
    ) -> Result<I, Box<TransportError>> {
        let raw = next?;
        match serde_json::from_str(raw.get()) {
            Ok(item) => Ok(item),
            Err(source) => Err(TransportError::json(raw.get(), source)),
        }
    }
}

impl<T, I> Stream for SubscriptionStream<T, I>
where
    T: BidiTransport + Clone + Unpin,
    I: for<'de> Deserialize<'de>,
{
    type Item = Result<I, Box<TransportError>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().poll_recv(cx)
    }
}
