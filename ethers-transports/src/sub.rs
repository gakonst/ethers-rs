use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use serde::Deserialize;
use serde_json::value::RawValue;
use tokio_stream::Stream;

use ethers_core::types::U256;

use crate::{err::TransportError, DuplexConnection, NotificationReceiver, Provider, ProviderError};

pub struct SubscriptionStream<T, C: DuplexConnection> {
    /// The ID of the of the subscription (`None` if no longer subscribed).
    id: Option<U256>,
    /// The `Provider` instance (owned) required to unsubscribe.
    provider: Provider<C>,
    /// The receiver for all notifications send for the ID.
    rx: NotificationReceiver,
    _marker: PhantomData<fn() -> T>,
}

impl<T, C> SubscriptionStream<T, C>
where
    T: for<'de> Deserialize<'de>,
    C: DuplexConnection,
{
    pub fn id(&self) -> Option<&U256> {
        self.id.as_ref()
    }

    pub async fn recv(&mut self) -> Option<Result<T, Box<TransportError>>> {
        let raw = self.rx.recv().await?;
        match serde_json::from_str(raw.get()) {
            Ok(item) => Some(Ok(item)),
            Err(source) => Some(Err(TransportError::json(raw.get(), source))),
        }
    }

    fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<T, Box<TransportError>>>> {
        match self.rx.poll_recv(cx) {
            Poll::Ready(Some(raw)) => Poll::Ready(Some(self.parse_next(raw))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn parse_next(&self, raw: Box<RawValue>) -> Result<T, Box<TransportError>> {
        match serde_json::from_str(raw.get()) {
            Ok(item) => Ok(item),
            Err(source) => Err(TransportError::json(raw.get(), source)),
        }
    }
}

impl<T, C> SubscriptionStream<T, C>
where
    C: DuplexConnection,
{
    pub(crate) fn new(id: U256, provider: Provider<C>, rx: NotificationReceiver) -> Self {
        Self { id: Some(id), provider, rx, _marker: PhantomData }
    }

    pub async fn unsubscribe(&mut self) -> Result<bool, Box<ProviderError>> {
        match self.id.take() {
            Some(id) => self.provider.unsubscribe(id).await,
            None => Ok(false),
        }
    }
}

impl<T, C> Stream for SubscriptionStream<T, C>
where
    T: for<'de> Deserialize<'de>,
    C: DuplexConnection + Clone + Unpin,
{
    type Item = Result<T, Box<TransportError>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().poll_recv(cx)
    }
}
