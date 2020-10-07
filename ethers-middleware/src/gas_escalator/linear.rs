use super::GasEscalator;
use ethers_core::types::U256;

/// Linearly increasing gas price.
///
///
/// Start with `initial_price`, then increase it by fixed amount `increase_by` every `every_secs` seconds
/// until the transaction gets confirmed. There is an optional upper limit.
/// https://github.com/makerdao/pymaker/blob/master/pymaker/gas.py#L129
#[derive(Clone, Debug)]
pub struct LinearGasPrice {
    pub every_secs: u64,
    pub increase_by: U256,
    pub max_price: Option<U256>,
}

impl Default for LinearGasPrice {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearGasPrice {
    /// Constructor
    pub fn new() -> Self {
        LinearGasPrice {
            every_secs: 30,
            increase_by: U256::from(0),
            max_price: None,
        }
    }
}

impl GasEscalator for LinearGasPrice {
    fn get_gas_price(&self, initial_price: U256, time_elapsed: u64) -> U256 {
        let mut result = initial_price + self.increase_by * (time_elapsed / self.every_secs) as u64;
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
        let mut oracle = LinearGasPrice::new();
        oracle.increase_by = U256::from(100);
        oracle.every_secs = 60;
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
        let mut oracle = LinearGasPrice::new();
        oracle.increase_by = U256::from(100);
        oracle.every_secs = 60;
        oracle.max_price = Some(2500.into());
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
