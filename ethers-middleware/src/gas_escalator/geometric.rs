use super::GasEscalator;
use ethers_core::types::U256;

/// Geometrically increasing gas price.
///
/// Start with `initial_price`, then increase it every 'every_secs' seconds by a fixed coefficient.
/// Coefficient defaults to 1.125 (12.5%), the minimum increase for Parity to replace a transaction.
/// Coefficient can be adjusted, and there is an optional upper limit.
///
/// <https://github.com/makerdao/pymaker/blob/master/pymaker/gas.py#L168>
#[derive(Clone, Debug)]
pub struct GeometricGasPrice {
    every_secs: u64,
    coefficient: f64,
    max_price: Option<U256>,
}

impl GeometricGasPrice {
    /// Constructor
    ///
    /// Note: Providing `None` to `max_price` requires giving it a type-hint, so you'll need
    /// to call this like `GeometricGasPrice::new(1.125, 60u64, None::<u64>)`.
    pub fn new<T: Into<U256>, K: Into<u64>>(
        coefficient: f64,
        every_secs: K,
        max_price: Option<T>,
    ) -> Self {
        GeometricGasPrice {
            every_secs: every_secs.into(),
            coefficient,
            max_price: max_price.map(Into::into),
        }
    }
}

impl GasEscalator for GeometricGasPrice {
    fn get_gas_price(&self, initial_price: U256, time_elapsed: u64) -> U256 {
        let mut result = initial_price.as_u64() as f64;

        if time_elapsed >= self.every_secs {
            let iters = time_elapsed / self.every_secs;
            for _ in 0..iters {
                result *= self.coefficient;
            }
        }

        let mut result = U256::from(result.ceil() as u64);
        if let Some(max_price) = self.max_price {
            result = std::cmp::min(result, max_price);
        }
        result
    }
}

#[cfg(test)]
// https://github.com/makerdao/pymaker/blob/master/tests/test_gas.py#L165
mod tests {
    use super::*;

    #[test]
    fn gas_price_increases_with_time() {
        let oracle = GeometricGasPrice::new(1.125, 10u64, None::<u64>);
        let initial_price = U256::from(100);

        assert_eq!(oracle.get_gas_price(initial_price, 0), 100.into());
        assert_eq!(oracle.get_gas_price(initial_price, 1), 100.into());
        assert_eq!(oracle.get_gas_price(initial_price, 10), 113.into());
        assert_eq!(oracle.get_gas_price(initial_price, 15), 113.into());
        assert_eq!(oracle.get_gas_price(initial_price, 20), 127.into());
        assert_eq!(oracle.get_gas_price(initial_price, 30), 143.into());
        assert_eq!(oracle.get_gas_price(initial_price, 50), 181.into());
        assert_eq!(oracle.get_gas_price(initial_price, 100), 325.into());
    }

    #[test]
    fn gas_price_should_obey_max_value() {
        let oracle = GeometricGasPrice::new(1.125, 60u64, Some(2500));
        let initial_price = U256::from(1000);

        assert_eq!(oracle.get_gas_price(initial_price, 0), 1000.into());
        assert_eq!(oracle.get_gas_price(initial_price, 1), 1000.into());
        assert_eq!(oracle.get_gas_price(initial_price, 59), 1000.into());
        assert_eq!(oracle.get_gas_price(initial_price, 60), 1125.into());
        assert_eq!(oracle.get_gas_price(initial_price, 119), 1125.into());
        assert_eq!(oracle.get_gas_price(initial_price, 120), 1266.into());
        assert_eq!(oracle.get_gas_price(initial_price, 1200), 2500.into());
        assert_eq!(oracle.get_gas_price(initial_price, 3000), 2500.into());
        assert_eq!(oracle.get_gas_price(initial_price, 1000000), 2500.into());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn behaves_with_realistic_values() {
        let oracle = GeometricGasPrice::new(1.25, 10u64, None::<u64>);
        const GWEI: f64 = 1000000000.0;
        let initial_price = U256::from(100 * GWEI as u64);

        for seconds in &[0u64, 1, 10, 12, 30, 60] {
            println!(
                "gas price after {} seconds is {}",
                seconds,
                oracle.get_gas_price(initial_price, *seconds).as_u64() as f64 / GWEI
            );
        }

        let normalized = |time| oracle.get_gas_price(initial_price, time).as_u64() as f64 / GWEI;

        assert_eq!(normalized(0), 100.0);
        assert_eq!(normalized(1), 100.0);
        assert_eq!(normalized(10), 125.0);
        assert_eq!(normalized(12), 125.0);
        assert_eq!(normalized(30), 195.3125);
        assert_eq!(normalized(60), 381.469726563);
    }
}
