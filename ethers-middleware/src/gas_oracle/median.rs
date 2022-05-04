use crate::gas_oracle::{GasOracle, GasOracleError};
use async_trait::async_trait;
use ethers_core::types::U256;
use futures_util::future::join_all;
use std::{fmt::Debug, future::Future};
use tracing::warn;

// TODO: Weighted median

#[derive(Debug)]
pub struct Median<'a> {
    oracles: Vec<Box<dyn 'a + GasOracle>>,
}

/// Computes the median gas price from a selection of oracles.
///
/// Don't forget to set a timeout on the source oracles. By default
/// the reqwest based oracles will never time out.
impl<'a> Median<'a> {
    pub fn new(oracles: Vec<Box<dyn GasOracle>>) -> Self {
        Self { oracles }
    }

    pub fn add<T: 'a + GasOracle>(&mut self, oracle: T) {
        self.oracles.push(Box::new(oracle));
    }

    pub async fn query_all<Fn, Fut, O>(&'a self, mut f: Fn) -> Result<Vec<O>, GasOracleError>
    where
        Fn: FnMut(&'a dyn GasOracle) -> Fut,
        Fut: Future<Output = Result<O, GasOracleError>>,
    {
        // Process the oracles in parallel
        let futures = self.oracles.iter().map(|oracle| f(oracle.as_ref()));
        let results = join_all(futures).await;

        // Filter out any errors
        let values = self.oracles.iter().zip(results).filter_map(|(oracle, result)| match result {
            Ok(value) => Some(value),
            Err(err) => {
                warn!("Failed to fetch gas price from {:?}: {}", oracle, err);
                None
            }
        });
        let values = values.collect::<Vec<_>>();
        if values.is_empty() {
            return Err(GasOracleError::NoValues)
        }
        Ok(values)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl GasOracle for Median<'_> {
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        let mut values = self.query_all(|oracle| oracle.fetch()).await?;
        values.sort();
        Ok(values[values.len() / 2])
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256), GasOracleError> {
        let values = self.query_all(|oracle| oracle.estimate_eip1559_fees()).await?;
        let mut max_fee_per_gas = Vec::with_capacity(self.oracles.len());
        let mut max_priority_fee_per_gas = Vec::with_capacity(self.oracles.len());
        for (fee, priority) in values {
            max_fee_per_gas.push(fee);
            max_priority_fee_per_gas.push(priority);
        }
        assert_eq!(max_fee_per_gas.len(), max_priority_fee_per_gas.len());

        // Sort the values and return the median
        max_fee_per_gas.sort();
        max_priority_fee_per_gas.sort();
        Ok((
            max_fee_per_gas[max_fee_per_gas.len() / 2],
            max_priority_fee_per_gas[max_priority_fee_per_gas.len() / 2],
        ))
    }
}
