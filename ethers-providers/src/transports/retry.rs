//! A [JsonRpcClient] implementation that retries requests filtered by [RetryPolicy]
//! with an exponential backoff.

use super::{common::JsonRpcError, http::ClientError};
use crate::{provider::ProviderError, JsonRpcClient};

use std::{
    fmt::Debug,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use tracing::trace;

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
    /// available CPU per second
    compute_units_per_second: u64,
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
        Self {
            inner,
            requests_enqueued: AtomicU32::new(0),
            policy,
            max_retry,
            initial_backoff,
            // alchemy max cpus <https://github.com/alchemyplatform/alchemy-docs/blob/master/documentation/compute-units.md#rate-limits-cups>
            compute_units_per_second: 330,
        }
    }

    /// Sets the free compute units per second limit.
    ///
    /// This is the maximum number of weighted request that can be handled per second by the
    /// endpoint before rate limit kicks in.
    ///
    /// This is used to guesstimate how long to wait until to retry again
    pub fn set_compute_units(&mut self, cpus: u64) -> &mut Self {
        self.compute_units_per_second = cpus;
        self
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
        // Helper type that caches the `params` value across several retries
        // This is necessary because the wrapper provider is supposed to skip he `params` if it's of
        // size 0, see `crate::transports::common::Request`
        enum RetryParams<Params> {
            Value(Params),
            Zst(()),
        }

        let params = if std::mem::size_of::<A>() == 0 {
            RetryParams::Zst(())
        } else {
            let params =
                serde_json::to_value(params).map_err(|err| RetryClientError::SerdeJson(err))?;
            RetryParams::Value(params)
        };

        let ahead_in_queue = self.requests_enqueued.fetch_add(1, Ordering::SeqCst) as u64;

        let mut retry_number: u32 = 0;

        loop {
            let err;

            // hack to not hold `R` across an await in the sleep future and prevent requiring
            // R: Send + Sync
            {
                let resp = match params {
                    RetryParams::Value(ref params) => self.inner.request(method, params).await,
                    RetryParams::Zst(unit) => self.inner.request(method, unit).await,
                };
                match resp {
                    Ok(ret) => {
                        self.requests_enqueued.fetch_sub(1, Ordering::SeqCst);
                        return Ok(ret)
                    }
                    Err(err_) => err = err_,
                }
            }

            retry_number += 1;
            if retry_number > self.max_retry {
                trace!("request timed out after {} retries", self.max_retry);
                return Err(RetryClientError::TimeoutError)
            }

            let should_retry = self.policy.should_retry(&err);
            if should_retry {
                let current_queued_requests = self.requests_enqueued.load(Ordering::SeqCst) as u64;
                // using `retry_number` for creating back pressure because
                // of already queued requests
                // this increases exponentially with retries and adds a delay based on how many
                // requests are currently queued
                let mut next_backoff = self.initial_backoff * 2u64.pow(retry_number);

                // requests are usually weighted and can vary from 10 CU to several 100 CU, cheaper
                // requests are more common some example alchemy weights:
                // - `eth_getStorageAt`: 17
                // - `eth_getBlockByNumber`: 16
                // - `eth_newFilter`: 20
                //
                // (coming from forking mode) assuming here that storage request will be the driver
                // for Rate limits we choose `17` as the average cost of any request
                const AVG_COST: u64 = 17u64;
                let seconds_to_wait_for_compute_budge = compute_unit_offset_in_secs(
                    AVG_COST,
                    self.compute_units_per_second,
                    current_queued_requests,
                    ahead_in_queue,
                );
                // backoff is measured in millis
                next_backoff += seconds_to_wait_for_compute_budge * 1000;

                trace!("retrying and backing off for {}ms", next_backoff);
                tokio::time::sleep(Duration::from_millis(next_backoff)).await;
            } else {
                trace!(err = ?err, "should not retry");
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

/// Calculates an offset in seconds by taking into account the number of currently queued requests,
/// number of requests that were ahead in the queue when the request was first issued, the average
/// cost a weighted request (heuristic), and the number of available compute units per seconds.
///
/// Returns the number of seconds (the unit the remote endpoint measures compute budget) a request
/// is supposed to wait to not get rate limited. The budget per second is
/// `compute_units_per_second`, assuming an average cost of `avg_cost` this allows (in theory)
/// `compute_units_per_second / avg_cost` requests per seconds without getting rate limited.
/// By taking into account the number of concurrent request and the position in queue when the
/// request was first issued and determine the number of seconds a request is supposed to wait, if
/// at all
fn compute_unit_offset_in_secs(
    avg_cost: u64,
    compute_units_per_second: u64,
    current_queued_requests: u64,
    ahead_in_queue: u64,
) -> u64 {
    let request_capacity_per_second = compute_units_per_second.saturating_div(avg_cost);
    if current_queued_requests > request_capacity_per_second {
        current_queued_requests.min(ahead_in_queue).saturating_div(request_capacity_per_second)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // assumed average cost of a request
    const AVG_COST: u64 = 17u64;
    const COMPUTE_UNITS: u64 = 330u64;

    fn compute_offset(current_queued_requests: u64, ahead_in_queue: u64) -> u64 {
        compute_unit_offset_in_secs(
            AVG_COST,
            COMPUTE_UNITS,
            current_queued_requests,
            ahead_in_queue,
        )
    }

    #[test]
    fn can_measure_unit_offset_single_request() {
        let current_queued_requests = 1;
        let ahead_in_queue = 0;
        let to_wait = compute_offset(current_queued_requests, ahead_in_queue);
        assert_eq!(to_wait, 0);

        let current_queued_requests = 19;
        let ahead_in_queue = 18;
        let to_wait = compute_offset(current_queued_requests, ahead_in_queue);
        assert_eq!(to_wait, 0);
    }

    #[test]
    fn can_measure_unit_offset_1x_over_budget() {
        let current_queued_requests = 20;
        let ahead_in_queue = 19;
        let to_wait = compute_offset(current_queued_requests, ahead_in_queue);
        // need to wait 1 second
        assert_eq!(to_wait, 1);
    }

    #[test]
    fn can_measure_unit_offset_2x_over_budget() {
        let current_queued_requests = 49;
        let ahead_in_queue = 48;
        let to_wait = compute_offset(current_queued_requests, ahead_in_queue);
        // need to wait 1 second
        assert_eq!(to_wait, 2);

        let current_queued_requests = 49;
        let ahead_in_queue = 20;
        let to_wait = compute_offset(current_queued_requests, ahead_in_queue);
        // need to wait 1 second
        assert_eq!(to_wait, 1);
    }
}
