use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{provider::ProviderError, JsonRpcClient, PubsubClient};
use async_trait::async_trait;
use ethers_core::types::{U256, U64};
use futures_core::Stream;
use futures_util::{future, FutureExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{value::RawValue, Value};
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

    pub fn new(quorum: Quorum, providers: impl IntoIterator<Item = WeightedProvider<T>>) -> Self {
        Self::builder().add_providers(providers).quorum(quorum).build()
    }

    pub fn providers(&self) -> &[WeightedProvider<T>] {
        &self.providers
    }

    /// The weight at which the provider reached a quorum
    pub fn quorum_weight(&self) -> u64 {
        self.quorum_weight
    }

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
    /// For each inner provider, attempts to perform an RPC that returns a numeric value.
    /// This a quorum of the highest numbers returned by inner providers, and returns the minimum
    /// of these numbers.
    /// For example, if the quorum threshold is 2 of a set of 5 inner providers, and the following
    /// numbers are returned: [100, 101, 102, 103, 104], 103 will be returned.
    /// This is useful for getting block numbers or gas estimates.
    async fn get_quorum_number<N>(
        &self,
        method: &str,
        params: WrappedParams,
    ) -> Result<N, QuorumError>
    where
        N: Serialize + DeserializeOwned + Ord + Copy,
    {
        let mut queries = self
            .providers
            .iter()
            .map(|provider| {
                let params_clone = params.clone();
                Box::pin(async move {
                    let num = provider.inner.request(method, params_clone).await?;
                    serde_json::from_value::<N>(num)
                        .map(|b| (provider, b))
                        .map_err(ProviderError::from)
                })
            })
            .collect::<Vec<_>>();

        let mut numbers = vec![];
        let mut errors = vec![];
        while !queries.is_empty() {
            let (response, _index, remaining) = future::select_all(queries).await;
            queries = remaining;
            match response {
                Ok(v) => numbers.push(v),
                Err(e) => errors.push(e),
            }
        }

        numbers.sort_by(|(_, block_a), (_, block_b)| {
            // order by descending block number
            block_a.cmp(block_b).reverse()
        });

        // find the highest possible block number a quorum agrees on
        let mut cumulative_weight = 0;
        let mut aggregated_num: Option<N> = None;

        for (provider, n) in numbers.iter().copied() {
            cumulative_weight += provider.weight;
            // Sanity check the sorting
            debug_assert!(aggregated_num.is_none() || aggregated_num.unwrap() >= n);
            aggregated_num = Some(n);
            if cumulative_weight >= self.quorum_weight {
                return Ok(aggregated_num.unwrap())
            }
        }
        Err(QuorumError::NoQuorumReached {
            values: numbers
                .into_iter()
                .map(|(_, number)| {
                    serde_json::to_value(number).expect("Failed to serialize number")
                })
                .collect(),
            errors,
        })
    }

    /// Returns the block height that a _quorum_ of providers have reached.
    async fn get_quorum_block_number(&self) -> Result<U64, QuorumError> {
        self.get_quorum_number("eth_blockNumber", WrappedParams::Zst).await
    }

    /// Normalizes the request payload depending on the call
    async fn normalize_request(&self, method: &str, q_params: &mut WrappedParams) {
        let params = if let WrappedParams::Value(v) = q_params {
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
            "eth_estimateGas" |
            "trace_call" |
            "trace_block" => {
                // calls that include the block number in the params at the last index of json array
                if let Some(block) = params.as_array_mut().and_then(|arr| arr.last_mut()) {
                    self.replace_latest(block).await
                }
            }
            "eth_getBlockByNumber" => {
                // calls that include the block number in the params at the first index of json
                // array
                if let Some(block) = params.as_array_mut().and_then(|arr| arr.first_mut()) {
                    self.replace_latest(block).await
                }
            }
            _ => {}
        }
    }

    async fn replace_latest(&self, block: &mut Value) {
        if Some("latest") == block.as_str() {
            // replace `latest` with block height of a quorum of providers
            if let Ok(minimum) = self.get_quorum_block_number().await {
                *block = serde_json::to_value(minimum).expect("Failed to serialize U64")
            }
        }
    }
}

/// Determines when the provider reached a quorum
#[derive(Debug, Copy, Clone)]
pub enum Quorum {
    ///  The quorum is reached when all providers return the exact value
    All,
    /// The quorum is reached when the majority of the providers have returned a
    /// matching value, taking into account their weight.
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

impl Default for Quorum {
    fn default() -> Self {
        Quorum::Majority
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
    weight: u64,
    inner: T,
}

impl<T> WeightedProvider<T> {
    /// Create a `WeightedProvider` with weight `1`
    pub fn new(inner: T) -> Self {
        Self::with_weight(inner, 1)
    }

    pub fn with_weight(inner: T, weight: u64) -> Self {
        assert!(weight > 0);
        Self { inner, weight }
    }
}

#[derive(Error, Debug)]
/// Error thrown when sending an HTTP request
pub enum QuorumError {
    #[error("No Quorum reached. (Values: {:?}, Errors: {:?})", values, errors)]
    NoQuorumReached { values: Vec<Value>, errors: Vec<ProviderError> },
}

impl From<QuorumError> for ProviderError {
    fn from(src: QuorumError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait JsonRpcClientWrapper: Send + Sync + Debug {
    async fn request(&self, method: &str, params: WrappedParams) -> Result<Value, ProviderError>;
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
    async fn request(&self, method: &str, params: WrappedParams) -> Result<Value, ProviderError> {
        let fut = if let WrappedParams::Value(params) = params {
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
    async fn request(&self, method: &str, params: WrappedParams) -> Result<Value, ProviderError> {
        self.as_ref().request(method, params).await
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl JsonRpcClientWrapper for Box<dyn PubsubClientWrapper> {
    async fn request(&self, method: &str, params: WrappedParams) -> Result<Value, ProviderError> {
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
        let mut params = WrappedParams::new(params)?;
        self.normalize_request(method, &mut params).await;

        match method {
            // TODO: to robustly support eip-1559, we will likely need to also support
            // eth_feeHistory. This returns an object with various numbers rather than a
            // single number, so we'll need some additional code to handle this case.

            // For RPCs that return numbers that can vary amongst inner providers, come to quorum on
            // a single number
            "eth_blockNumber" | "eth_estimateGas" | "eth_gasPrice" | "eth_maxPriorityFeePerGas" => {
                let number: U256 = self.get_quorum_number(method, params).await?;
                // a little janky to convert to a string and back but we don't know for sure what
                // type R is and adding constraints for just this case feels wrong.
                let value = serde_json::to_value(number).expect("Failed to serialize number");
                Ok(serde_json::from_value(value)?)
            }
            "eth_sendTransaction" | "eth_sendRawTransaction" => {
                // non-idempotent requests may fail due to delays in processing even though the
                // operation was a success.

                // TODO: be more clever than to just accept any Ok response and to look for
                //   "nonce too low", "already known", or any other specific errors that indicate
                //   things were successful.

                let mut requests = self
                    .providers
                    .iter()
                    .enumerate()
                    .map(|(idx, provider)| {
                        let params = params.clone();
                        let fut = provider.inner.request(method, params).map(move |res| (res, idx));
                        Box::pin(fut) as PendingRequest
                    })
                    .collect::<Vec<_>>();

                let mut errors = vec![];
                let mut succeeded = None;
                // this does assume that there is a timeout on providers, otherwise it might hang.
                while !requests.is_empty() {
                    let ((res, _quorum_idx), _requests_idx, remaining) =
                        future::select_all(requests).await;
                    match res {
                        Ok(value) if succeeded.is_none() => {
                            succeeded = Some(value);
                        }
                        Ok(_) => {}
                        Err(err) => {
                            errors.push(err);
                        }
                    }
                    requests = remaining;
                }

                if let Some(value) = succeeded {
                    Ok(serde_json::from_value(value)?)
                } else {
                    Err(QuorumError::NoQuorumReached { values: vec![], errors }.into())
                }
            }
            _ => {
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
pub enum WrappedParams {
    Value(Value),
    Zst,
}

impl WrappedParams {
    pub fn new<T: Serialize>(params: T) -> Result<Self, serde_json::Error> {
        Ok(if std::mem::size_of::<T>() == 0 {
            // we don't want `()` to become `"null"`.
            WrappedParams::Zst
        } else {
            WrappedParams::Value(serde_json::to_value(params)?)
        })
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::{Quorum, QuorumProvider, WeightedProvider};
    use crate::{transports::quorum::WrappedParams, Middleware, MockProvider, Provider};
    use ethers_core::types::{U256, U64};

    async fn test_quorum(q: Quorum) {
        let num = 5u64;
        let value = U256::from(42);
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
        let blk = provider.get_chainid().await.unwrap();
        assert_eq!(blk, value);

        // count the number of providers that returned a value
        let requested =
            mocked.iter().filter(|mock| mock.assert_request("eth_chainId", ()).is_ok()).count();

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
    async fn test_get_quorum_block_number() {
        let mut providers = Vec::new();

        for value in [100, 101, 68, 100, 102] {
            let mock = MockProvider::new();
            for _ in 0..6 {
                mock.push(U64::from(value)).unwrap();
            }
            providers.push(WeightedProvider::new(mock.clone()));
        }

        let quorum = QuorumProvider::builder()
            .add_providers(providers.clone())
            .quorum(Quorum::ProviderCount(5))
            .build();
        assert_eq!(
            quorum.get_quorum_number::<U64>("foo", WrappedParams::Zst).await.unwrap().as_u64(),
            68
        );

        let quorum = QuorumProvider::builder()
            .add_providers(providers.clone())
            .quorum(Quorum::ProviderCount(4))
            .build();
        assert_eq!(
            quorum.get_quorum_number::<U64>("foo", WrappedParams::Zst).await.unwrap().as_u64(),
            100
        );

        let quorum = QuorumProvider::builder()
            .add_providers(providers.clone())
            .quorum(Quorum::ProviderCount(3))
            .build();
        assert_eq!(
            quorum.get_quorum_number::<U64>("foo", WrappedParams::Zst).await.unwrap().as_u64(),
            100
        );

        let quorum = QuorumProvider::builder()
            .add_providers(providers.clone())
            .quorum(Quorum::ProviderCount(2))
            .build();
        assert_eq!(
            quorum.get_quorum_number::<U64>("foo", WrappedParams::Zst).await.unwrap().as_u64(),
            101
        );

        let quorum = QuorumProvider::builder()
            .add_providers(providers.clone())
            .quorum(Quorum::ProviderCount(1))
            .build();
        assert_eq!(
            quorum.get_quorum_number::<U64>("foo", WrappedParams::Zst).await.unwrap().as_u64(),
            102
        );

        let quorum = QuorumProvider::builder()
            .add_providers(providers.clone())
            .quorum(Quorum::Majority)
            .build();
        assert_eq!(
            quorum.get_quorum_number::<U64>("foo", WrappedParams::Zst).await.unwrap().as_u64(),
            100
        );
    }

    #[tokio::test]
    async fn test_get_quorum_number_with_errors() {
        let mut providers = Vec::new();

        let mock = MockProvider::new();
        for _ in 0..2 {
            mock.push(U64::from(100)).unwrap();
        }
        providers.push(WeightedProvider::new(mock.clone()));

        let mock = MockProvider::new();
        // this one will error
        providers.push(WeightedProvider::new(mock.clone()));

        let mock = MockProvider::new();

        for _ in 0..2 {
            mock.push(U64::from(101)).unwrap();
        }
        providers.push(WeightedProvider::new(mock.clone()));

        let quorum = QuorumProvider::builder()
            .add_providers(providers.clone())
            .quorum(Quorum::ProviderCount(2))
            .build();
        assert_eq!(
            quorum.get_quorum_number::<U64>("foo", WrappedParams::Zst).await.unwrap().as_u64(),
            100
        );

        let quorum = QuorumProvider::builder()
            .add_providers(providers.clone())
            .quorum(Quorum::Majority)
            .build();
        assert_eq!(
            quorum.get_quorum_number::<U64>("foo", WrappedParams::Zst).await.unwrap().as_u64(),
            100
        );
    }

    #[tokio::test]
    async fn test_get_quorum_number_fails_to_reach_quorum() {
        let mut providers = Vec::new();

        let mock = MockProvider::new();
        for _ in 0..2 {
            mock.push(U64::from(100)).unwrap();
        }
        providers.push(WeightedProvider::new(mock.clone()));

        let mock = MockProvider::new();
        // this one will error
        providers.push(WeightedProvider::new(mock.clone()));

        let mock = MockProvider::new();
        // this one will error
        providers.push(WeightedProvider::new(mock.clone()));

        let quorum = QuorumProvider::builder()
            .add_providers(providers.clone())
            .quorum(Quorum::ProviderCount(2))
            .build();
        assert!(quorum.get_quorum_number::<U64>("foo", WrappedParams::Zst).await.is_err());

        let quorum = QuorumProvider::builder()
            .add_providers(providers.clone())
            .quorum(Quorum::Majority)
            .build();
        assert!(quorum.get_quorum_number::<U64>("foo", WrappedParams::Zst).await.is_err());
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
