use super::Source;
use crate::util;
use ethers_core::types::{Address, Chain};
use ethers_etherscan::Client;
use eyre::{Context, Result};
use std::{fmt, str::FromStr};
use url::Url;

/// An [etherscan](https://etherscan.io)-like blockchain explorer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Explorer {
    /// <https://etherscan.io>
    #[default]
    Etherscan,
    /// <https://bscscan.com>
    Bscscan,
    /// <https://polygonscan.com>
    Polygonscan,
    /// <https://snowtrace.io>
    Snowtrace,
}

impl FromStr for Explorer {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "etherscan" | "etherscan.io" => Ok(Self::Etherscan),
            "bscscan" | "bscscan.com" => Ok(Self::Bscscan),
            "polygonscan" | "polygonscan.com" => Ok(Self::Polygonscan),
            "snowtrace" | "snowtrace.io" => Ok(Self::Snowtrace),
            _ => Err(eyre::eyre!("Invalid or unsupported blockchain explorer: {s}")),
        }
    }
}

impl fmt::Display for Explorer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Explorer {
    /// Returns the chain's Explorer, if it is known.
    pub fn from_chain(chain: Chain) -> Result<Self> {
        match chain {
            Chain::Mainnet => Ok(Self::Etherscan),
            Chain::BinanceSmartChain => Ok(Self::Bscscan),
            Chain::Polygon => Ok(Self::Polygonscan),
            Chain::Avalanche => Ok(Self::Snowtrace),
            _ => Err(eyre::eyre!("Provided chain has no known blockchain explorer")),
        }
    }

    /// Returns the Explorer's chain. If it has multiple, the main one is returned.
    pub const fn chain(&self) -> Chain {
        match self {
            Self::Etherscan => Chain::Mainnet,
            Self::Bscscan => Chain::BinanceSmartChain,
            Self::Polygonscan => Chain::Polygon,
            Self::Snowtrace => Chain::Avalanche,
        }
    }

    /// Creates an `ethers-etherscan` client using this Explorer's settings.
    pub fn client(self, api_key: Option<String>) -> Result<Client> {
        let chain = self.chain();
        let client = match api_key {
            Some(api_key) => Client::new(chain, api_key),
            None => Client::new_from_opt_env(chain),
        }?;
        Ok(client)
    }

    /// Retrieves a contract ABI from the Etherscan HTTP API and wraps it in an artifact JSON for
    /// compatibility with the code generation facilities.
    pub fn get(self, address: Address) -> Result<String> {
        // TODO: Improve this
        let client = self.client(None)?;
        let future = client.contract_abi(address);
        let abi = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle.block_on(future),
            _ => tokio::runtime::Runtime::new().expect("Could not start runtime").block_on(future),
        }?;
        Ok(serde_json::to_string(&abi)?)
    }
}

impl Source {
    #[inline]
    pub(super) fn parse_online(source: &str) -> Result<Self> {
        if let Ok(url) = Url::parse(source) {
            match url.scheme() {
                // file://<path>
                "file" => Self::local(source),

                // npm:<npm package>
                "npm" => Ok(Self::npm(url.path())),

                // try first: <explorer url>/.../<address>
                // then: any http url
                "http" | "https" => Ok(url
                    .host_str()
                    .and_then(|host| Self::from_explorer(host, &url).ok())
                    .unwrap_or(Self::Http(url))),

                // custom scheme: <explorer or chain>:<address>
                // fallback: local fs path
                scheme => Self::from_explorer(scheme, &url)
                    .or_else(|_| Self::local(source))
                    .wrap_err("Invalid path or URL"),
            }
        } else {
            // not a valid URL so fallback to path
            Self::local(source)
        }
    }

    /// Parse `s` as an explorer ("etherscan"), explorer domain ("etherscan.io") or a chain that has
    /// an explorer ("mainnet").
    ///
    /// The URL can be either `<explorer>:<address>` or `<explorer_url>/.../<address>`
    fn from_explorer(s: &str, url: &Url) -> Result<Self> {
        let explorer: Explorer = s.parse().or_else(|_| Explorer::from_chain(s.parse()?))?;
        let address = last_segment_address(url).ok_or_else(|| eyre::eyre!("Invalid URL: {url}"))?;
        Ok(Self::Explorer(explorer, address))
    }

    /// Creates an HTTP source from a URL.
    pub fn http(url: impl AsRef<str>) -> Result<Self> {
        Ok(Self::Http(Url::parse(url.as_ref())?))
    }

    /// Creates an Etherscan source from an address string.
    pub fn explorer(chain: Chain, address: Address) -> Result<Self> {
        let explorer = Explorer::from_chain(chain)?;
        Ok(Self::Explorer(explorer, address))
    }

    /// Creates an Etherscan source from an address string.
    pub fn npm(package_path: impl Into<String>) -> Self {
        Self::Npm(package_path.into())
    }

    #[inline]
    pub(super) fn get_online(&self) -> Result<String> {
        match self {
            Self::Http(url) => {
                util::http_get(url.clone()).wrap_err("Failed to retrieve ABI from URL")
            }
            Self::Explorer(explorer, address) => explorer.get(*address),
            Self::Npm(package) => {
                // TODO: const?
                let unpkg = Url::parse("https://unpkg.io/").unwrap();
                let url = unpkg.join(package).wrap_err("Invalid NPM package")?;
                util::http_get(url).wrap_err("Failed to retrieve ABI from NPM package")
            }
            _ => unreachable!(),
        }
    }
}

fn last_segment_address(url: &Url) -> Option<Address> {
    url.path().rsplit('/').next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_online_source() {
        assert_eq!(
            Source::parse("https://my.domain.eth/path/to/Contract.json").unwrap(),
            Source::http("https://my.domain.eth/path/to/Contract.json").unwrap()
        );

        assert_eq!(
            Source::parse("npm:@openzeppelin/contracts@2.5.0/build/contracts/IERC20.json").unwrap(),
            Source::npm("@openzeppelin/contracts@2.5.0/build/contracts/IERC20.json")
        );

        let explorers = &[
            ("mainnet:", "etherscan:", "https://etherscan.io/address/", Chain::Mainnet),
            ("bsc:", "bscscan:", "https://bscscan.com/address/", Chain::BinanceSmartChain),
            ("polygon:", "polygonscan:", "https://polygonscan.com/address/", Chain::Polygon),
            ("avalanche:", "snowtrace:", "https://snowtrace.io/address/", Chain::Avalanche),
        ];

        let address: Address = "0x0102030405060708091011121314151617181920".parse().unwrap();
        for &(chain_s, scan_s, url_s, chain) in explorers {
            let expected = Source::explorer(chain, address).unwrap();

            let tests2 = [chain_s, scan_s, url_s].map(|s| s.to_string() + &format!("{address:?}"));
            let tests2 = tests2.map(Source::parse).into_iter().chain(Some(Ok(expected.clone())));
            let tests2 = tests2.collect::<Result<Vec<_>>>().unwrap();

            for slice in tests2.windows(2) {
                let [a, b] = slice else { unreachable!() };
                if a != b {
                    panic!("Expected: {expected:?}; Got: {a:?} | {b:?}");
                }
            }
        }
    }

    #[test]
    fn get_mainnet_contract() {
        // Skip if ETHERSCAN_API_KEY is not set
        if std::env::var("ETHERSCAN_API_KEY").is_err() {
            return
        }

        let source = Source::parse("mainnet:0x6b175474e89094c44da98b954eedeac495271d0f").unwrap();
        let abi = source.get().unwrap();
        assert!(!abi.is_empty());
    }
}
