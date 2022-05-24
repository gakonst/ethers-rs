//! A [JsonRpcClient] implementation that retries requests filtered by [RetryPolicy]
//! and [Backoff] strategy.

use super::http::ClientError;
use crate::{provider::ProviderError, JsonRpcClient};

use std::{
    clone::Clone,
    fmt::Debug,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
    time::Duration,
};

use async_trait::async_trait;
use backoff::ExponentialBackoff as BackoffExponentialBackoff;
#[cfg(not(target_arch = "wasm32"))]
use futures_timer::Delay;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
#[cfg(target_arch = "wasm32")]
use wasm_timer::Delay;

/// [RetryPolicy] defines logic for which [JsonRpcClient::Error] instances should
/// the client retry the request and try to recover from.
pub trait RetryPolicy<E> {
    fn should_retry(&self, error: &E) -> bool;
}

/// [Backoff] defines the logic for determining the [Duration] for which
/// the client must backoff before retrying the request.
pub trait Backoff {
    ///
    fn next_backoff(&mut self, total_enqueued_requests: u32, retry_number: u64)
        -> Option<Duration>;
}

/// [RetryClient] presents as a wrapper around [JsonRpcClient] that will retry
/// requests based on [Backoff] strategy and filtering based on [RetryPolicy].
#[derive(Debug)]
pub struct RetryClient<T, U, B>
where
    T: JsonRpcClient,
    T::Error: Sync + Send + 'static,
    U: RetryPolicy<T::Error> + Send + Sync + Debug,
    B: Backoff + Send + Sync,
{
    inner: T,
    requests_enqueued: AtomicU32,
    policy: U,
    backoff: Mutex<B>,
}

impl<T, U, B> RetryClient<T, U, B>
where
    T: JsonRpcClient,
    T::Error: Sync + Send + 'static,
    U: RetryPolicy<T::Error> + Send + Sync + Debug,
    B: Backoff + Send + Sync,
{
    /// Example:
    ///
    /// ```no_run
    /// let http = Http::new(Url::parse("http://localhost:8545").unwrap());
    /// let delay = Duration::new(10, 0);
    /// let client = RetryClient::new(http, HttpRateLimitRetryPolicy::new(), DelayedQueuedBackoff::new(delay));
    /// ```
    pub fn new(inner: T, policy: U, backoff: B) -> Self {
        Self { inner, requests_enqueued: AtomicU32::new(0), policy, backoff: Mutex::new(backoff) }
    }
}

/// Error thrown when:
/// 1. Internal client throws an error we do not wish to try to recover from.
/// 2. Params serialization failed.
/// 3. Request timed out i.e. `Backoff::next_backoff` returned `None`.
#[derive(Error, Debug)]
pub enum RetryClientError<T>
where
    T: JsonRpcClient,
    T::Error: Sync + Send + 'static,
{
    #[error(transparent)]
    ProviderError(T::Error),
    TimeoutError,
    #[error(transparent)]
    SerdeJson(serde_json::Error),
}

impl<T> std::fmt::Display for RetryClientError<T>
where
    T: JsonRpcClient,
    <T as JsonRpcClient>::Error: Sync + Send + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T> From<RetryClientError<T>> for ProviderError
where
    T: JsonRpcClient + 'static,
    <T as JsonRpcClient>::Error: Sync + Send + 'static,
{
    fn from(src: RetryClientError<T>) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T, U, B> JsonRpcClient for RetryClient<T, U, B>
where
    T: JsonRpcClient + 'static,
    T::Error: Sync + Send + 'static,
    U: RetryPolicy<T::Error> + Send + Sync + Debug,
    B: Backoff + Send + Sync + Debug + Clone,
{
    type Error = RetryClientError<T>;

    async fn request<A, R>(&self, method: &str, params: A) -> Result<R, Self::Error>
    where
        A: std::fmt::Debug + Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        self.requests_enqueued.fetch_add(1, Ordering::SeqCst);

        let params =
            serde_json::to_value(params).map_err(|err| RetryClientError::SerdeJson(err))?;

        let mut retry_number = 0;

        loop {
            let err;

            // hack to not hold `R` across an await in the sleep future and prevent requiring
            // R: Send + Sync
            {
                let ret = self.inner.request(method, params.clone()).await;
                if let Ok(ret) = ret {
                    return Ok(ret)
                }

                err = ret.err().unwrap();
            }

            retry_number += 1;
            let should_retry = self.policy.should_retry(&err);
            if should_retry {
                let next_backoff = self
                    .backoff
                    .lock()
                    .unwrap()
                    .next_backoff(self.requests_enqueued.load(Ordering::SeqCst), retry_number);
                if next_backoff.is_none() {
                    return Err(RetryClientError::TimeoutError)
                }
                let next_backoff = next_backoff.unwrap();
                Delay::new(next_backoff).await;
                continue
            } else {
                return Err(RetryClientError::ProviderError(err))
            }
        }
    }
}

/// Implements [RetryPolicy] that will retry requests that errored with
/// status code 429 i.e. TOO_MANY_REQUESTS
#[derive(Debug)]
pub struct HttpRateLimitRetryPolicy;

impl RetryPolicy<ClientError> for HttpRateLimitRetryPolicy {
    fn should_retry(&self, error: &ClientError) -> bool {
        match error {
            ClientError::ReqwestError(err) => {
                err.status() == Some(http::StatusCode::TOO_MANY_REQUESTS)
            }
            _ => false,
        }
    }
}

/// Implements [Backoff] strategy that imposes 1 request per [`delay`](Duration)
/// until max retries have reached.
pub struct DelayedQueuedBackoff {
    delay: Duration,
    max_retry: u64,
}

impl DelayedQueuedBackoff {
    pub fn new(delay: Duration, max_retry: u64) -> Self {
        Self { delay, max_retry }
    }
}

impl Backoff for DelayedQueuedBackoff {
    fn next_backoff(
        &mut self,
        total_enqueued_requests: u32,
        retry_number: u64,
    ) -> Option<Duration> {
        if retry_number > self.max_retry {
            None
        } else {
            let secs = self.delay.as_secs();
            let nsecs = self.delay.subsec_nanos();
            Some(Duration::new(
                secs * total_enqueued_requests as u64,
                nsecs * total_enqueued_requests,
            ))
        }
    }
}

/// [Backoff] strategy that uses [backoff::ExponentialBackoff] internally.
pub struct ExponentialBackoff {
    internal: BackoffExponentialBackoff,
}

impl ExponentialBackoff {
    pub fn new() -> Self {
        Self { internal: BackoffExponentialBackoff::default() }
    }
}

impl std::default::Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for ExponentialBackoff {
    type Target = BackoffExponentialBackoff;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl std::ops::DerefMut for ExponentialBackoff {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.internal
    }
}

impl Backoff for ExponentialBackoff {
    fn next_backoff(
        &mut self,
        _total_enqueued_requests: u32,
        _retry_number: u64,
    ) -> Option<Duration> {
        backoff::backoff::Backoff::next_backoff(&mut self.internal)
    }
}
