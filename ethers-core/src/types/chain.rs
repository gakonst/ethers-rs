use serde::Deserialize;
use thiserror::Error;

use core::convert::TryFrom;
use std::{fmt, str::FromStr};

use crate::types::U256;

#[derive(Debug, Clone, Error)]
#[error("Failed to parse chain: {0}")]
pub struct ParseChainError(String);

#[repr(u64)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Chain {
    Mainnet = 1,
    Ropsten = 3,
    Rinkeby = 4,
    Goerli = 5,
    Kovan = 42,
    XDai = 100,
    Polygon = 137,
    Fantom = 250,
    FantomTestnet = 4002,
    PolygonMumbai = 80001,
    Avalanche = 43114,
    AvalancheFuji = 43113,
    Sepolia = 11155111,
    Moonbeam = 1287,
    MoonbeamDev = 1281,
    Moonriver = 1285,
    Optimism = 10,
    OptimismKovan = 69,
    BinanceSmartChain = 56,
    BinanceSmartChainTestnet = 97,
    Arbitrum = 42161,
    ArbitrumTestnet = 421611,
}

impl fmt::Display for Chain {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let chain = match self {
            Chain::Mainnet => "mainnet",
            Chain::Ropsten => "ropsten",
            Chain::Rinkeby => "rinkeby",
            Chain::Goerli => "goerli",
            Chain::Kovan => "kovan",
            Chain::XDai => "xdai",
            Chain::Polygon => "polygon",
            Chain::PolygonMumbai => "polygon-mumbai",
            Chain::Avalanche => "avalanche",
            Chain::AvalancheFuji => "avalanche-fuji",
            Chain::Sepolia => "sepolia",
            Chain::Moonbeam => "moonbeam",
            Chain::MoonbeamDev => "moonbeam-dev",
            Chain::Moonriver => "moonriver",
            Chain::Optimism => "optimism",
            Chain::OptimismKovan => "optimism-kovan",
            Chain::Fantom => "fantom",
            Chain::FantomTestnet => "fantom-testnet",
            Chain::BinanceSmartChain => "bsc",
            Chain::BinanceSmartChainTestnet => "bsc-testnet",
            Chain::Arbitrum => "arbitrum",
            Chain::ArbitrumTestnet => "arbitrum-testnet",
        };

        write!(formatter, "{}", chain)
    }
}

impl From<Chain> for u32 {
    fn from(chain: Chain) -> Self {
        chain as u32
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

impl TryFrom<u64> for Chain {
    type Error = ParseChainError;

    fn try_from(chain: u64) -> Result<Chain, Self::Error> {
        Ok(match chain {
            1 => Chain::Mainnet,
            3 => Chain::Ropsten,
            4 => Chain::Rinkeby,
            5 => Chain::Goerli,
            42 => Chain::Kovan,
            100 => Chain::XDai,
            137 => Chain::Polygon,
            80001 => Chain::PolygonMumbai,
            43114 => Chain::Avalanche,
            43113 => Chain::AvalancheFuji,
            11155111 => Chain::Sepolia,
            1287 => Chain::Moonbeam,
            1281 => Chain::MoonbeamDev,
            1285 => Chain::Moonriver,
            10 => Chain::Optimism,
            69 => Chain::OptimismKovan,
            56 => Chain::BinanceSmartChain,
            97 => Chain::BinanceSmartChainTestnet,
            42161 => Chain::Arbitrum,
            421611 => Chain::ArbitrumTestnet,
            _ => return Err(ParseChainError(chain.to_string())),
        })
    }
}

impl FromStr for Chain {
    type Err = ParseChainError;
    fn from_str(chain: &str) -> Result<Self, Self::Err> {
        Ok(match chain {
            "mainnet" => Chain::Mainnet,
            "ropsten" => Chain::Ropsten,
            "rinkeby" => Chain::Rinkeby,
            "goerli" => Chain::Goerli,
            "kovan" => Chain::Kovan,
            "xdai" => Chain::XDai,
            "polygon" => Chain::Polygon,
            "polygon-mumbai" => Chain::PolygonMumbai,
            "avalanche" => Chain::Avalanche,
            "avalanche-fuji" => Chain::AvalancheFuji,
            "sepolia" => Chain::Sepolia,
            "moonbeam" => Chain::Moonbeam,
            "moonbeam-dev" => Chain::MoonbeamDev,
            "moonriver" => Chain::Moonriver,
            "optimism" => Chain::Optimism,
            "optimism-kovan" => Chain::OptimismKovan,
            "fantom" => Chain::Fantom,
            "fantom-testnet" => Chain::FantomTestnet,
            "bsc" => Chain::BinanceSmartChain,
            "bsc-testnet" => Chain::BinanceSmartChainTestnet,
            "arbitrum" => Chain::Arbitrum,
            "arbitrum-testnet" => Chain::ArbitrumTestnet,
            _ => return Err(ParseChainError(chain.to_owned())),
        })
    }
}

impl Chain {
    /// Helper function for checking if a chainid corresponds to a legacy chainid
    /// without eip1559
    pub fn is_legacy(&self) -> bool {
        // TODO: Add other chains which do not support EIP1559.
        matches!(
            self,
            Chain::Optimism |
                Chain::OptimismKovan |
                Chain::Fantom |
                Chain::FantomTestnet |
                Chain::BinanceSmartChain |
                Chain::BinanceSmartChainTestnet |
                Chain::Arbitrum |
                Chain::ArbitrumTestnet,
        )
    }
}
