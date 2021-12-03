use std::fmt;

use crate::types::U256;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Chain {
    Mainnet,
    Ropsten,
    Rinkeby,
    Goerli,
    Kovan,
    XDai,
    Polygon,
    PolygonMumbai,
    Avalanche,
    AvalancheFuji,
    Sepolia,
    Moonbeam,
    MoonbeamDev,
    Moonriver,
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
            Chain::Polygon => 137,
            Chain::PolygonMumbai => 80001,
            Chain::Avalanche => 43114,
            Chain::AvalancheFuji => 43113,
            Chain::Sepolia => 11155111,
            Chain::Moonbeam => 1287,
            Chain::MoonbeamDev => 1281,
            Chain::Moonriver => 1285,
        }
    }
}

impl From<Chain> for U256 {
    fn from(chain: Chain) -> Self {
        u32::from(chain).into()
    }
}

impl From<Chain> for u64 {
    fn from(chain: Chain) -> Self {
        u32::from(chain).into()
    }
}
