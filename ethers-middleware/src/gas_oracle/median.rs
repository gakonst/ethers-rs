use super::{GasOracle, GasOracleError, Result};
use async_trait::async_trait;
use ethers_core::types::U256;
use futures_util::future::join_all;
use std::{fmt::Debug, future::Future};
use tracing::warn;

#[derive(Default, Debug)]
pub struct Median {
    oracles: Vec<(f32, Box<dyn GasOracle>)>,
}

/// Computes the median gas price from a selection of oracles.
///
/// Don't forget to set a timeout on the source oracles. By default
/// the reqwest based oracles will never time out.
impl Median {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<T: 'static + GasOracle>(&mut self, oracle: T) {
        self.add_weighted(1.0, oracle)
    }

    pub fn add_weighted<T: 'static + GasOracle>(&mut self, weight: f32, oracle: T) {
        assert!(weight > 0.0);
        self.oracles.push((weight, Box::new(oracle)));
    }

    pub async fn query_all<'a, Fn, Fut, O>(&'a self, mut f: Fn) -> Result<Vec<(f32, O)>>
    where
        Fn: FnMut(&'a dyn GasOracle) -> Fut,
        Fut: Future<Output = Result<O>>,
    {
        // Process the oracles in parallel
        let futures = self.oracles.iter().map(|(_, oracle)| f(oracle.as_ref()));
        let results = join_all(futures).await;

        // Filter out any errors
        let values =
            self.oracles.iter().zip(results).filter_map(
                |((weight, oracle), result)| match result {
                    Ok(value) => Some((*weight, value)),
                    Err(err) => {
                        warn!("Failed to fetch gas price from {:?}: {}", oracle, err);
                        None
                    }
                },
            );
        let values = values.collect::<Vec<_>>();
        if values.is_empty() {
            return Err(GasOracleError::NoValues)
        }
        Ok(values)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl GasOracle for Median {
    async fn fetch(&self) -> Result<U256> {
        let mut values = self.query_all(|oracle| oracle.fetch()).await?;
        // `query_all` guarantees `values` is not empty
        Ok(*weighted_fractile_by_key(0.5, &mut values, |fee| fee).unwrap())
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256)> {
        let mut values = self.query_all(|oracle| oracle.estimate_eip1559_fees()).await?;
        // `query_all` guarantees `values` is not empty
        Ok((
            weighted_fractile_by_key(0.5, &mut values, |(max_fee, _)| max_fee).unwrap().0,
            weighted_fractile_by_key(0.5, &mut values, |(_, priority_fee)| priority_fee).unwrap().1,
        ))
    }
}

/// Weighted fractile by key.
///
/// Sort the values in place by key and return the weighted fractile value such that `fractile`
/// fraction of the values by weight are less than or equal to the value.
///
/// Returns `None` if the values are empty.
///
/// Note: it doesn't handle NaNs or other special float values.
///
/// See: <https://en.wikipedia.org/wiki/Percentile#The_weighted_percentile_method>
///
/// # Panics
///
/// Panics if `fractile` is not in the range `0.0..=1.0`.
fn weighted_fractile_by_key<'a, T, F, K>(
    fractile: f32,
    values: &'a mut [(f32, T)],
    mut key: F,
) -> Option<&'a T>
where
    F: for<'b> FnMut(&'b T) -> &'b K,
    K: Ord,
{
    assert!((0.0..=1.0).contains(&fractile));
    if values.is_empty() {
        return None
    }
    let weight_rank = fractile * values.iter().map(|(weight, _)| *weight).sum::<f32>();
    values.sort_unstable_by(|a, b| key(&a.1).cmp(key(&b.1)));
    let mut cumulative_weight = 0.0_f32;
    for (weight, value) in values.iter() {
        cumulative_weight += *weight;
        if cumulative_weight >= weight_rank {
            return Some(value)
        }
    }
    // By the last element, cumulative_weight == weight_rank and we should have
    // returned already. Assume there is a slight rounding error causing
    // cumulative_weight to be slightly less than expected. In this case the last
    // element is appropriate. (This is not exactly right, since the last
    // elements may have zero weight.)
    // `values` is not empty.
    Some(&values.last().unwrap().1)
}
