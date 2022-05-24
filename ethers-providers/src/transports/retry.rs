//! A [JsonRpcClient] implementation that retries requests filtered by [RetryPolicy]
//! and [Backoff] strategy.

use super::http::ClientError;
use crate::{provider::ProviderError, JsonRpcClient};

use std::{
    clone::Clone,
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

/// [RetryPolicy] defines logic for which [JsonRpcClient::Error] instances should
/// the client retry the request and try to recover from.
pub trait RetryPolicy<E>: Send + Sync + Debug {
    fn should_retry(&self, error: &E) -> bool;
}

/// [RetryClient] presents as a wrapper around [JsonRpcClient] that will retry
/// requests based on [Backoff] strategy and filtering based on [RetryPolicy].
#[derive(Debug)]
pub struct RetryClient<T>
where
    T: JsonRpcClient,
    T::Error: Sync + Send + 'static,
{
    inner: T,
    requests_enqueued: AtomicU64,
    policy: Box<dyn RetryPolicy<T::Error>>,
    max_retry: u64,
    initial_backoff: u64,
}

impl<T> RetryClient<T>
where
    T: JsonRpcClient,
    T::Error: Sync + Send + 'static,
{
    /// Example:
    ///
    /// ```no_run
    /// let http = Http::new(Url::parse("http://localhost:8545").unwrap());
    /// let delay = Duration::new(10, 0);
    /// let client = RetryClient::new(http, HttpRateLimitRetryPolicy::new(), 10, 1);
    /// ```
    pub fn new(
        inner: T,
        policy: Box<dyn RetryPolicy<T::Error>>,
        max_retry: u64,
        // in seconds
        initial_backoff: u64,
    ) -> Self {
        Self { inner, requests_enqueued: AtomicU64::new(0), policy, max_retry, initial_backoff }
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

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T> JsonRpcClient for RetryClient<T>
where
    T: JsonRpcClient + 'static,
    T::Error: Sync + Send + 'static,
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

        let mut retry_number: u64 = 0;

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
            if retry_number > self.max_retry {
                return Err(RetryClientError::TimeoutError)
            }

            let should_retry = self.policy.should_retry(&err);
            if should_retry {
                let current_queued_requests = self.requests_enqueued.load(Ordering::SeqCst);
                let next_backoff = self.initial_backoff * (retry_number + current_queued_requests);
                tokio::time::sleep(Duration::new(next_backoff, 0)).await;
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
