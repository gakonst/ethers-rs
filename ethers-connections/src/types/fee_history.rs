use ethers_core::types::{U256, U64};

pub struct FeeHistory {
    pub oldest_block: U64,
    pub rewards: Vec<Vec<U256>>,
    pub base_fee_per_gas: Vec<U256>,
    pub gas_used_ratio: Vec<f64>,
}
