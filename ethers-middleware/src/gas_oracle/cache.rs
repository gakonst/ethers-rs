use super::{GasOracle, Result};
use async_trait::async_trait;
use ethers_core::types::U256;
use futures_locks::RwLock;
use instant::{Duration, Instant};
use std::{fmt::Debug, future::Future};

#[derive(Debug)]
pub struct Cache<T: GasOracle> {
    inner: T,
    validity: Duration,
    fee: Cached<U256>,
    eip1559: Cached<(U256, U256)>,
}

#[derive(Default, Debug)]
struct Cached<T: Clone>(RwLock<Option<(Instant, T)>>);

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T: GasOracle> GasOracle for Cache<T> {
    async fn fetch(&self) -> Result<U256> {
        self.fee.get(self.validity, || self.inner.fetch()).await
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256)> {
        self.eip1559.get(self.validity, || self.inner.estimate_eip1559_fees()).await
    }
}

impl<T: GasOracle> Cache<T> {
    pub fn new(validity: Duration, inner: T) -> Self {
        Self { inner, validity, fee: Cached::default(), eip1559: Cached::default() }
    }
}

impl<T: Clone> Cached<T> {
    async fn get<F, E, Fut>(&self, validity: Duration, fetch: F) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        // Try with a read lock
        {
            let lock = self.0.read().await;
            if let Some((last_fetch, value)) = lock.as_ref() {
                if Instant::now().duration_since(*last_fetch) < validity {
                    return Ok(value.clone())
                }
            }
        }
        // Acquire a write lock
        {
            let mut lock = self.0.write().await;
            // Check again, a concurrent thread may have raced us to the write.
            if let Some((last_fetch, value)) = lock.as_ref() {
                if Instant::now().duration_since(*last_fetch) < validity {
                    return Ok(value.clone())
                }
            }
            // Set a fresh value
            let value = fetch().await?;
            *lock = Some((Instant::now(), value.clone()));
            Ok(value)
        }
    }
}
