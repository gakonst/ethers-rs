//! A [JsonRpcClient] implementation that retries requests filtered by [RetryPolicy]
//! with an exponential backoff.

use super::{common::JsonRpcError, http::ClientError};
use crate::{provider::ProviderError, JsonRpcClient};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::Debug,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};
use thiserror::Error;
use tracing::trace;

/// [RetryPolicy] defines logic for which [JsonRpcClient::Error] instances should
/// the client retry the request and try to recover from.
pub trait RetryPolicy<E>: Send + Sync + Debug {
    /// Whether to retry the request based on the given `error`
    fn should_retry(&self, error: &E) -> bool;

    /// Providers may include the `backoff` in the error response directly
    fn backoff_hint(&self, error: &E) -> Option<Duration>;
}

/// [RetryClient] presents as a wrapper around [JsonRpcClient] that will retry
/// requests based with an exponential backoff and filtering based on [RetryPolicy].
///
/// The `RetryPolicy`, mainly for rate-limiting errors, can be adjusted for specific applications,
/// endpoints. In addition to the `RetryPolicy` errors due to connectivity issues, like timed out
/// connections or responses in range `5xx` can be retried separately.
///
/// # Example
///
/// ```
/// #  async fn demo() {
/// use ethers_providers::{Http, RetryClient, RetryClientBuilder, HttpRateLimitRetryPolicy};
/// use std::time::Duration;
/// use url::Url;
///
/// let http = Http::new(Url::parse("http://localhost:8545").unwrap());
/// let client = RetryClientBuilder::default()
///     .rate_limit_retries(10)
///     .timeout_retries(3)
///     .initial_backoff(Duration::from_millis(500))
///     .build(http, Box::new(HttpRateLimitRetryPolicy::default()));
/// # }
/// ```
#[derive(Debug)]
pub struct RetryClient<T>
where
    T: JsonRpcClient,
    T::Error: Sync + Send + 'static,
{
    inner: T,
    requests_enqueued: AtomicU32,
    /// The policy to use to determine whether to retry a request due to rate limiting
    policy: Box<dyn RetryPolicy<T::Error>>,
    /// How many connection `TimedOut` should be retried.
    timeout_retries: u32,
    /// How many retries for rate limited responses
    rate_limit_retries: u32,
    /// How long to wait initially
    initial_backoff: Duration,
    /// available CPU per second
    compute_units_per_second: u64,
}

impl<T> RetryClient<T>
where
    T: JsonRpcClient,
    T::Error: Sync + Send + 'static,
{
    /// Creates a new `RetryClient` that wraps a client and adds retry and backoff support
    ///
    /// # Example
    ///
    /// ```
    /// 
    /// # async fn demo() {
    /// use ethers_providers::{Http, RetryClient, HttpRateLimitRetryPolicy};
    /// use std::time::Duration;
    /// use url::Url;
    ///
    /// let http = Http::new(Url::parse("http://localhost:8545").unwrap());
    /// let backoff_timeout = 3000; // in ms
    /// let max_retries = 10;
    /// let client = RetryClient::new(http, Box::new(HttpRateLimitRetryPolicy::default()), max_retries, backoff_timeout);
    ///
    /// # }
    /// ```
    pub fn new(
        inner: T,
        policy: Box<dyn RetryPolicy<T::Error>>,
        max_retry: u32,
        // in milliseconds
        initial_backoff: u64,
    ) -> Self {
        RetryClientBuilder::default()
            .initial_backoff(Duration::from_millis(initial_backoff))
            .rate_limit_retries(max_retry)
            .build(inner, policy)
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RetryClientBuilder {
    /// How many connection `TimedOut` should be retried.
    timeout_retries: u32,
    /// How many retries for rate limited responses
    rate_limit_retries: u32,
    /// How long to wait initially
    initial_backoff: Duration,
    /// available CPU per second
    compute_units_per_second: u64,
}

// === impl RetryClientBuilder ===

impl RetryClientBuilder {
    /// Sets the number of retries after a connection times out
    ///
    /// **Note:** this will only be used for `request::Error::TimedOut`
    pub fn timeout_retries(mut self, timeout_retries: u32) -> Self {
        self.timeout_retries = timeout_retries;
        self
    }

    /// How many retries for rate limited responses
    pub fn rate_limit_retries(mut self, rate_limit_retries: u32) -> Self {
        self.rate_limit_retries = rate_limit_retries;
        self
    }

    /// Sets the number of assumed available compute units per second
    ///
    /// See also, <https://github.com/alchemyplatform/alchemy-docs/blob/master/documentation/compute-units.md#rate-limits-cups>
    pub fn compute_units_per_second(mut self, compute_units_per_second: u64) -> Self {
        self.compute_units_per_second = compute_units_per_second;
        self
    }

    /// Sets the duration to wait initially before retrying
    pub fn initial_backoff(mut self, initial_backoff: Duration) -> Self {
        self.initial_backoff = initial_backoff;
        self
    }

    /// Creates the `RetryClient` with the configured settings
    pub fn build<T>(self, client: T, policy: Box<dyn RetryPolicy<T::Error>>) -> RetryClient<T>
    where
        T: JsonRpcClient,
        T::Error: Sync + Send + 'static,
    {
        let RetryClientBuilder {
            timeout_retries,
            rate_limit_retries,
            initial_backoff,
            compute_units_per_second,
        } = self;
        RetryClient {
            inner: client,
            requests_enqueued: AtomicU32::new(0),
            policy,
            timeout_retries,
            rate_limit_retries,
            initial_backoff,
            compute_units_per_second,
        }
    }
}

// Some sensible defaults
impl Default for RetryClientBuilder {
    fn default() -> Self {
        Self {
            timeout_retries: 3,
            // this should be enough to even out heavy loads
            rate_limit_retries: 10,
            initial_backoff: Duration::from_millis(1000),
            // alchemy max cpus <https://github.com/alchemyplatform/alchemy-docs/blob/master/documentation/compute-units.md#rate-limits-cups>
            compute_units_per_second: 330,
        }
    }
}

/// Error thrown when:
/// 1. Internal client throws an error we do not wish to try to recover from.
/// 2. Params serialization failed.
/// 3. Request timed out i.e. max retries were already made.
#[derive(Error, Debug)]
pub enum RetryClientError {
    #[error(transparent)]
    ProviderError(ProviderError),
    TimeoutError,
    #[error(transparent)]
    SerdeJson(serde_json::Error),
    TimerError,
}

impl std::fmt::Display for RetryClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<RetryClientError> for ProviderError {
    fn from(src: RetryClientError) -> Self {
        match src {
            RetryClientError::ProviderError(err) => err,
            RetryClientError::TimeoutError => ProviderError::JsonRpcClientError(Box::new(src)),
            RetryClientError::SerdeJson(err) => err.into(),
            RetryClientError::TimerError => ProviderError::JsonRpcClientError(Box::new(src)),
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<T> JsonRpcClient for RetryClient<T>
where
    T: JsonRpcClient + 'static,
    T::Error: Sync + Send + 'static,
{
    type Error = RetryClientError;

    async fn request<A, R>(&self, method: &str, params: A) -> Result<R, Self::Error>
    where
        A: Debug + Serialize + Send + Sync,
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
            let params = serde_json::to_value(params).map_err(RetryClientError::SerdeJson)?;
            RetryParams::Value(params)
        };

        let ahead_in_queue = self.requests_enqueued.fetch_add(1, Ordering::SeqCst) as u64;

        let mut rate_limit_retry_number: u32 = 0;
        let mut timeout_retries: u32 = 0;

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

            let should_retry = self.policy.should_retry(&err);
            if should_retry {
                rate_limit_retry_number += 1;
                if rate_limit_retry_number > self.rate_limit_retries {
                    trace!("request timed out after {} retries", self.rate_limit_retries);
                    return Err(RetryClientError::TimeoutError)
                }

                let current_queued_requests = self.requests_enqueued.load(Ordering::SeqCst) as u64;

                // try to extract the requested backoff from the error or compute the next backoff
                // based on retry count
                let mut next_backoff = self.policy.backoff_hint(&err).unwrap_or_else(|| {
                    Duration::from_millis(self.initial_backoff.as_millis() as u64)
                });

                // requests are usually weighted and can vary from 10 CU to several 100 CU, cheaper
                // requests are more common some example alchemy weights:
                // - `eth_getStorageAt`: 17
                // - `eth_getBlockByNumber`: 16
                // - `eth_newFilter`: 20
                //
                // (coming from forking mode) assuming here that storage request will be the driver
                // for Rate limits we choose `17` as the average cost of any request
                const AVG_COST: u64 = 17u64;
                let seconds_to_wait_for_compute_budget = compute_unit_offset_in_secs(
                    AVG_COST,
                    self.compute_units_per_second,
                    current_queued_requests,
                    ahead_in_queue,
                );
                next_backoff += Duration::from_secs(seconds_to_wait_for_compute_budget);

                trace!("retrying and backing off for {:?}", next_backoff);

                #[cfg(target_arch = "wasm32")]
                wasm_timer::Delay::new(next_backoff)
                    .await
                    .map_err(|_| RetryClientError::TimerError)?;

                #[cfg(not(target_arch = "wasm32"))]
                tokio::time::sleep(next_backoff).await;
            } else {
                let err: ProviderError = err.into();
                if timeout_retries < self.timeout_retries && maybe_connectivity(&err) {
                    timeout_retries += 1;
                    trace!(err = ?err, "retrying due to spurious network");
                    continue
                }

                trace!(err = ?err, "should not retry");
                self.requests_enqueued.fetch_sub(1, Ordering::SeqCst);
                return Err(RetryClientError::ProviderError(err))
            }
        }
    }
}

/// Implements [RetryPolicy] that will retry requests that errored with
/// status code 429 i.e. TOO_MANY_REQUESTS
///
/// Infura often fails with a `"header not found"` rpc error which is apparently linked to load
/// balancing, which are retried as well.
#[derive(Debug, Default)]
pub struct HttpRateLimitRetryPolicy;

impl RetryPolicy<ClientError> for HttpRateLimitRetryPolicy {
    fn should_retry(&self, error: &ClientError) -> bool {
        match error {
            ClientError::ReqwestError(err) => {
                err.status() == Some(http::StatusCode::TOO_MANY_REQUESTS)
            }
            ClientError::JsonRpcError(JsonRpcError { code, message, .. }) => {
                // alchemy throws it this way
                if *code == 429 {
                    return true
                }

                // alternative alchemy error for specific IPs
                if *code == -32016 && message.contains("rate limit") {
                    return true
                }

                match message.as_str() {
                    // this is commonly thrown by infura and is apparently a load balancer issue, see also <https://github.com/MetaMask/metamask-extension/issues/7234>
                    "header not found" => true,
                    // also thrown by infura if out of budget for the day and ratelimited
                    "daily request count exceeded, request rate limited" => true,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn backoff_hint(&self, error: &ClientError) -> Option<Duration> {
        if let ClientError::JsonRpcError(JsonRpcError { data, .. }) = error {
            let data = data.as_ref()?;

            // if daily rate limit exceeded, infura returns the requested backoff in the error
            // response
            let backoff_seconds = &data["rate"]["backoff_seconds"];
            // infura rate limit error
            if let Some(seconds) = backoff_seconds.as_u64() {
                return Some(Duration::from_secs(seconds))
            }
            if let Some(seconds) = backoff_seconds.as_f64() {
                return Some(Duration::from_secs(seconds as u64 + 1))
            }
        }

        None
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

/// Checks whether the `error` is the result of a connectivity issue, like
/// `request::Error::TimedOut`
fn maybe_connectivity(err: &ProviderError) -> bool {
    if let ProviderError::HTTPError(reqwest_err) = err {
        if reqwest_err.is_timeout() {
            return true
        }

        #[cfg(not(target_arch = "wasm32"))]
        if reqwest_err.is_connect() {
            return true
        }

        // Error HTTP codes (5xx) are considered connectivity issues and will prompt retry
        if let Some(status) = reqwest_err.status() {
            let code = status.as_u16();
            if (500..600).contains(&code) {
                return true
            }
        }
    }
    false
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

    #[test]
    fn can_extract_backoff() {
        let resp = r#"{"rate": {"allowed_rps": 1, "backoff_seconds": 30, "current_rps": 1.1}, "see": "https://infura.io/dashboard"}"#;

        let err = ClientError::JsonRpcError(JsonRpcError {
            code: 0,
            message: "daily request count exceeded, request rate limited".to_string(),
            data: Some(serde_json::from_str(resp).unwrap()),
        });
        let backoff = HttpRateLimitRetryPolicy.backoff_hint(&err).unwrap();
        assert_eq!(backoff, Duration::from_secs(30));

        let err = ClientError::JsonRpcError(JsonRpcError {
            code: 0,
            message: "daily request count exceeded, request rate limited".to_string(),
            data: Some(serde_json::Value::String("blocked".to_string())),
        });
        let backoff = HttpRateLimitRetryPolicy.backoff_hint(&err);
        assert!(backoff.is_none());
    }

    #[test]
    fn test_alchemy_ip_rate_limit() {
        let s = "{\"code\":-32016,\"message\":\"Your IP has exceeded its requests per second capacity. To increase your rate limits, please sign up for a free Alchemy account at https://www.alchemy.com/optimism.\"}";
        let err: JsonRpcError = serde_json::from_str(s).unwrap();
        let err = ClientError::JsonRpcError(err);

        let should_retry = HttpRateLimitRetryPolicy::default().should_retry(&err);
        assert!(should_retry);
    }
}
