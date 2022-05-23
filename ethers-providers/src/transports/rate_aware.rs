//! A [JsonRpcClient] implementation that is rate limit

use crate::{provider::ProviderError, JsonRpcClient};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct RateAwareClient<T> {
    quota: u64,
    duration: Duration,
    period_start: AtomicU64,
    requests_made: AtomicU64,
    client: T,
}

impl<T: JsonRpcClient> RateAwareClient<T> {
    pub fn new(quota: u64, duration: Duration, client: T) -> Self {
        Self {
            quota,
            duration,
            period_start: AtomicU64::new(Self::unix_now_secs()),
            requests_made: AtomicU64::new(0),
            client,
        }
    }

    fn requests_left(&self) -> u64 {
        if self.is_new_period() {
            return self.quota
        }

        let requests_made = self.requests_made.load(Ordering::SeqCst);
        self.quota - requests_made
    }

    fn is_new_period(&self) -> bool {
        Self::unix_now_secs() > self.current_period_end()
    }

    fn current_period_end(&self) -> u64 {
        self.period_start.load(Ordering::SeqCst) + self.duration.as_secs()
    }

    fn time_until_next_period(&self) -> Duration {
        let now = Self::unix_now_secs();
        let current_period_end = self.current_period_end();
        if now > current_period_end {
            Duration::new(0, 0)
        } else {
            Duration::new(current_period_end - now, 0)
        }
    }

    fn unix_now_secs() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    fn record_request(&self) {
        // record the request in current period or start a new period
        if !self.is_new_period() {
            self.requests_made.fetch_add(1, Ordering::SeqCst);
        } else {
            self.requests_made.store(1, Ordering::SeqCst);
            self.period_start.store(Self::unix_now_secs(), Ordering::SeqCst);
        }
    }
}

#[derive(Error, Debug)]
pub struct RateAwareClientError<T>(T::Error)
where
    T: JsonRpcClient,
    <T as JsonRpcClient>::Error: Sync + Send + 'static;

impl<T> std::fmt::Display for RateAwareClientError<T>
where
    T: JsonRpcClient,
    <T as JsonRpcClient>::Error: Sync + Send + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T> From<RateAwareClientError<T>> for ProviderError
where
    T: JsonRpcClient + 'static,
    <T as JsonRpcClient>::Error: Sync + Send + 'static,
{
    fn from(src: RateAwareClientError<T>) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T> JsonRpcClient for RateAwareClient<T>
where
    T: JsonRpcClient + 'static,
    <T as JsonRpcClient>::Error: Sync + Send + 'static,
{
    type Error = RateAwareClientError<T>;

    /// Send out the request if we've not hit the rate limit else wait until the next
    /// cycle for quota renewal.
    async fn request<A: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: A,
    ) -> Result<R, Self::Error>
    where
        A: std::fmt::Debug + Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        loop {
            let requests_left = self.requests_left();
            if requests_left == 0 {
                let sleep_time = self.time_until_next_period();
                tokio::time::sleep(sleep_time).await;
                continue
            }

            let ret = self.client.request(method, params).await;
            if ret.is_err() {
                let err = ret.err().unwrap();
                return Err(RateAwareClientError(err))
            }
            self.record_request();
            return Ok(ret.unwrap())
        }
    }
}
