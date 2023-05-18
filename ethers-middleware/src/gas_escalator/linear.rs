use super::GasEscalator;
use ethers_core::types::U256;

/// Linearly increasing gas price.
///
///
/// Start with `initial_price`, then increase it by fixed amount `increase_by` every `every_secs`
/// seconds until the transaction gets confirmed. There is an optional upper limit.
///
/// <https://github.com/makerdao/pymaker/blob/master/pymaker/gas.py#L129>
#[derive(Clone, Debug)]
pub struct LinearGasPrice {
    every_secs: u64,
    increase_by: U256,
    max_price: Option<U256>,
}

impl LinearGasPrice {
    /// Constructor
    pub fn new<T: Into<U256>>(
        increase_by: T,
        every_secs: impl Into<u64>,
        max_price: Option<T>,
    ) -> Self {
        LinearGasPrice {
            every_secs: every_secs.into(),
            increase_by: increase_by.into(),
            max_price: max_price.map(Into::into),
        }
    }
}

impl GasEscalator for LinearGasPrice {
    fn get_gas_price(&self, initial_price: U256, time_elapsed: u64) -> U256 {
        let mut result = initial_price + self.increase_by * (time_elapsed / self.every_secs);
        if let Some(max_price) = self.max_price {
            result = std::cmp::min(result, max_price);
        }
        result
    }
}

#[cfg(test)]
// https://github.com/makerdao/pymaker/blob/master/tests/test_gas.py#L107
mod tests {
    use super::*;

    #[test]
    fn gas_price_increases_with_time() {
        let oracle = LinearGasPrice::new(100, 60u64, None);
        let initial_price = U256::from(1000);

        assert_eq!(oracle.get_gas_price(initial_price, 0), 1000.into());
        assert_eq!(oracle.get_gas_price(initial_price, 1), 1000.into());
        assert_eq!(oracle.get_gas_price(initial_price, 59), 1000.into());
        assert_eq!(oracle.get_gas_price(initial_price, 60), 1100.into());
        assert_eq!(oracle.get_gas_price(initial_price, 119), 1100.into());
        assert_eq!(oracle.get_gas_price(initial_price, 120), 1200.into());
        assert_eq!(oracle.get_gas_price(initial_price, 1200), 3000.into());
    }

    #[test]
    fn gas_price_should_obey_max_value() {
        let oracle = LinearGasPrice::new(100, 60u64, Some(2500));
        let initial_price = U256::from(1000);

        assert_eq!(oracle.get_gas_price(initial_price, 0), 1000.into());
        assert_eq!(oracle.get_gas_price(initial_price, 1), 1000.into());
        assert_eq!(oracle.get_gas_price(initial_price, 59), 1000.into());
        assert_eq!(oracle.get_gas_price(initial_price, 60), 1100.into());
        assert_eq!(oracle.get_gas_price(initial_price, 119), 1100.into());
        assert_eq!(oracle.get_gas_price(initial_price, 120), 1200.into());
        assert_eq!(oracle.get_gas_price(initial_price, 1200), 2500.into());
        assert_eq!(oracle.get_gas_price(initial_price, 3000), 2500.into());
        assert_eq!(oracle.get_gas_price(initial_price, 1000000), 2500.into());
    }
}
