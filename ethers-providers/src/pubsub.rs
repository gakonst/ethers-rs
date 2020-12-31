use crate::{JsonRpcClient, Middleware, Provider};

use ethers_core::types::U256;

use futures_util::stream::Stream;
use pin_project::{pin_project, pinned_drop};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

/// A transport implementation supporting pub sub subscriptions.
pub trait PubsubClient: JsonRpcClient {
    /// The type of stream this transport returns
    type NotificationStream: futures_core::Stream<Item = Value>;

    /// Add a subscription to this transport
    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, Self::Error>;

    /// Remove a subscription from this transport
    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), Self::Error>;
}

#[must_use = "subscriptions do nothing unless you stream them"]
#[pin_project(PinnedDrop)]
pub struct SubscriptionStream<'a, P: PubsubClient, R: DeserializeOwned> {
    /// The subscription's installed id on the ethereum node
    pub id: U256,

    provider: &'a Provider<P>,

    #[pin]
    rx: P::NotificationStream,

    ret: PhantomData<R>,
}

impl<'a, P, R> SubscriptionStream<'a, P, R>
where
    P: PubsubClient,
    R: DeserializeOwned,
{
    /// Creates a new subscription stream for the provided subscription id
    pub fn new(id: U256, provider: &'a Provider<P>) -> Result<Self, P::Error> {
        // Call the underlying PubsubClient's subscribe
        let rx = provider.as_ref().subscribe(id)?;
        Ok(Self {
            id,
            provider,
            rx,
            ret: PhantomData,
        })
    }

    /// Unsubscribes from the subscription
    pub async fn unsubscribe(&self) -> Result<bool, crate::ProviderError> {
        self.provider.unsubscribe(self.id).await
    }
}

// Each subscription item is a serde_json::Value which must be decoded to the
// subscription's return type.
// TODO: Can this be replaced with an `rx.map` in the constructor?
impl<'a, P, R> Stream for SubscriptionStream<'a, P, R>
where
    P: PubsubClient,
    R: DeserializeOwned,
{
    type Item = R;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match futures_util::ready!(this.rx.poll_next(ctx)) {
            Some(item) => match serde_json::from_value(item) {
                Ok(res) => Poll::Ready(Some(res)),
                _ => Poll::Pending,
            },
            None => Poll::Pending,
        }
    }
}

#[pinned_drop]
impl<P, R> PinnedDrop for SubscriptionStream<'_, P, R>
where
    P: PubsubClient,
    R: DeserializeOwned,
{
    fn drop(self: Pin<&mut Self>) {
        // on drop it removes the handler from the websocket so that it stops
        // getting populated. We need to call `unsubscribe` explicitly to cancel
        // the subscription
        let _ = (*self.provider).as_ref().unsubscribe(self.id);
    }
}
