use crate::{JsonRpcClient, Middleware, Provider};

use ethers_core::types::U256;

use futures_util::stream::Stream;
use pin_project::{pin_project, pinned_drop};
use serde::de::DeserializeOwned;
use serde_json::value::RawValue;
use std::{
    collections::VecDeque,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use tracing::error;

/// A transport implementation supporting pub sub subscriptions.
pub trait PubsubClient: JsonRpcClient {
    /// The type of stream this transport returns
    type NotificationStream: futures_core::Stream<Item = Box<RawValue>> + Send + Unpin;

    /// Add a subscription to this transport
    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, Self::Error>;

    /// Remove a subscription from this transport
    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), Self::Error>;
}

#[must_use = "subscriptions do nothing unless you stream them"]
#[pin_project(PinnedDrop)]
/// Streams data from an installed filter via `eth_subscribe`
pub struct SubscriptionStream<'a, P: PubsubClient, R: DeserializeOwned> {
    /// The subscription's installed id on the ethereum node
    pub id: U256,

    loaded_elements: VecDeque<R>,

    pub(crate) provider: &'a Provider<P>,

    #[pin]
    rx: P::NotificationStream,

    ret: PhantomData<R>,
}

impl<'a, P, R> SubscriptionStream<'a, P, R>
where
    P: PubsubClient,
    R: DeserializeOwned,
{
    /// Creates a new subscription stream for the provided subscription id.
    ///
    /// ### Note
    /// Most providers treat `SubscriptionStream` IDs as global singletons.
    /// Instantiating this directly with a known ID will likely cause any
    /// existing streams with that ID to end. To avoid this, start a new stream
    /// using [`Provider::subscribe`] instead of `SubscriptionStream::new`.
    pub fn new(id: U256, provider: &'a Provider<P>) -> Result<Self, P::Error> {
        // Call the underlying PubsubClient's subscribe
        let rx = provider.as_ref().subscribe(id)?;
        Ok(Self { id, provider, rx, ret: PhantomData, loaded_elements: VecDeque::new() })
    }

    /// Unsubscribes from the subscription.
    pub async fn unsubscribe(&self) -> Result<bool, crate::ProviderError> {
        self.provider.unsubscribe(self.id).await
    }

    /// Set the loaded elements buffer. This buffer contains logs waiting for
    /// the consumer to read. Setting the buffer can be used to add logs
    /// without receiving them from the RPC node
    ///
    /// ### Warning
    ///
    /// Setting the buffer will drop any logs in the current buffer.
    pub fn set_loaded_elements(&mut self, loaded_elements: VecDeque<R>) {
        self.loaded_elements = loaded_elements;
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
        if !self.loaded_elements.is_empty() {
            let next_element = self.get_mut().loaded_elements.pop_front();
            return Poll::Ready(next_element)
        }

        let mut this = self.project();
        loop {
            return match futures_util::ready!(this.rx.as_mut().poll_next(ctx)) {
                Some(item) => match serde_json::from_str(item.get()) {
                    Ok(res) => Poll::Ready(Some(res)),
                    Err(err) => {
                        error!("failed to deserialize item {:?}", err);
                        continue
                    }
                },
                None => Poll::Ready(None),
            }
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
