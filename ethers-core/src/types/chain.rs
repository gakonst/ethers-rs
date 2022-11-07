use crate::types::U256;
use serde::Deserialize;
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    str::FromStr,
    time::Duration,
};
use strum::EnumVariantNames;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("Failed to parse chain: {0}")]
pub struct ParseChainError(String);

/// Enum for all known chains
///
/// When adding a new chain:
///   1. add new variant
///   2. update Display/FromStr impl
///   3. add etherscan_keys if supported
#[repr(u64)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Deserialize, EnumVariantNames)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "kebab-case")]
pub enum Chain {
    Mainnet = 1,
    Morden = 2,
    Ropsten = 3,
    Rinkeby = 4,
    Goerli = 5,
    Kovan = 42,
    #[strum(serialize = "gnosis")]
    XDai = 100,
    Chiado = 10200,
    Polygon = 137,
    Fantom = 250,
    Dev = 1337,
    AnvilHardhat = 31337,
    FantomTestnet = 4002,
    PolygonMumbai = 80001,
    Avalanche = 43114,
    AvalancheFuji = 43113,
    Sepolia = 11155111,
    Moonbeam = 1284,
    Moonbase = 1287,
    MoonbeamDev = 1281,
    Moonriver = 1285,
    Optimism = 10,
    OptimismGoerli = 420,
    OptimismKovan = 69,
    Arbitrum = 42161,
    ArbitrumTestnet = 421611,
    ArbitrumGoerli = 421613,
    Cronos = 25,
    CronosTestnet = 338,
    #[strum(serialize = "bsc")]
    BinanceSmartChain = 56,
    #[strum(serialize = "bsc-testnet")]
    BinanceSmartChainTestnet = 97,
    Poa = 99,
    Sokol = 77,
    Rsk = 30,
    Oasis = 26863,
    Emerald = 42262,
    EmeraldTestnet = 42261,
    Evmos = 9001,
    EvmosTestnet = 9000,
    Aurora = 1313161554,
    AuroraTestnet = 1313161555,
}

// === impl Chain ===

impl Chain {
    /// The blocktime varies from chain to chain
    ///
    /// It can be beneficial to know the average blocktime to adjust the polling of an Http provider
    /// for example.
    ///
    /// **Note:** this will not return the accurate average depending on the time but is rather a
    /// sensible default derived from blocktime charts like <https://etherscan.com/chart/blocktime>
    /// <https://polygonscan.com/chart/blocktime>
    pub fn average_blocktime_hint(&self) -> Option<Duration> {
        let ms = match self {
            Chain::Arbitrum | Chain::ArbitrumTestnet | Chain::ArbitrumGoerli => 1_300,
            Chain::Mainnet | Chain::Optimism => 13_000,
            Chain::Polygon | Chain::PolygonMumbai => 2_100,
            Chain::Moonbeam | Chain::Moonriver => 12_500,
            Chain::BinanceSmartChain | Chain::BinanceSmartChainTestnet => 3_000,
            Chain::Avalanche | Chain::AvalancheFuji => 2_000,
            Chain::Fantom | Chain::FantomTestnet => 1_200,
            Chain::Cronos | Chain::CronosTestnet => 5_700,
            Chain::Evmos | Chain::EvmosTestnet => 1_900,
            Chain::Aurora | Chain::AuroraTestnet => 1_100,
            Chain::Oasis => 5_500,
            Chain::Emerald => 6_000,
            Chain::Dev | Chain::AnvilHardhat => 200,
            // Explictly handle all network to make it easier not to forget this match when new
            // networks are added.
            Chain::Morden |
            Chain::Ropsten |
            Chain::Rinkeby |
            Chain::Goerli |
            Chain::Kovan |
            Chain::XDai |
            Chain::Chiado |
            Chain::Sepolia |
            Chain::Moonbase |
            Chain::MoonbeamDev |
            Chain::OptimismGoerli |
            Chain::OptimismKovan |
            Chain::Poa |
            Chain::Sokol |
            Chain::Rsk |
            Chain::EmeraldTestnet => return None,
        };

        Some(Duration::from_millis(ms))
    }

    /// Returns the corresponding etherscan URLs
    ///
    /// Returns `(API URL, BASE_URL)`, like `("https://api(-chain).etherscan.io/api", "https://etherscan.io")`
    pub fn etherscan_urls(&self) -> Option<(&'static str, &'static str)> {
        let urls = match self {
            Chain::Mainnet => ("https://api.etherscan.io/api", "https://etherscan.io"),
            Chain::Ropsten => {
                ("https://api-ropsten.etherscan.io/api", "https://ropsten.etherscan.io")
            }
            Chain::Kovan => ("https://api-kovan.etherscan.io/api", "https://kovan.etherscan.io"),
            Chain::Rinkeby => {
                ("https://api-rinkeby.etherscan.io/api", "https://rinkeby.etherscan.io")
            }
            Chain::Goerli => ("https://api-goerli.etherscan.io/api", "https://goerli.etherscan.io"),
            Chain::Sepolia => {
                ("https://api-sepolia.etherscan.io/api", "https://sepolia.etherscan.io")
            }
            Chain::Polygon => ("https://api.polygonscan.com/api", "https://polygonscan.com"),
            Chain::PolygonMumbai => {
                ("https://api-testnet.polygonscan.com/api", "https://mumbai.polygonscan.com")
            }
            Chain::Avalanche => ("https://api.snowtrace.io/api", "https://snowtrace.io"),
            Chain::AvalancheFuji => {
                ("https://api-testnet.snowtrace.io/api", "https://testnet.snowtrace.io")
            }
            Chain::Optimism => {
                ("https://api-optimistic.etherscan.io/api", "https://optimistic.etherscan.io")
            }
            Chain::OptimismGoerli => (
                "https://api-goerli-optimistic.etherscan.io/api",
                "https://goerli-optimism.etherscan.io",
            ),
            Chain::OptimismKovan => (
                "https://api-kovan-optimistic.etherscan.io/api",
                "https://kovan-optimistic.etherscan.io",
            ),
            Chain::Fantom => ("https://api.ftmscan.com/api", "https://ftmscan.com"),
            Chain::FantomTestnet => {
                ("https://api-testnet.ftmscan.com/api", "https://testnet.ftmscan.com")
            }
            Chain::BinanceSmartChain => ("https://api.bscscan.com/api", "https://bscscan.com"),
            Chain::BinanceSmartChainTestnet => {
                ("https://api-testnet.bscscan.com/api", "https://testnet.bscscan.com")
            }
            Chain::Arbitrum => ("https://api.arbiscan.io/api", "https://arbiscan.io"),
            Chain::ArbitrumTestnet => {
                ("https://api-testnet.arbiscan.io/api", "https://testnet.arbiscan.io")
            }
            Chain::ArbitrumGoerli => (
                "https://goerli-rollup-explorer.arbitrum.io/api",
                "https://goerli-rollup-explorer.arbitrum.io",
            ),
            Chain::Cronos => ("https://api.cronoscan.com/api", "https://cronoscan.com"),
            Chain::CronosTestnet => {
                ("https://api-testnet.cronoscan.com/api", "https://testnet.cronoscan.com")
            }
            Chain::Moonbeam => {
                ("https://api-moonbeam.moonscan.io/api", "https://moonbeam.moonscan.io/")
            }
            Chain::Moonbase => {
                ("https://api-moonbase.moonscan.io/api", "https://moonbase.moonscan.io/")
            }
            Chain::Moonriver => {
                ("https://api-moonriver.moonscan.io/api", "https://moonriver.moonscan.io")
            }
            // blockscout API is etherscan compatible
            Chain::XDai => {
                ("https://blockscout.com/xdai/mainnet/api", "https://blockscout.com/xdai/mainnet")
            }
            Chain::Chiado => {
                ("https://blockscout.chiadochain.net/api", "https://blockscout.chiadochain.net")
            }
            Chain::Sokol => {
                ("https://blockscout.com/poa/sokol/api", "https://blockscout.com/poa/sokol")
            }
            Chain::Poa => {
                ("https://blockscout.com/poa/core/api", "https://blockscout.com/poa/core")
            }
            Chain::Rsk => {
                ("https://blockscout.com/rsk/mainnet/api", "https://blockscout.com/rsk/mainnet")
            }
            Chain::Oasis => ("https://scan.oasischain.io/api", "https://scan.oasischain.io/"),
            Chain::Emerald => {
                ("https://explorer.emerald.oasis.dev/api", "https://explorer.emerald.oasis.dev/")
            }
            Chain::EmeraldTestnet => (
                "https://testnet.explorer.emerald.oasis.dev/api",
                "https://testnet.explorer.emerald.oasis.dev/",
            ),
            Chain::Aurora => ("https://api.aurorascan.dev/api", "https://aurorascan.dev"),
            Chain::AuroraTestnet => {
                ("https://testnet.aurorascan.dev/api", "https://testnet.aurorascan.dev")
            }
            Chain::Evmos => ("https://evm.evmos.org/api", "https://evm.evmos.org/"),
            Chain::EvmosTestnet => ("https://evm.evmos.dev/api", "https://evm.evmos.dev/"),
            Chain::AnvilHardhat | Chain::Dev | Chain::Morden | Chain::MoonbeamDev => {
                // this is explicitly exhaustive so we don't forget to add new urls when adding a
                // new chain
                return None
            }
        };

        Some(urls)
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let chain = match self {
            Chain::Mainnet => "mainnet",
            Chain::Morden => "morden",
            Chain::Ropsten => "ropsten",
            Chain::Rinkeby => "rinkeby",
            Chain::Goerli => "goerli",
            Chain::Kovan => "kovan",
            Chain::XDai => "gnosis",
            Chain::Chiado => "chiado",
            Chain::Polygon => "polygon",
            Chain::PolygonMumbai => "mumbai",
            Chain::Avalanche => "avalanche",
            Chain::AvalancheFuji => "fuji",
            Chain::Sepolia => "sepolia",
            Chain::Moonbeam => "moonbeam",
            Chain::Moonbase => "moonbase",
            Chain::MoonbeamDev => "moonbeam-dev",
            Chain::Moonriver => "moonriver",
            Chain::Optimism => "optimism",
            Chain::OptimismGoerli => "optimism-goerli",
            Chain::OptimismKovan => "optimism-kovan",
            Chain::Fantom => "fantom",
            Chain::Dev => "dev",
            Chain::FantomTestnet => "fantom-testnet",
            Chain::BinanceSmartChain => "bsc",
            Chain::BinanceSmartChainTestnet => "bsc-testnet",
            Chain::Arbitrum => "arbitrum",
            Chain::ArbitrumTestnet => "arbitrum-testnet",
            Chain::ArbitrumGoerli => "arbitrum-goerli",
            Chain::Cronos => "cronos",
            Chain::CronosTestnet => "cronos-testnet",
            Chain::Poa => "poa",
            Chain::Sokol => "sokol",
            Chain::Rsk => "rsk",
            Chain::Oasis => "oasis",
            Chain::Emerald => "emerald",
            Chain::EmeraldTestnet => "emerald-testnet",
            Chain::AnvilHardhat => "anvil-hardhat",
            Chain::Evmos => "evmos",
            Chain::EvmosTestnet => "evmos-testnet",
            Chain::Aurora => "aurora",
            Chain::AuroraTestnet => "aurora-testnet",
        };

        write!(formatter, "{chain}")
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
            2 => Chain::Morden,
            3 => Chain::Ropsten,
            4 => Chain::Rinkeby,
            5 => Chain::Goerli,
            42 => Chain::Kovan,
            100 => Chain::XDai,
            10200 => Chain::Chiado,
            137 => Chain::Polygon,
            1337 => Chain::Dev,
            31337 => Chain::AnvilHardhat,
            250 => Chain::Fantom,
            4002 => Chain::FantomTestnet,
            80001 => Chain::PolygonMumbai,
            43114 => Chain::Avalanche,
            43113 => Chain::AvalancheFuji,
            11155111 => Chain::Sepolia,
            1284 => Chain::Moonbeam,
            1287 => Chain::Moonbase,
            1281 => Chain::MoonbeamDev,
            1285 => Chain::Moonriver,
            10 => Chain::Optimism,
            420 => Chain::OptimismGoerli,
            69 => Chain::OptimismKovan,
            56 => Chain::BinanceSmartChain,
            97 => Chain::BinanceSmartChainTestnet,
            42161 => Chain::Arbitrum,
            421611 => Chain::ArbitrumTestnet,
            421613 => Chain::ArbitrumGoerli,
            25 => Chain::Cronos,
            338 => Chain::CronosTestnet,
            99 => Chain::Poa,
            77 => Chain::Sokol,
            30 => Chain::Rsk,
            26863 => Chain::Oasis,
            42262 => Chain::Emerald,
            42261 => Chain::EmeraldTestnet,
            9001 => Chain::Evmos,
            9000 => Chain::EvmosTestnet,
            1313161554 => Chain::Aurora,
            1313161555 => Chain::AuroraTestnet,
            _ => return Err(ParseChainError(chain.to_string())),
        })
    }
}

impl TryFrom<U256> for Chain {
    type Error = ParseChainError;

    fn try_from(chain: U256) -> Result<Chain, Self::Error> {
        if chain.bits() > 64 {
            return Err(ParseChainError(chain.to_string()))
        }
        chain.as_u64().try_into()
    }
}

impl FromStr for Chain {
    type Err = ParseChainError;
    fn from_str(chain: &str) -> Result<Self, Self::Err> {
        Ok(match chain {
            "mainnet" => Chain::Mainnet,
            "morden" => Chain::Morden,
            "ropsten" => Chain::Ropsten,
            "rinkeby" => Chain::Rinkeby,
            "goerli" => Chain::Goerli,
            "kovan" => Chain::Kovan,
            "xdai" | "gnosis" | "gnosis-chain" => Chain::XDai,
            "chiado" => Chain::Chiado,
            "polygon" => Chain::Polygon,
            "mumbai" | "polygon-mumbai" => Chain::PolygonMumbai,
            "avalanche" => Chain::Avalanche,
            "fuji" | "avalanche-fuji" => Chain::AvalancheFuji,
            "sepolia" => Chain::Sepolia,
            "moonbeam" => Chain::Moonbeam,
            "moonbase" => Chain::Moonbase,
            "moonbeam-dev" => Chain::MoonbeamDev,
            "moonriver" => Chain::Moonriver,
            "optimism" => Chain::Optimism,
            "optimism-goerli" => Chain::OptimismGoerli,
            "optimism-kovan" => Chain::OptimismKovan,
            "fantom" => Chain::Fantom,
            "fantom-testnet" => Chain::FantomTestnet,
            "dev" => Chain::Dev,
            "anvil" | "hardhat" | "anvil-hardhat" => Chain::AnvilHardhat,
            "bsc" => Chain::BinanceSmartChain,
            "bsc-testnet" => Chain::BinanceSmartChainTestnet,
            "arbitrum" => Chain::Arbitrum,
            "arbitrum-testnet" => Chain::ArbitrumTestnet,
            "arbitrum-goerli" => Chain::ArbitrumGoerli,
            "cronos" => Chain::Cronos,
            "cronos-testnet" => Chain::CronosTestnet,
            "poa" => Chain::Poa,
            "sokol" => Chain::Sokol,
            "rsk" => Chain::Rsk,
            "oasis" => Chain::Oasis,
            "emerald" => Chain::Emerald,
            "emerald-testnet" => Chain::EmeraldTestnet,
            "aurora" => Chain::Aurora,
            "aurora-testnet" => Chain::AuroraTestnet,
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
                Chain::OptimismGoerli |
                Chain::OptimismKovan |
                Chain::Fantom |
                Chain::FantomTestnet |
                Chain::BinanceSmartChain |
                Chain::BinanceSmartChainTestnet |
                Chain::Arbitrum |
                Chain::ArbitrumTestnet |
                Chain::ArbitrumGoerli |
                Chain::Rsk |
                Chain::Oasis |
                Chain::Emerald |
                Chain::EmeraldTestnet,
        )
    }
}

impl Default for Chain {
    fn default() -> Self {
        Chain::Mainnet
    }
}

#[test]
fn test_default_chain() {
    assert_eq!(Chain::default(), Chain::Mainnet);
}
