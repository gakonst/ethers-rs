use ethers_contract::{abigen, Lazy};
use ethers_core::types::{Address, U256};
use std::collections::HashMap;

/// A lazily computed hash map with the Ethereum network IDs as keys and the corresponding
/// DsProxyFactory contract addresses as values
pub static ADDRESS_BOOK: Lazy<HashMap<U256, Address>> = Lazy::new(|| {
    HashMap::from([
        // Mainnet
        (U256::from(1_u64), "eefba1e63905ef1d7acba5a8513c70307c1ce441".parse().unwrap()),
    ])
});

abigen!(
    DsProxyFactory,
    "./contracts/DSProxyFactory.json",
    methods {
        build() as build_with_sender;
    }
);
