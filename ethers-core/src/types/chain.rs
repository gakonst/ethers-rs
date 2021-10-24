use std::fmt;

use crate::types::U256;

#[derive(Debug)]
pub enum Chain {
    Mainnet,
    Ropsten,
    Rinkeby,
    Goerli,
    Kovan,
    XDai,
}

impl fmt::Display for Chain {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

impl From<Chain> for u32 {
    fn from(chain: Chain) -> Self {
        match chain {
            Chain::Mainnet => 1,
            Chain::Ropsten => 3,
            Chain::Rinkeby => 4,
            Chain::Goerli => 5,
            Chain::Kovan => 42,
            Chain::XDai => 100,
        }
    }
}

impl From<Chain> for U256 {
    fn from(chain: Chain) -> Self {
        u32::from(chain).into()
    }
}
