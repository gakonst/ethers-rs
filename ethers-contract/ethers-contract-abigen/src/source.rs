//! Module implements reading of contract artifacts from various sources.
use super::util;
use ethers_core::types::Address;

use anyhow::{anyhow, Context, Error, Result};
use std::{
    borrow::Cow,
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use url::Url;

/// A source of a Truffle artifact JSON.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Source {
    /// A raw ABI string
    String(String),

    /// An ABI located on the local file system.
    Local(PathBuf),

    /// An ABI to be retrieved over HTTP(S).
    Http(Url),

    /// An address of a mainnet contract that has been verified on Etherscan.io.
    Etherscan(Address),

    /// The package identifier of an npm package with a path to a Truffle
    /// artifact or ABI to be retrieved from `unpkg.io`.
    Npm(String),
}

impl Source {
    /// Parses an ABI from a source
    ///
    /// Contract ABIs can be retrieved from the local filesystem or online
    /// from `etherscan.io`. They can also be provided in-line. This method parses
    /// ABI source URLs and accepts the following:
    ///
    /// - raw ABI JSON
    ///
    /// - `relative/path/to/Contract.json`: a relative path to an ABI JSON file.
    /// This relative path is rooted in the current working directory.
    /// To specify the root for relative paths, use `Source::with_root`.
    ///
    /// - `/absolute/path/to/Contract.json` or
    ///   `file:///absolute/path/to/Contract.json`: an absolute path or file URL
    ///   to an ABI JSON file.
    ///
    /// - `http(s)://...` an HTTP url to a contract ABI.
    ///
    /// - `etherscan:0xXX..XX` or `https://etherscan.io/address/0xXX..XX`: a
    ///   address or URL of a verified contract on Etherscan.
    ///
    /// - `npm:@org/package@1.0.0/path/to/contract.json` an npmjs package with
    ///   an optional version and path (defaulting to the latest version and
    ///   `index.js`). The contract ABI will be retrieved through
    ///   `unpkg.io`.
    pub fn parse<S>(source: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let source = source.as_ref();
        if source.starts_with('[') {
            return Ok(Source::String(source.to_owned()));
        }
        let root = env::current_dir()?.canonicalize()?;
        Source::with_root(root, source)
    }

    /// Parses an artifact source from a string and a specified root directory
    /// for resolving relative paths. See `Source::with_root` for more details
    /// on supported source strings.
    fn with_root<P, S>(root: P, source: S) -> Result<Self>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let base = Url::from_directory_path(root)
            .map_err(|_| anyhow!("root path '{}' is not absolute"))?;
        let url = base.join(source.as_ref())?;

        match url.scheme() {
            "file" => Ok(Source::local(url.path())),
            "http" | "https" => match url.host_str() {
                Some("etherscan.io") => Source::etherscan(
                    url.path()
                        .rsplit('/')
                        .next()
                        .ok_or_else(|| anyhow!("HTTP URL does not have a path"))?,
                ),
                _ => Ok(Source::Http(url)),
            },
            "etherscan" => Source::etherscan(url.path()),
            "npm" => Ok(Source::npm(url.path())),
            _ => Err(anyhow!("unsupported URL '{}'", url)),
        }
    }

    /// Creates a local filesystem source from a path string.
    fn local<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Source::Local(path.as_ref().into())
    }

    /// Creates an HTTP source from a URL.
    fn http<S>(url: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Source::Http(Url::parse(url.as_ref())?))
    }

    /// Creates an Etherscan source from an address string.
    fn etherscan<S>(address: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let address =
            util::parse_address(address).context("failed to parse address for Etherscan source")?;
        Ok(Source::Etherscan(address))
    }

    /// Creates an Etherscan source from an address string.
    fn npm<S>(package_path: S) -> Self
    where
        S: Into<String>,
    {
        Source::Npm(package_path.into())
    }

    /// Retrieves the source JSON of the artifact this will either read the JSON
    /// from the file system or retrieve a contract ABI from the network
    /// dependending on the source type.
    pub fn get(&self) -> Result<String> {
        match self {
            Source::Local(path) => get_local_contract(path),
            Source::Http(url) => get_http_contract(url),
            Source::Etherscan(address) => get_etherscan_contract(*address),
            Source::Npm(package) => get_npm_contract(package),
            Source::String(abi) => Ok(abi.clone()),
        }
    }
}

impl FromStr for Source {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Source::parse(s)
    }
}

/// Reads a Truffle artifact JSON file from the local filesystem.
fn get_local_contract(path: &Path) -> Result<String> {
    let path = if path.is_relative() {
        let absolute_path = path.canonicalize().with_context(|| {
            format!(
                "unable to canonicalize file from working dir {} with path {}",
                env::current_dir()
                    .map(|cwd| cwd.display().to_string())
                    .unwrap_or_else(|err| format!("??? ({})", err)),
                path.display(),
            )
        })?;
        Cow::Owned(absolute_path)
    } else {
        Cow::Borrowed(path)
    };

    let json = fs::read_to_string(&path).context(format!(
        "failed to read artifact JSON file with path {}",
        &path.display()
    ))?;
    Ok(json)
}

/// Retrieves a Truffle artifact or ABI from an HTTP URL.
fn get_http_contract(url: &Url) -> Result<String> {
    let json = util::http_get(url.as_str())
        .with_context(|| format!("failed to retrieve JSON from {}", url))?;
    Ok(json)
}

/// Retrieves a contract ABI from the Etherscan HTTP API and wraps it in an
/// artifact JSON for compatibility with the code generation facilities.
fn get_etherscan_contract(address: Address) -> Result<String> {
    // NOTE: We do not retrieve the bytecode since deploying contracts with the
    //   same bytecode is unreliable as the libraries have already linked and
    //   probably don't reference anything when deploying on other networks.

    let api_key = env::var("ETHERSCAN_API_KEY")
        .map(|key| format!("&apikey={}", key))
        .unwrap_or_default();

    let abi_url = format!(
        "http://api.etherscan.io/api\
         ?module=contract&action=getabi&address={:?}&format=raw{}",
        address, api_key,
    );
    let abi = util::http_get(&abi_url).context("failed to retrieve ABI from Etherscan.io")?;
    Ok(abi)
}

/// Retrieves a Truffle artifact or ABI from an npm package through `unpkg.io`.
fn get_npm_contract(package: &str) -> Result<String> {
    let unpkg_url = format!("https://unpkg.io/{}", package);
    let json = util::http_get(&unpkg_url)
        .with_context(|| format!("failed to retrieve JSON from for npm package {}", package))?;

    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_source() {
        let root = "/rooted";
        for (url, expected) in &[
            (
                "relative/Contract.json",
                Source::local("/rooted/relative/Contract.json"),
            ),
            (
                "/absolute/Contract.json",
                Source::local("/absolute/Contract.json"),
            ),
            (
                "https://my.domain.eth/path/to/Contract.json",
                Source::http("https://my.domain.eth/path/to/Contract.json").unwrap(),
            ),
            (
                "etherscan:0x0001020304050607080910111213141516171819",
                Source::etherscan("0x0001020304050607080910111213141516171819").unwrap(),
            ),
            (
                "https://etherscan.io/address/0x0001020304050607080910111213141516171819",
                Source::etherscan("0x0001020304050607080910111213141516171819").unwrap(),
            ),
            (
                "npm:@openzeppelin/contracts@2.5.0/build/contracts/IERC20.json",
                Source::npm("@openzeppelin/contracts@2.5.0/build/contracts/IERC20.json"),
            ),
        ] {
            let source = Source::with_root(root, url).unwrap();
            assert_eq!(source, *expected);
        }

        let src = r#"[{"constant":true,"inputs":[],"name":"name","outputs":[{"name":"name","type":"string"}],"payable":false,"type":"function"},{"constant":true,"inputs":[],"name":"symbol","outputs":[{"name":"symbol","type":"string"}],"payable":false,"type":"function"},{"constant":true,"inputs":[],"name":"decimals","outputs":[{"name":"decimals","type":"uint8"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"spender","type":"address"},{"name":"value","type":"uint256"}],"name":"approve","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":true,"inputs":[],"name":"totalSupply","outputs":[{"name":"totalSupply","type":"uint256"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"from","type":"address"},{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transferFrom","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"who","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transfer","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"name":"allowance","outputs":[{"name":"remaining","type":"uint256"}],"payable":false,"type":"function"},{"anonymous":false,"inputs":[{"indexed":true,"name":"owner","type":"address"},{"indexed":true,"name":"spender","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Approval","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":true,"name":"to","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Transfer","type":"event"}]"#;
        let parsed = Source::parse(src).unwrap();
        assert_eq!(parsed, Source::String(src.to_owned()));
    }
}
