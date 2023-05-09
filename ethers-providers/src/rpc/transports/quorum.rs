use crate::{errors::ProviderError, JsonRpcClient, PubsubClient};
use async_trait::async_trait;
use ethers_core::types::{U256, U64};
use futures_core::Stream;
use futures_util::{future::join_all, FutureExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{value::RawValue, Value};
use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use thiserror::Error;

/// A provider that bundles multiple providers and only returns a value to the
/// caller once the quorum has been reached.
///
/// # Example
///
/// Create a `QuorumProvider` that uses a homogenous `Provider` type only returns a value if the
/// `Quorum::Majority` of the weighted providers return the same value.
///
/// ```
/// use ethers_core::types::U64;
/// use ethers_providers::{JsonRpcClient, QuorumProvider, Quorum, WeightedProvider, Http};
/// use std::str::FromStr;
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let provider1 = WeightedProvider::new(Http::from_str("http://localhost:8545")?);
/// let provider2 = WeightedProvider::with_weight(Http::from_str("http://localhost:8545")?, 2);
/// let provider3 = WeightedProvider::new(Http::from_str("http://localhost:8545")?);
/// let provider = QuorumProvider::builder()
///     .add_providers([provider1, provider2, provider3])
///     .quorum(Quorum::Majority)
///     .build();
/// // the weight at which a quorum is reached,
/// assert_eq!(provider.quorum_weight(), 4 / 2); // majority >=50%
/// let block_number: U64 = provider.request("eth_blockNumber", ()).await?;
///
/// # Ok(())
/// # }
/// ```
///
/// # Example
///
/// Create a `QuorumProvider` consisting of different `Provider` types
///
/// ```
/// use ethers_core::types::U64;
/// use ethers_providers::{JsonRpcClient, QuorumProvider, Quorum, WeightedProvider, Http, Ws};
/// use std::str::FromStr;
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
///     let provider: QuorumProvider = QuorumProvider::dyn_rpc()
///         .add_provider(WeightedProvider::new(Box::new(Http::from_str("http://localhost:8545")?)))
///         .add_provider(WeightedProvider::with_weight(
///             Box::new(Ws::connect("ws://localhost:8545").await?),
///             2,
///         ))
///         .add_provider(WeightedProvider::with_weight(
///             Box::new(Ws::connect("ws://localhost:8545").await?),
///             2,
///         ))
///         // the quorum provider will yield the response if >50% of the weighted inner provider
///         // returned the same value
///         .quorum(Quorum::Majority)
///         .build();
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct QuorumProvider<T = Box<dyn JsonRpcClientWrapper>> {
    /// What kind of quorum is required
    quorum: Quorum,
    /// The weight at which quorum is reached
    quorum_weight: u64,
    /// All the internal providers this providers runs
    providers: Vec<WeightedProvider<T>>,
}

impl QuorumProvider<Box<dyn JsonRpcClientWrapper>> {
    /// Create a `QuorumProvider` for different `JsonRpcClient` types
    pub fn dyn_rpc() -> QuorumProviderBuilder<Box<dyn JsonRpcClientWrapper>> {
        Self::builder()
    }
}

impl QuorumProvider<Box<dyn PubsubClientWrapper>> {
    /// Create a `QuorumProvider` for different `PubsubClient` types
    pub fn dyn_pub_sub() -> QuorumProviderBuilder<Box<dyn PubsubClientWrapper>> {
        Self::builder()
    }
}

impl<T> QuorumProvider<T> {
    /// Convenience method for creating a `QuorumProviderBuilder` with same `JsonRpcClient` types
    pub fn builder() -> QuorumProviderBuilder<T> {
        QuorumProviderBuilder::default()
    }

    /// Instantiate a new `QuorumProvider` from a [`Quorum`] and a set of
    /// providers
    pub fn new(quorum: Quorum, providers: impl IntoIterator<Item = WeightedProvider<T>>) -> Self {
        Self::builder().add_providers(providers).quorum(quorum).build()
    }

    /// Return a reference to the weighted providers
    pub fn providers(&self) -> &[WeightedProvider<T>] {
        &self.providers
    }

    /// The weight at which the provider reached a quorum
    pub fn quorum_weight(&self) -> u64 {
        self.quorum_weight
    }

    /// Add a provider to the set
    pub fn add_provider(&mut self, provider: WeightedProvider<T>) {
        self.providers.push(provider);
        self.quorum_weight = self.quorum.weight(&self.providers)
    }
}

#[derive(Debug, Clone)]
pub struct QuorumProviderBuilder<T> {
    quorum: Quorum,
    providers: Vec<WeightedProvider<T>>,
}

impl<T> Default for QuorumProviderBuilder<T> {
    fn default() -> Self {
        Self { quorum: Default::default(), providers: Vec::new() }
    }
}

impl<T> QuorumProviderBuilder<T> {
    pub fn add_provider(mut self, provider: WeightedProvider<T>) -> Self {
        self.providers.push(provider);
        self
    }
    pub fn add_providers(
        mut self,
        providers: impl IntoIterator<Item = WeightedProvider<T>>,
    ) -> Self {
        for provider in providers {
            self.providers.push(provider);
        }
        self
    }

    /// Set the kind of quorum
    pub fn quorum(mut self, quorum: Quorum) -> Self {
        self.quorum = quorum;
        self
    }

    pub fn build(self) -> QuorumProvider<T> {
        let quorum_weight = self.quorum.weight(&self.providers);
        QuorumProvider { quorum: self.quorum, quorum_weight, providers: self.providers }
    }
}

impl<T: JsonRpcClientWrapper> QuorumProvider<T> {
    /// Returns the block height that _all_ providers have surpassed.
    ///
    /// This is the minimum of all provider's block numbers
    async fn get_minimum_block_number(&self) -> Result<U64, ProviderError> {
        let mut numbers = join_all(self.providers.iter().map(|provider| async move {
            let block = provider.inner.request("eth_blockNumber", QuorumParams::Zst).await?;
            serde_json::from_value::<U64>(block).map_err(ProviderError::from)
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
        numbers.sort();

        numbers
            .into_iter()
            .next()
            .ok_or_else(|| ProviderError::CustomError("No Providers".to_string()))
    }

    /// Normalizes the request payload depending on the call
    async fn normalize_request(&self, method: &str, q_params: &mut QuorumParams) {
        let params = if let QuorumParams::Value(v) = q_params {
            v
        } else {
            // at this time no normalization is required for calls with zero parameters.
            return
        };
        match method {
            "eth_call" |
            "eth_createAccessList" |
            "eth_getStorageAt" |
            "eth_getCode" |
            "eth_getProof" |
            "trace_call" |
            "trace_block" => {
                // calls that include the block number in the params at the last index of json array
                if let Some(block) = params.as_array_mut().and_then(|arr| arr.last_mut()) {
                    if Some("latest") == block.as_str() {
                        // replace `latest` with the minimum block height of all providers
                        if let Ok(minimum) = self
                            .get_minimum_block_number()
                            .await
                            .and_then(|num| Ok(serde_json::to_value(num)?))
                        {
                            *block = minimum
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Determines when the provider reached a quorum
#[derive(Debug, Default, Copy, Clone)]
pub enum Quorum {
    ///  The quorum is reached when all providers return the exact value
    All,
    /// The quorum is reached when the majority of the providers have returned a
    /// matching value, taking into account their weight.
    #[default]
    Majority,
    /// The quorum is reached when the cumulative weight of a matching return
    /// exceeds the given percentage of the total weight.
    ///
    /// NOTE: this must be less than `100u8`
    Percentage(u8),
    /// The quorum is reached when the given number of provider agree
    /// The configured weight is ignored in this case.
    ProviderCount(usize),
    /// The quorum is reached once the accumulated weight of the matching return
    /// exceeds this weight.
    Weight(u64),
}

impl Quorum {
    fn weight<T>(self, providers: &[WeightedProvider<T>]) -> u64 {
        match self {
            Quorum::All => providers.iter().map(|p| p.weight).sum::<u64>(),
            Quorum::Majority => {
                let total = providers.iter().map(|p| p.weight).sum::<u64>();
                let rem = total % 2;
                total / 2 + rem
            }
            Quorum::Percentage(p) => {
                providers.iter().map(|p| p.weight).sum::<u64>() * (p as u64) / 100
            }
            Quorum::ProviderCount(num) => {
                // take the lowest `num` weights
                let mut weights = providers.iter().map(|p| p.weight).collect::<Vec<_>>();
                weights.sort_unstable();
                weights.into_iter().take(num).sum()
            }
            Quorum::Weight(w) => w,
        }
    }
}

// A future that returns the provider's response and it's index within the
// `QuorumProvider` provider set
#[cfg(target_arch = "wasm32")]
type PendingRequest<'a> = Pin<Box<dyn Future<Output = (Result<Value, ProviderError>, usize)> + 'a>>;
#[cfg(not(target_arch = "wasm32"))]
type PendingRequest<'a> =
    Pin<Box<dyn Future<Output = (Result<Value, ProviderError>, usize)> + 'a + Send>>;

/// A future that only returns a value of the `QuorumProvider`'s provider
/// reached a quorum.
struct QuorumRequest<'a, T> {
    inner: &'a QuorumProvider<T>,
    /// The different answers with their cumulative weight
    responses: Vec<(Value, u64)>,
    /// All the errors the provider yielded
    errors: Vec<ProviderError>,
    // Requests currently pending
    requests: Vec<PendingRequest<'a>>,
}

impl<'a, T> QuorumRequest<'a, T> {
    fn new(inner: &'a QuorumProvider<T>, requests: Vec<PendingRequest<'a>>) -> Self {
        Self { responses: Vec::new(), errors: Vec::new(), inner, requests }
    }
}

impl<'a, T> Future for QuorumRequest<'a, T> {
    type Output = Result<Value, QuorumError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        for n in (0..this.requests.len()).rev() {
            let mut request = this.requests.swap_remove(n);
            match request.poll_unpin(cx) {
                Poll::Ready((Ok(val), idx)) => {
                    let response_weight = this.inner.providers[idx].weight;
                    if let Some((_, weight)) = this.responses.iter_mut().find(|(v, _)| &val == v) {
                        // add the weight to equal response value
                        *weight += response_weight;
                        if *weight >= this.inner.quorum_weight {
                            // reached quorum with multiple responses
                            return Poll::Ready(Ok(val))
                        } else {
                            this.responses.push((val, response_weight));
                        }
                    } else if response_weight >= this.inner.quorum_weight {
                        // reached quorum with single response
                        return Poll::Ready(Ok(val))
                    } else {
                        this.responses.push((val, response_weight));
                    }
                }
                Poll::Ready((Err(err), _)) => this.errors.push(err),
                _ => {
                    this.requests.push(request);
                }
            }
        }

        if this.requests.is_empty() {
            // No more requests and no quorum reached
            this.responses.sort_by(|a, b| b.1.cmp(&a.1));
            let values = std::mem::take(&mut this.responses).into_iter().map(|r| r.0).collect();
            let errors = std::mem::take(&mut this.errors);
            Poll::Ready(Err(QuorumError::NoQuorumReached { values, errors }))
        } else {
            Poll::Pending
        }
    }
}

/// The configuration of a provider for the `QuorumProvider`
#[derive(Debug, Clone)]
pub struct WeightedProvider<T> {
    inner: T,
    weight: u64,
}

impl<T> WeightedProvider<T> {
    /// Create a `WeightedProvider` with weight `1`
    pub fn new(inner: T) -> Self {
        Self::with_weight(inner, 1)
    }

    /// Instantiate a `WeightedProvider` with a set weight
    pub fn with_weight(inner: T, weight: u64) -> Self {
        assert!(weight > 0);
        Self { inner, weight }
    }
}

#[derive(Error, Debug)]
/// Error thrown when sending an HTTP request
pub enum QuorumError {
    #[error("No Quorum reached.")]
    /// NoQuorumReached
    NoQuorumReached {
        /// Returned responses
        values: Vec<Value>,
        /// Returned errors
        errors: Vec<ProviderError>,
    },
}

impl crate::RpcError for QuorumError {
    fn as_error_response(&self) -> Option<&super::JsonRpcError> {
        None
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        None
    }
}

impl From<QuorumError> for ProviderError {
    fn from(src: QuorumError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

/// Wrapper trait for [`crate::JsonRpcClient`] that erases generics and is
/// object-safe. This trait is not intended for outside implementation
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait JsonRpcClientWrapper: Send + Sync + Debug {
    /// Make a request, as [`crate::JsonRpcClient`]
    async fn request(&self, method: &str, params: QuorumParams) -> Result<Value, ProviderError>;
}
type NotificationStream =
    Box<dyn futures_core::Stream<Item = Box<RawValue>> + Send + Unpin + 'static>;

pub trait PubsubClientWrapper: JsonRpcClientWrapper {
    /// Add a subscription to this transport
    fn subscribe(&self, id: U256) -> Result<NotificationStream, ProviderError>;

    /// Remove a subscription from this transport
    fn unsubscribe(&self, id: U256) -> Result<(), ProviderError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<C: JsonRpcClient> JsonRpcClientWrapper for C {
    async fn request(&self, method: &str, params: QuorumParams) -> Result<Value, ProviderError> {
        let fut = if let QuorumParams::Value(params) = params {
            JsonRpcClient::request(self, method, params)
        } else {
            JsonRpcClient::request(self, method, ())
        };

        Ok(fut.await.map_err(C::Error::into)?)
    }
}
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl JsonRpcClientWrapper for Box<dyn JsonRpcClientWrapper> {
    async fn request(&self, method: &str, params: QuorumParams) -> Result<Value, ProviderError> {
        self.as_ref().request(method, params).await
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl JsonRpcClientWrapper for Box<dyn PubsubClientWrapper> {
    async fn request(&self, method: &str, params: QuorumParams) -> Result<Value, ProviderError> {
        self.as_ref().request(method, params).await
    }
}

impl<C: PubsubClient> PubsubClientWrapper for C
where
    <C as PubsubClient>::NotificationStream: 'static,
{
    fn subscribe(&self, id: U256) -> Result<NotificationStream, ProviderError> {
        Ok(Box::new(PubsubClient::subscribe(self, id).map_err(C::Error::into)?))
    }

    fn unsubscribe(&self, id: U256) -> Result<(), ProviderError> {
        PubsubClient::unsubscribe(self, id).map_err(C::Error::into)
    }
}

impl PubsubClientWrapper for Box<dyn PubsubClientWrapper> {
    fn subscribe(&self, id: U256) -> Result<NotificationStream, ProviderError> {
        self.as_ref().subscribe(id)
    }

    fn unsubscribe(&self, id: U256) -> Result<(), ProviderError> {
        self.as_ref().unsubscribe(id)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<C> JsonRpcClient for QuorumProvider<C>
where
    C: JsonRpcClientWrapper,
{
    type Error = ProviderError;

    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, Self::Error> {
        let mut params = if std::mem::size_of::<T>() == 0 {
            // we don't want `()` to become `"null"`.
            QuorumParams::Zst
        } else {
            QuorumParams::Value(serde_json::to_value(params)?)
        };
        self.normalize_request(method, &mut params).await;

        let requests = self
            .providers
            .iter()
            .enumerate()
            .map(|(idx, provider)| {
                let params = params.clone();
                let fut = provider.inner.request(method, params).map(move |res| (res, idx));
                Box::pin(fut) as PendingRequest
            })
            .collect::<Vec<_>>();

        let value = QuorumRequest::new(self, requests).await?;
        Ok(serde_json::from_value(value)?)
    }
}

// A stream that returns a value and the weight of its provider
type WeightedNotificationStream =
    Pin<Box<dyn futures_core::Stream<Item = (Box<RawValue>, u64)> + Send + Unpin + 'static>>;

/// A Subscription stream that only yields the next value if the underlying
/// providers reached quorum.
pub struct QuorumStream {
    // Weight required to reach quorum
    quorum_weight: u64,
    /// The different notifications with their cumulative weight
    responses: Vec<(Box<RawValue>, u64)>,
    /// All provider notification streams
    active: Vec<WeightedNotificationStream>,
    /// Provider streams that already yielded a new value and are waiting for
    /// active to finish
    benched: Vec<WeightedNotificationStream>,
}

impl QuorumStream {
    fn new(quorum_weight: u64, notifications: Vec<WeightedNotificationStream>) -> Self {
        Self { quorum_weight, responses: Vec::new(), active: notifications, benched: Vec::new() }
    }
}

impl Stream for QuorumStream {
    type Item = Box<RawValue>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.active.is_empty() {
            std::mem::swap(&mut this.active, &mut this.benched);
        }

        for n in (0..this.active.len()).rev() {
            let mut stream = this.active.swap_remove(n);

            match stream.poll_next_unpin(cx) {
                Poll::Ready(Some((val, response_weight))) => {
                    if let Some((_, weight)) =
                        this.responses.iter_mut().find(|(v, _)| val.get() == v.get())
                    {
                        *weight += response_weight;
                        if *weight >= this.quorum_weight {
                            // reached quorum with multiple notification
                            this.benched.push(stream);
                            return Poll::Ready(Some(val))
                        } else {
                            this.responses.push((val, response_weight));
                        }
                    } else if response_weight >= this.quorum_weight {
                        // reached quorum with single notification
                        this.benched.push(stream);
                        return Poll::Ready(Some(val))
                    } else {
                        this.responses.push((val, response_weight));
                    }

                    this.benched.push(stream);
                }
                Poll::Ready(None) => {}
                _ => {
                    this.active.push(stream);
                }
            }
        }

        if this.active.is_empty() && this.benched.is_empty() {
            return Poll::Ready(None)
        }
        Poll::Pending
    }
}

impl<C> PubsubClient for QuorumProvider<C>
where
    C: PubsubClientWrapper,
{
    type NotificationStream = QuorumStream;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, Self::Error> {
        let id = id.into();
        let mut notifications = Vec::with_capacity(self.providers.len());
        for provider in &self.providers {
            let weight = provider.weight;
            let fut = provider.inner.subscribe(id)?.map(move |val| (val, weight));
            notifications.push(Box::pin(fut) as WeightedNotificationStream);
        }
        Ok(QuorumStream::new(self.quorum_weight, notifications))
    }

    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), Self::Error> {
        let id = id.into();
        for provider in &self.providers {
            provider.inner.unsubscribe(id)?;
        }
        Ok(())
    }
}

/// Helper type that can be used to pass through the `params` value.
/// This is necessary because the wrapper provider is supposed to skip the `params` if it's of
/// size 0, see `crate::transports::common::Request`
#[derive(Clone)]
pub enum QuorumParams {
    Value(Value),
    Zst,
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::{Quorum, QuorumProvider, WeightedProvider};
    use crate::{Middleware, MockProvider, Provider};
    use ethers_core::types::U64;

    async fn test_quorum(q: Quorum) {
        let num = 5u64;
        let value = U64::from(42);
        let mut providers = Vec::new();
        let mut mocked = Vec::new();
        for _ in 0..num {
            let mock = MockProvider::new();
            mock.push(value).unwrap();
            providers.push(WeightedProvider::new(mock.clone()));
            mocked.push(mock);
        }
        let quorum = QuorumProvider::builder().add_providers(providers).quorum(q).build();
        let quorum_weight = quorum.quorum_weight;

        let provider = Provider::quorum(quorum);
        let blk = provider.get_block_number().await.unwrap();
        assert_eq!(blk, value);

        // count the number of providers that returned a value
        let requested =
            mocked.iter().filter(|mock| mock.assert_request("eth_blockNumber", ()).is_ok()).count();

        match q {
            Quorum::All => {
                assert_eq!(requested as u64, num);
            }
            Quorum::Majority => {
                assert_eq!(requested as u64, quorum_weight);
            }
            Quorum::Percentage(pct) => {
                let expected = num * (pct as u64) / 100;
                assert_eq!(requested, expected as usize);
            }
            Quorum::ProviderCount(count) => {
                assert_eq!(requested, count);
            }
            Quorum::Weight(w) => {
                assert_eq!(requested as u64, w);
            }
        }
    }

    #[tokio::test]
    async fn majority_quorum() {
        test_quorum(Quorum::Majority).await
    }

    #[tokio::test]
    async fn percentage_quorum() {
        test_quorum(Quorum::Percentage(100)).await
    }

    #[tokio::test]
    async fn count_quorum() {
        test_quorum(Quorum::ProviderCount(3)).await
    }

    #[tokio::test]
    async fn weight_quorum() {
        test_quorum(Quorum::Weight(5)).await
    }

    #[tokio::test]
    async fn all_quorum() {
        test_quorum(Quorum::All).await
    }
}
