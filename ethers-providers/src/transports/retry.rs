//! A [JsonRpcClient] implementation that retries requests filtered by [RetryPolicy]
//! with an exponential backoff.

use super::{common::JsonRpcError, http::ClientError};
use crate::{provider::ProviderError, JsonRpcClient};

use std::{
    clone::Clone,
    fmt::Debug,
    sync::atomic::{AtomicU32, Ordering},
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
/// requests based with an exponential backoff and filtering based on [RetryPolicy].
#[derive(Debug)]
pub struct RetryClient<T>
where
    T: JsonRpcClient,
    T::Error: Sync + Send + 'static,
{
    inner: T,
    requests_enqueued: AtomicU32,
    policy: Box<dyn RetryPolicy<T::Error>>,
    max_retry: u32,
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
    /// use ethers_providers::{Http, RetryClient, HttpRateLimitRetryPolicy};
    /// use std::time::Duration;
    /// use url::Url;
    ///
    /// let http = Http::new(Url::parse("http://localhost:8545").unwrap());
    /// let delay = Duration::new(10, 0);
    /// let client = RetryClient::new(http, Box::new(HttpRateLimitRetryPolicy), 10, 1);
    /// ```
    pub fn new(
        inner: T,
        policy: Box<dyn RetryPolicy<T::Error>>,
        max_retry: u32,
        // in milliseconds
        initial_backoff: u64,
    ) -> Self {
        Self { inner, requests_enqueued: AtomicU32::new(0), policy, max_retry, initial_backoff }
    }
}

/// Error thrown when:
/// 1. Internal client throws an error we do not wish to try to recover from.
/// 2. Params serialization failed.
/// 3. Request timed out i.e. max retries were already made.
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

        let mut retry_number: u32 = 0;

        loop {
            let err;

            // hack to not hold `R` across an await in the sleep future and prevent requiring
            // R: Send + Sync
            {
                match self.inner.request(method, params.clone()).await {
                    Ok(ret) => {
                        self.requests_enqueued.fetch_sub(1, Ordering::SeqCst);
                        return Ok(ret)
                    }
                    Err(err_) => err = err_,
                }
            }

            retry_number += 1;
            if retry_number > self.max_retry {
                return Err(RetryClientError::TimeoutError)
            }

            let should_retry = self.policy.should_retry(&err);
            if should_retry {
                let current_queued_requests = self.requests_enqueued.load(Ordering::SeqCst);
                // using `retry_number + current_queued_requests` for creating back pressure because
                // of already queued requests
                let next_backoff =
                    self.initial_backoff * 2u64.pow(retry_number + current_queued_requests);
                tokio::time::sleep(Duration::from_millis(next_backoff)).await;
            } else {
                self.requests_enqueued.fetch_sub(1, Ordering::SeqCst);
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
            // alchemy throws it this way
            ClientError::JsonRpcError(JsonRpcError { code, message: _, data: _ }) => *code == 429,
            _ => false,
        }
    }
}
