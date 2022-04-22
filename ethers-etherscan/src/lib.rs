//! Bindings for [etherscan.io web api](https://docs.etherscan.io/)

use std::{
    borrow::Cow,
    io::Write,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use contract::ContractMetadata;
use reqwest::{header, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::trace;

use errors::EtherscanError;
use ethers_core::{
    abi::{Abi, Address},
    types::{Chain, H256},
};

pub mod account;
pub mod contract;
pub mod errors;
pub mod gas;
pub mod source_tree;
pub mod transaction;

pub(crate) type Result<T> = std::result::Result<T, EtherscanError>;

/// The Etherscan.io API client.
#[derive(Clone, Debug)]
pub struct Client {
    /// Client that executes HTTP requests
    client: reqwest::Client,
    /// Etherscan API key
    api_key: String,
    /// Etherscan API endpoint like <https://api(-chain).etherscan.io/api>
    etherscan_api_url: Url,
    /// Etherscan base endpoint like <https://etherscan.io>
    etherscan_url: Url,
    /// Path to where ABI files should be cached
    cache: Option<Cache>,
}

/// A wrapper around an Etherscan cache object with an expiry
#[derive(Clone, Debug, Deserialize, Serialize)]
struct CacheEnvelope<T> {
    expiry: u64,
    data: T,
}

/// Simple cache for etherscan requests
#[derive(Clone, Debug)]
struct Cache {
    root: PathBuf,
    ttl: Duration,
}

impl Cache {
    fn new(root: PathBuf, ttl: Duration) -> Self {
        Self { root, ttl }
    }

    fn get_abi(&self, address: Address) -> Option<Option<ethers_core::abi::Abi>> {
        self.get("abi", address)
    }

    fn set_abi(&self, address: Address, abi: Option<&Abi>) {
        self.set("abi", address, abi)
    }

    fn get_source(&self, address: Address) -> Option<Option<ContractMetadata>> {
        self.get("sources", address)
    }

    fn set_source(&self, address: Address, source: Option<&ContractMetadata>) {
        self.set("sources", address, source)
    }

    fn set<T: Serialize>(&self, prefix: &str, address: Address, item: T) {
        let path = self.root.join(prefix).join(format!("{:?}.json", address));
        let writer = std::fs::File::create(path).ok().map(std::io::BufWriter::new);
        if let Some(mut writer) = writer {
            let _ = serde_json::to_writer(
                &mut writer,
                &CacheEnvelope {
                    expiry: SystemTime::now()
                        .checked_add(self.ttl)
                        .expect("cache ttl overflowed")
                        .duration_since(UNIX_EPOCH)
                        .expect("system time is before unix epoch")
                        .as_secs(),
                    data: item,
                },
            );
            let _ = writer.flush();
        }
    }

    fn get<T: DeserializeOwned>(&self, prefix: &str, address: Address) -> Option<T> {
        let path = self.root.join(prefix).join(format!("{:?}.json", address));
        let reader = std::io::BufReader::new(std::fs::File::open(path).ok()?);
        if let Ok(inner) = serde_json::from_reader::<_, CacheEnvelope<T>>(reader) {
            // If this does not return None then we have passed the expiry
            if SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is before unix epoch")
                .checked_sub(Duration::from_secs(inner.expiry))
                .is_some()
            {
                return None
            }

            return Some(inner.data)
        }
        None
    }
}

impl Client {
    pub fn new_cached(
        chain: Chain,
        api_key: impl Into<String>,
        cache_root: Option<PathBuf>,
        cache_ttl: Duration,
    ) -> Result<Self> {
        let mut this = Self::new(chain, api_key)?;
        this.cache = cache_root.map(|root| Cache::new(root, cache_ttl));
        Ok(this)
    }

    /// Create a new client with the correct endpoints based on the chain and provided API key
    pub fn new(chain: Chain, api_key: impl Into<String>) -> Result<Self> {
        let (etherscan_api_url, etherscan_url) = match chain {
            Chain::Mainnet => {
                (Url::parse("https://api.etherscan.io/api"), Url::parse("https://etherscan.io"))
            }
            Chain::Ropsten | Chain::Kovan | Chain::Rinkeby | Chain::Goerli => {
                let chain_name = chain.to_string().to_lowercase();

                (
                    Url::parse(&format!("https://api-{}.etherscan.io/api", chain_name)),
                    Url::parse(&format!("https://{}.etherscan.io", chain_name)),
                )
            }
            Chain::Polygon => (
                Url::parse("https://api.polygonscan.com/api"),
                Url::parse("https://polygonscan.com"),
            ),
            Chain::PolygonMumbai => (
                Url::parse("https://api-testnet.polygonscan.com/api"),
                Url::parse("https://mumbai.polygonscan.com"),
            ),
            Chain::Avalanche => {
                (Url::parse("https://api.snowtrace.io/api"), Url::parse("https://snowtrace.io"))
            }
            Chain::AvalancheFuji => (
                Url::parse("https://api-testnet.snowtrace.io/api"),
                Url::parse("https://testnet.snowtrace.io"),
            ),
            Chain::Optimism => (
                Url::parse("https://api-optimistic.etherscan.io/api"),
                Url::parse("https://optimistic.etherscan.io"),
            ),
            Chain::OptimismKovan => (
                Url::parse("https://api-kovan-optimistic.etherscan.io/api"),
                Url::parse("https://kovan-optimistic.etherscan.io"),
            ),
            Chain::Fantom => {
                (Url::parse("https://api.ftmscan.com/api"), Url::parse("https://ftmscan.com"))
            }
            Chain::FantomTestnet => (
                Url::parse("https://api-testnet.ftmscan.com/api"),
                Url::parse("https://testnet.ftmscan.com"),
            ),
            Chain::BinanceSmartChain => {
                (Url::parse("https://api.bscscan.com/api"), Url::parse("https://bscscan.com"))
            }
            Chain::BinanceSmartChainTestnet => (
                Url::parse("https://api-testnet.bscscan.com/api"),
                Url::parse("https://testnet.bscscan.com"),
            ),
            Chain::Arbitrum => {
                (Url::parse("https://api.arbiscan.io/api"), Url::parse("https://arbiscan.io"))
            }
            Chain::ArbitrumTestnet => (
                Url::parse("https://api-testnet.arbiscan.io/api"),
                Url::parse("https://testnet.arbiscan.io"),
            ),
            Chain::Cronos => {
                (Url::parse("https://api.cronoscan.com/api"), Url::parse("https://cronoscan.com"))
            }
            Chain::Dev => return Err(EtherscanError::LocalNetworksNotSupported),
            chain => return Err(EtherscanError::ChainNotSupported(chain)),
        };

        Ok(Self {
            client: Default::default(),
            api_key: api_key.into(),
            etherscan_api_url: etherscan_api_url.expect("is valid http"),
            etherscan_url: etherscan_url.expect("is valid http"),
            cache: None,
        })
    }

    /// Create a new client with the correct endpoints based on the chain and API key
    /// from ETHERSCAN_API_KEY environment variable
    pub fn new_from_env(chain: Chain) -> Result<Self> {
        let api_key = match chain {
            Chain::Avalanche | Chain::AvalancheFuji => std::env::var("SNOWTRACE_API_KEY")?,
            Chain::Polygon | Chain::PolygonMumbai => std::env::var("POLYGONSCAN_API_KEY")?,
            Chain::Mainnet |
            Chain::Ropsten |
            Chain::Kovan |
            Chain::Rinkeby |
            Chain::Goerli |
            Chain::Optimism |
            Chain::OptimismKovan |
            Chain::BinanceSmartChain |
            Chain::BinanceSmartChainTestnet |
            Chain::Arbitrum |
            Chain::ArbitrumTestnet |
            Chain::Cronos => std::env::var("ETHERSCAN_API_KEY")?,
            Chain::Fantom | Chain::FantomTestnet => {
                std::env::var("FTMSCAN_API_KEY").or_else(|_| std::env::var("FANTOMSCAN_API_KEY"))?
            }

            Chain::XDai | Chain::Sepolia | Chain::CronosTestnet => String::default(),
            Chain::Moonbeam | Chain::MoonbeamDev | Chain::Moonriver => {
                std::env::var("MOONSCAN_API_KEY")?
            }
            Chain::Dev => return Err(EtherscanError::LocalNetworksNotSupported),
        };
        Self::new(chain, api_key)
    }

    pub fn etherscan_api_url(&self) -> &Url {
        &self.etherscan_api_url
    }

    pub fn etherscan_url(&self) -> &Url {
        &self.etherscan_url
    }

    /// Return the URL for the given block number
    pub fn block_url(&self, block: u64) -> String {
        format!("{}block/{}", self.etherscan_url, block)
    }

    /// Return the URL for the given address
    pub fn address_url(&self, address: Address) -> String {
        format!("{}address/{:?}", self.etherscan_url, address)
    }

    /// Return the URL for the given transaction hash
    pub fn transaction_url(&self, tx_hash: H256) -> String {
        format!("{}tx/{:?}", self.etherscan_url, tx_hash)
    }

    /// Return the URL for the given token hash
    pub fn token_url(&self, token_hash: Address) -> String {
        format!("{}token/{:?}", self.etherscan_url, token_hash)
    }

    /// Execute an API POST request with a form
    async fn post_form<T: DeserializeOwned, Form: Serialize>(
        &self,
        form: &Form,
    ) -> Result<Response<T>> {
        trace!(target: "etherscan", "POST FORM {}", self.etherscan_api_url);
        Ok(self
            .client
            .post(self.etherscan_api_url.clone())
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(form)
            .send()
            .await?
            .json()
            .await?)
    }

    /// Execute an API GET request with parameters
    async fn get_json<T: DeserializeOwned, Q: Serialize>(&self, query: &Q) -> Result<Response<T>> {
        trace!(target: "etherscan", "GET JSON {}", self.etherscan_api_url);
        let res: ResponseData<T> = self
            .client
            .get(self.etherscan_api_url.clone())
            .header(header::ACCEPT, "application/json")
            .query(query)
            .send()
            .await?
            .json()
            .await?;

        match res {
            ResponseData::Error { result, .. } => {
                if result.starts_with("Max rate limit reached") {
                    Err(EtherscanError::RateLimitExceeded)
                } else {
                    Err(EtherscanError::Unknown(result))
                }
            }
            ResponseData::Success(res) => Ok(res),
        }
    }

    fn create_query<T: Serialize>(
        &self,
        module: &'static str,
        action: &'static str,
        other: T,
    ) -> Query<T> {
        Query {
            apikey: Cow::Borrowed(&self.api_key),
            module: Cow::Borrowed(module),
            action: Cow::Borrowed(action),
            other,
        }
    }
}

/// The API response type
#[derive(Debug, Clone, Deserialize)]
pub struct Response<T> {
    pub status: String,
    pub message: String,
    pub result: T,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ResponseData<T> {
    Success(Response<T>),
    Error { status: String, message: String, result: String },
}

/// The type that gets serialized as query
#[derive(Debug, Serialize)]
struct Query<'a, T: Serialize> {
    apikey: Cow<'a, str>,
    module: Cow<'a, str>,
    action: Cow<'a, str>,
    #[serde(flatten)]
    other: T,
}

#[cfg(test)]
mod tests {
    use crate::{Client, EtherscanError};
    use ethers_core::types::{Address, Chain, H256};
    use std::{
        future::Future,
        time::{Duration, SystemTime},
    };

    #[test]
    fn chain_not_supported() {
        let err = Client::new_from_env(Chain::XDai).unwrap_err();

        assert!(matches!(err, EtherscanError::ChainNotSupported(_)));
        assert_eq!(err.to_string(), "Chain xdai not supported");
    }

    #[test]
    fn stringifies_block_url() {
        let etherscan = Client::new_from_env(Chain::Mainnet).unwrap();
        let block: u64 = 1;
        let block_url: String = etherscan.block_url(block);
        assert_eq!(block_url, format!("https://etherscan.io/block/{}", block));
    }

    #[test]
    fn stringifies_address_url() {
        let etherscan = Client::new_from_env(Chain::Mainnet).unwrap();
        let addr: Address = Address::zero();
        let address_url: String = etherscan.address_url(addr);
        assert_eq!(address_url, format!("https://etherscan.io/address/{:?}", addr));
    }

    #[test]
    fn stringifies_transaction_url() {
        let etherscan = Client::new_from_env(Chain::Mainnet).unwrap();
        let tx_hash = H256::zero();
        let tx_url: String = etherscan.transaction_url(tx_hash);
        assert_eq!(tx_url, format!("https://etherscan.io/tx/{:?}", tx_hash));
    }

    #[test]
    fn stringifies_token_url() {
        let etherscan = Client::new_from_env(Chain::Mainnet).unwrap();
        let token_hash = Address::zero();
        let token_url: String = etherscan.token_url(token_hash);
        assert_eq!(token_url, format!("https://etherscan.io/token/{:?}", token_hash));
    }

    #[test]
    fn local_networks_not_supported() {
        let err = Client::new_from_env(Chain::Dev).unwrap_err();
        assert!(matches!(err, EtherscanError::LocalNetworksNotSupported));
    }

    pub async fn run_at_least_duration(duration: Duration, block: impl Future) {
        let start = SystemTime::now();
        block.await;
        if let Some(sleep) = duration.checked_sub(start.elapsed().unwrap()) {
            tokio::time::sleep(sleep).await;
        }
    }
}
