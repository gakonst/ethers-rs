use ethers_core::types::{Address, Chain};
use once_cell::sync::Lazy;
use serde::Deserialize;

use std::collections::HashMap;

const TOKENS_JSON: &str = include_str!("./tokens/tokens.json");

static TOKENS: Lazy<HashMap<String, Token>> =
    Lazy::new(|| serde_json::from_str(TOKENS_JSON).unwrap());

#[derive(Clone, Debug, Deserialize)]
pub struct Token {
    addresses: HashMap<Chain, Address>,
}

impl Token {
    pub fn address(&self, chain: Chain) -> Option<Address> {
        self.addresses.get(&chain).cloned()
    }
}

pub fn token<S: Into<String>>(symbol: S) -> Option<Token> {
    TOKENS.get(&symbol.into()).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens() {
        assert!(token("dai").is_some());
        assert!(token("usdc").is_some());
        assert!(token("rand").is_none());
    }

    #[test]
    fn test_addrs() {
        assert!(token("dai").unwrap().address(Chain::Mainnet).is_some());
        assert!(token("dai").unwrap().address(Chain::MoonbeamDev).is_none());
    }
}
