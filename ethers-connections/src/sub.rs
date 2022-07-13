use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use serde::Deserialize;
use serde_json::value::RawValue;
use tokio_stream::Stream;

use ethers_core::types::U256;

use crate::{
    connection::ConnectionError, DuplexConnection, NotificationReceiver, Provider, ProviderError,
};
/// A stream that receives notifications for a registered subscription and
/// parses them into an expected type.
pub struct SubscriptionStream<T, C: DuplexConnection> {
    /// The ID of the of the subscription (`None` if no longer subscribed).
    id: Option<U256>,
    /// The `Provider` instance (owned) required to unsubscribe.
    provider: Provider<C>,
    /// The receiver for all notifications send for the ID.
    rx: NotificationReceiver,
    /// The marker indicating the type produced by this stream.
    _marker: PhantomData<fn() -> T>,
}

impl<T, C: DuplexConnection> SubscriptionStream<T, C> {
    /// Consumes the [`SubscriptionStream`] and returns it's internal
    /// components or `None`, if the stream has previously been unsubscribed.
    pub fn into_raw(self) -> Option<(U256, NotificationReceiver)> {
        self.id.map(|id| (id, self.rx))
    }
}

impl<T, C> SubscriptionStream<T, C>
where
    T: for<'de> Deserialize<'de>,
    C: DuplexConnection,
{
    /// Returns the stream's subscription ID or `None`, if it has previously
    /// been unsubscribed.
    pub fn id(&self) -> Option<&U256> {
        self.id.as_ref()
    }

    /// Receives the next notification from the stream.
    pub async fn recv(&mut self) -> Option<Result<T, ConnectionError>> {
        let raw = self.rx.recv().await?;
        match serde_json::from_str(raw.get()) {
            Ok(item) => Some(Ok(item)),
            Err(source) => Some(Err(ConnectionError::json(raw.get(), source))),
        }
    }

    /// Polls & parses the next notification.
    fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<T, ConnectionError>>> {
        match self.rx.poll_recv(cx) {
            Poll::Ready(Some(raw)) => Poll::Ready(Some(self.parse_next(raw))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    /// Parses the given `raw` notification.
    fn parse_next(&self, raw: Box<RawValue>) -> Result<T, ConnectionError> {
        match serde_json::from_str(raw.get()) {
            Ok(item) => Ok(item),
            Err(source) => Err(ConnectionError::json(raw.get(), source)),
        }
    }
}

impl<T, C> SubscriptionStream<T, C>
where
    C: DuplexConnection,
{
    /// Creates a new [`SubscriptionStream`].
    pub(crate) fn new(id: U256, provider: Provider<C>, rx: NotificationReceiver) -> Self {
        Self { id: Some(id), provider, rx, _marker: PhantomData }
    }

    /// Unsubscribes from the stream's subscription.
    pub async fn unsubscribe(&mut self) -> Result<(), Box<ProviderError>> {
        match self.id.take() {
            Some(id) => {
                let _ = self.provider.unsubscribe(id).await?;
                self.rx.close();
                Ok(())
            }
            None => Ok(()),
        }
    }
}

impl<T, C> Stream for SubscriptionStream<T, C>
where
    T: for<'de> Deserialize<'de>,
    C: DuplexConnection + Clone + Unpin,
{
    type Item = Result<T, ConnectionError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().poll_recv(cx)
    }
}
