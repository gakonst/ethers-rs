//! A [JsonRpcClient] implementation that retries requests on certain failure
//! with an exponential backoff.

use crate::{provider::ProviderError, JsonRpcClient};
use async_trait::async_trait;
use backoff::{backoff::Backoff, ExponentialBackoff};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

use std::{clone::Clone, fmt::Debug, sync::atomic::AtomicU64};

#[derive(Debug)]
pub struct RetryClient<T: JsonRpcClient, U: RetryPolicy<T::Error> + Send + Sync + Debug> {
    inner: T,
    requests_enqueued: AtomicU64,
    policy: U,
    backoff: ExponentialBackoff,
}

pub trait RetryPolicy<E> {
    fn should_retry(&self, error: &E) -> bool;
}

impl<T: JsonRpcClient, U: RetryPolicy<T::Error> + Send + Sync + Debug> RetryClient<T, U> {
    pub fn new(inner: T, policy: U, backoff: ExponentialBackoff) -> Self {
        Self { inner, requests_enqueued: AtomicU64::new(0), policy, backoff }
    }
}

#[derive(Error, Debug)]
enum RetryClientError<T>
where
    T: JsonRpcClient,
    T::Error: Sync + Send + 'static,
{
    #[error(transparent)]
    ProviderError(T::Error),
    TimeoutError,
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
impl<T, U> JsonRpcClient for RetryClient<T, U>
where
    T: JsonRpcClient + 'static,
    T::Error: Sync + Send + 'static,
    U: RetryPolicy<T::Error> + Send + Sync + Debug,
{
    type Error = RetryClientError<T>;

    async fn request<A, R>(&self, method: &str, params: A) -> Result<R, Self::Error>
    where
        A: std::fmt::Debug + Serialize + Send + Sync + Copy,
        R: DeserializeOwned,
    {
        let mut backoff = self.backoff.clone();
        loop {
            let ret: Result<R, T::Error> = self.inner.request(method, params).await;
            if ret.is_err() {
                let err = ret.err().unwrap();
                let should_retry = self.policy.should_retry(&err);
                if !should_retry {
                    return Err(RetryClientError::ProviderError(err))
                } else {
                    let next_backoff = backoff.next_backoff();
                    if next_backoff.is_none() {
                        return Err(RetryClientError::TimeoutError)
                    }
                }
            }
        }
    }
}
