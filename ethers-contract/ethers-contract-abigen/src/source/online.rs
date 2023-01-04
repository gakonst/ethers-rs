use super::Source;
use crate::util;
use ethers_core::types::{Address, Chain};
use ethers_etherscan::Client;
use eyre::{Context, Result};
use std::{fmt, str::FromStr};
use url::Url;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Explorer {
    #[default]
    Etherscan,
    Bscscan,
    Polygonscan,
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
    pub fn from_chain(chain: Chain) -> Option<Self> {
        match chain {
            Chain::Mainnet => Some(Self::Etherscan),
            Chain::BinanceSmartChain => Some(Self::Bscscan),
            Chain::Polygon => Some(Self::Polygonscan),
            Chain::Avalanche => Some(Self::Snowtrace),
            _ => None,
        }
    }

    pub const fn chain(&self) -> Chain {
        match self {
            Self::Etherscan => Chain::Mainnet,
            Self::Bscscan => Chain::BinanceSmartChain,
            Self::Polygonscan => Chain::Polygon,
            Self::Snowtrace => Chain::Avalanche,
        }
    }

    pub fn client(self, api_key: Option<String>) -> Result<Client> {
        let chain = self.chain();
        let client = match api_key {
            Some(api_key) => Client::new(chain, api_key),
            None => Client::new_from_env(chain),
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
                "file" => Self::local(source),
                "http" | "https" => {
                    if let Some(host) = url.host_str() {
                        Self::_explorer(host, &url)
                    } else {
                        Ok(Self::Http(url))
                    }
                }
                "npm" => Ok(Self::npm(source)),
                scheme => Self::_explorer(scheme, &url)
                    .or_else(|_| Self::local(source))
                    .wrap_err("Invalid path or URL"),
            }
        } else {
            Self::local(source)
        }
    }

    fn _explorer(explorer: &str, url: &Url) -> Result<Self> {
        let explorer: Explorer = explorer.parse()?;
        let address = url
            .as_str()
            .rsplit('/')
            .next()
            .ok_or_else(|| eyre::eyre!("Invalid URL: {url}"))?
            .parse()
            .wrap_err("Invalid address")?;
        Ok(Self::Explorer(explorer, address))
    }

    /// Creates an HTTP source from a URL.
    pub fn http(url: impl AsRef<str>) -> Result<Self> {
        Ok(Self::Http(Url::parse(url.as_ref())?))
    }

    /// Creates an Etherscan source from an address string.
    pub fn explorer(chain: Chain, address: Address) -> Result<Self> {
        let explorer = Explorer::from_chain(chain)
            .ok_or_else(|| eyre::eyre!("Provided chain has no known blockchain explorer"))?;
        Ok(Self::Explorer(explorer, address))
    }

    /// Creates an Etherscan source from an address string.
    pub fn npm(package_path: impl Into<String>) -> Self {
        Self::Npm(package_path.into())
    }

    #[inline]
    pub(super) fn get_online(&self) -> Result<String> {
        match self {
            Self::Http(url) => get_http_contract(url),
            Self::Explorer(explorer, address) => explorer.get(*address),
            Self::Npm(package) => get_npm_contract(package),
            _ => unreachable!(),
        }
    }
}

/// Retrieves a Truffle artifact or ABI from an HTTP URL.
fn get_http_contract(url: &Url) -> Result<String> {
    let json = util::http_get(url.as_str())
        .wrap_err_with(|| format!("failed to retrieve JSON from {url}"))?;
    Ok(json)
}

/// Retrieves a Truffle artifact or ABI from an npm package through `unpkg.io`.
fn get_npm_contract(package: &str) -> Result<String> {
    let unpkg_url = format!("https://unpkg.io/{package}");
    let json = util::http_get(&unpkg_url)
        .wrap_err_with(|| format!("failed to retrieve JSON from for npm package {package}"))?;

    Ok(json)
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
            (
                "mainnet:0x0102030405060708091011121314151617181920",
                "etherscan:0x0102030405060708091011121314151617181920",
                "https://etherscan.io/address/0x0102030405060708091011121314151617181920",
                Chain::Mainnet,
            ),
            (
                "bsc:0x0102030405060708091011121314151617181920",
                "bscscan:0x0102030405060708091011121314151617181920",
                "https://bscscan.com/address/0x0102030405060708091011121314151617181920",
                Chain::BinanceSmartChain,
            ),
            (
                "polygon:0x0102030405060708091011121314151617181920",
                "polygonscan:0x0102030405060708091011121314151617181920",
                "https://polygonscan.com/address/0x0102030405060708091011121314151617181920",
                Chain::Polygon,
            ),
            (
                "avalanche:0x0102030405060708091011121314151617181920",
                "snowtrace:0x0102030405060708091011121314151617181920",
                "https://snowtrace.io/address/0x0102030405060708091011121314151617181920",
                Chain::Avalanche,
            ),
        ];

        let address: Address = "0x0102030405060708091011121314151617181920".parse().unwrap();
        for &(chain_s, scan_s, url_s, chain) in explorers {
            let a = Source::parse(chain_s).unwrap();
            let b = Source::parse(scan_s).unwrap();
            let c = Source::parse(url_s).unwrap();
            let expected = Source::explorer(chain, address).unwrap();
            assert!(
                a == b && b == c && c == expected,
                "Expected: {expected:?}; Got: {chain_s} => {a:?}, {scan_s} => {b:?}, {url_s} => {c:?}"
            );
        }
    }
}
