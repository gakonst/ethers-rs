use ethers_core::types::{Address, Chain};

use std::{collections::HashMap, str::FromStr};

#[derive(Debug)]
pub struct Token {
    symbol: String,
    addresses: HashMap<Chain, Address>,
}

impl Token {
    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }

    pub fn address(&self, chain: Chain) -> Option<Address> {
        self.addresses.get(&chain).cloned()
    }
}

macro_rules! declare_token {
    ( $token: ident, $( $chain_id: expr => $addr: expr ),* ) => {
        pub fn $token() -> Token {
            let mut addresses = HashMap::new();
            $(
                addresses.insert(
                    Chain::from_str($chain_id).expect("invalid chain"),
                    Address::from_str($addr).expect("invalid address"),
                );
            )*

            Token {
                symbol: stringify!($token).to_string(),
                addresses,
            }
        }
    }
}

declare_token!(dai, "mainnet" => "0x6b175474e89094c44da98b954eedeac495271d0f", "rinkeby" => "0x8ad3aa5d5ff084307d28c8f514d7a193b2bfe725");
declare_token!(usdc, "mainnet" => "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dai() {
        let dai_token = dai();
        assert_eq!(dai_token.symbol(), "dai".to_string());
        assert_eq!(
            dai_token.address(Chain::Mainnet),
            Some(Address::from_str("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap())
        );
        assert_eq!(
            dai_token.address(Chain::Rinkeby),
            Some(Address::from_str("0x8ad3aa5d5ff084307d28c8f514d7a193b2bfe725").unwrap())
        );
        assert_eq!(dai_token.address(Chain::Goerli), None);
    }
}
