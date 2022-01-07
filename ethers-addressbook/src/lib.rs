use ethers_core::types::{Address, Chain};
use once_cell::sync::Lazy;
use serde::Deserialize;

use std::collections::HashMap;

const CONTRACTS_JSON: &str = include_str!("./contracts/contracts.json");

static ADDRESSBOOK: Lazy<HashMap<String, Contract>> =
    Lazy::new(|| serde_json::from_str(CONTRACTS_JSON).unwrap());

#[derive(Clone, Debug, Deserialize)]
pub struct Contract {
    addresses: HashMap<Chain, Address>,
}

impl Contract {
    pub fn address(&self, chain: Chain) -> Option<Address> {
        self.addresses.get(&chain).cloned()
    }
}

pub fn contract<S: Into<String>>(name: S) -> Option<Contract> {
    ADDRESSBOOK.get(&name.into()).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens() {
        assert!(contract("dai").is_some());
        assert!(contract("usdc").is_some());
        assert!(contract("rand").is_none());
    }

    #[test]
    fn test_addrs() {
        assert!(contract("dai").unwrap().address(Chain::Mainnet).is_some());
        assert!(contract("dai").unwrap().address(Chain::MoonbeamDev).is_none());
    }
}
