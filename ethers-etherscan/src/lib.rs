//! Bindings for [etherscan.io web api](https://docs.etherscan.io/)

use crate::errors::is_blocked_by_cloudflare_response;
use contract::ContractMetadata;
use errors::EtherscanError;
use ethers_core::{
    abi::{Abi, Address},
    types::{Chain, H256},
};
use reqwest::{header, IntoUrl, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    borrow::Cow,
    io::Write,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tracing::{error, trace};

pub mod account;
pub mod contract;
pub mod errors;
pub mod gas;
pub mod source_tree;
pub mod transaction;
pub mod utils;
pub mod verify;

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

impl Client {
    /// Creates a `ClientBuilder` to configure a `Client`.
    /// This is the same as `ClientBuilder::default()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ethers_core::types::Chain;
    /// use ethers_etherscan::Client;
    /// let client = Client::builder().with_api_key("<API KEY>").chain(Chain::Mainnet).unwrap().build().unwrap();
    /// ```
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Creates a new instance that caches etherscan requests
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
        Client::builder().with_api_key(api_key).chain(chain)?.build()
    }

    /// Create a new client with the correct endpoints based on the chain and API key
    /// from ETHERSCAN_API_KEY environment variable
    pub fn new_from_env(chain: Chain) -> Result<Self> {
        let api_key = match chain {
            Chain::Avalanche | Chain::AvalancheFuji => std::env::var("SNOWTRACE_API_KEY")?,
            Chain::Polygon | Chain::PolygonMumbai => std::env::var("POLYGONSCAN_API_KEY")?,
            Chain::Mainnet |
            Chain::Morden |
            Chain::Ropsten |
            Chain::Kovan |
            Chain::Rinkeby |
            Chain::Goerli |
            Chain::Optimism |
            Chain::OptimismGoerli |
            Chain::OptimismKovan |
            Chain::BinanceSmartChain |
            Chain::BinanceSmartChainTestnet |
            Chain::Arbitrum |
            Chain::ArbitrumTestnet |
            Chain::ArbitrumGoerli |
            Chain::Cronos |
            Chain::CronosTestnet |
            Chain::Aurora |
            Chain::AuroraTestnet => std::env::var("ETHERSCAN_API_KEY")?,
            Chain::Fantom | Chain::FantomTestnet => {
                std::env::var("FTMSCAN_API_KEY").or_else(|_| std::env::var("FANTOMSCAN_API_KEY"))?
            }
            Chain::XDai |
            Chain::Chiado |
            Chain::Sepolia |
            Chain::Rsk |
            Chain::Sokol |
            Chain::Poa |
            Chain::Oasis |
            Chain::Emerald |
            Chain::EmeraldTestnet |
            Chain::Evmos |
            Chain::EvmosTestnet => String::default(),
            Chain::Moonbeam | Chain::Moonbase | Chain::MoonbeamDev | Chain::Moonriver => {
                std::env::var("MOONSCAN_API_KEY")?
            }
            Chain::AnvilHardhat | Chain::Dev => {
                return Err(EtherscanError::LocalNetworksNotSupported)
            }
        };
        Self::new(chain, api_key)
    }

    /// Sets the root to the cache dir and the ttl to use
    pub fn set_cache(&mut self, root: impl Into<PathBuf>, ttl: Duration) -> &mut Self {
        self.cache = Some(Cache { root: root.into(), ttl });
        self
    }

    pub fn etherscan_api_url(&self) -> &Url {
        &self.etherscan_api_url
    }

    pub fn etherscan_url(&self) -> &Url {
        &self.etherscan_url
    }

    /// Return the URL for the given block number
    pub fn block_url(&self, block: u64) -> String {
        format!("{}block/{block}", self.etherscan_url)
    }

    /// Return the URL for the given address
    pub fn address_url(&self, address: Address) -> String {
        format!("{}address/{address:?}", self.etherscan_url)
    }

    /// Return the URL for the given transaction hash
    pub fn transaction_url(&self, tx_hash: H256) -> String {
        format!("{}tx/{tx_hash:?}", self.etherscan_url)
    }

    /// Return the URL for the given token hash
    pub fn token_url(&self, token_hash: Address) -> String {
        format!("{}token/{token_hash:?}", self.etherscan_url)
    }

    /// Execute an GET request with parameters.
    async fn get_json<T: DeserializeOwned, Q: Serialize>(&self, query: &Q) -> Result<Response<T>> {
        let res = self.get(query).await?;
        self.sanitize_response(res)
    }

    /// Execute a GET request with parameters, without sanity checking the response.
    async fn get<Q: Serialize>(&self, query: &Q) -> Result<String> {
        trace!(target: "etherscan", "GET {}", self.etherscan_api_url);
        let response = self
            .client
            .get(self.etherscan_api_url.clone())
            .header(header::ACCEPT, "application/json")
            .query(query)
            .send()
            .await?
            .text()
            .await?;
        Ok(response)
    }

    /// Execute a POST request with a form.
    async fn post_form<T: DeserializeOwned, F: Serialize>(&self, form: &F) -> Result<Response<T>> {
        let res = self.post(form).await?;
        self.sanitize_response(res)
    }

    /// Execute a POST request with a form, without sanity checking the response.
    async fn post<F: Serialize>(&self, form: &F) -> Result<String> {
        trace!(target: "etherscan", "POST {}", self.etherscan_api_url);
        let response = self
            .client
            .post(self.etherscan_api_url.clone())
            .form(form)
            .send()
            .await?
            .text()
            .await?;
        Ok(response)
    }

    /// Perform sanity checks on a response and deserialize it into a [Response].
    fn sanitize_response<T: DeserializeOwned>(&self, res: impl AsRef<str>) -> Result<Response<T>> {
        let res = res.as_ref();
        let res: ResponseData<T> = serde_json::from_str(res).map_err(|err| {
            error!(target: "etherscan", ?res, "Failed to deserialize response: {}", err);
            if is_blocked_by_cloudflare_response(res) {
                EtherscanError::BlockedByCloudflare
            } else {
                EtherscanError::Serde(err)
            }
        })?;

        match res {
            ResponseData::Error { result, .. } => {
                if result.starts_with("Max rate limit reached") {
                    Err(EtherscanError::RateLimitExceeded)
                } else if result.to_lowercase() == "invalid api key" {
                    Err(EtherscanError::InvalidApiKey)
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

#[derive(Clone, Debug, Default)]
pub struct ClientBuilder {
    /// Client that executes HTTP requests
    client: Option<reqwest::Client>,
    /// Etherscan API key
    api_key: Option<String>,
    /// Etherscan API endpoint like <https://api(-chain).etherscan.io/api>
    etherscan_api_url: Option<Url>,
    /// Etherscan base endpoint like <https://etherscan.io>
    etherscan_url: Option<Url>,
    /// Path to where ABI files should be cached
    cache: Option<Cache>,
}

// === impl ClientBuilder ===

impl ClientBuilder {
    /// Configures the etherscan url and api url for the given chain
    ///
    /// # Errors
    ///
    /// Fails if the chain is not supported by etherscan
    pub fn chain(self, chain: Chain) -> Result<Self> {
        fn urls(
            api: impl IntoUrl,
            url: impl IntoUrl,
        ) -> (reqwest::Result<Url>, reqwest::Result<Url>) {
            (api.into_url(), url.into_url())
        }
        let (etherscan_api_url, etherscan_url) = chain
            .etherscan_urls()
            .map(|(api, base)| urls(api, base))
            .ok_or_else(|| EtherscanError::ChainNotSupported(chain))?;
        self.with_api_url(etherscan_api_url?)?.with_url(etherscan_url?)
    }

    /// Configures the etherscan url
    ///
    /// # Errors
    ///
    /// Fails if the `etherscan_url` is not a valid `Url`
    pub fn with_url(mut self, etherscan_url: impl IntoUrl) -> Result<Self> {
        self.etherscan_url = Some(etherscan_url.into_url()?);
        Ok(self)
    }

    /// Configures the `reqwest::Client`
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Configures the etherscan api url
    ///
    /// # Errors
    ///
    /// Fails if the `etherscan_api_url` is not a valid `Url`
    pub fn with_api_url(mut self, etherscan_api_url: impl IntoUrl) -> Result<Self> {
        self.etherscan_api_url = Some(etherscan_api_url.into_url()?);
        Ok(self)
    }

    /// Configures the etherscan api key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Configures cache for etherscan request
    pub fn with_cache(mut self, cache_root: Option<PathBuf>, cache_ttl: Duration) -> Self {
        self.cache = cache_root.map(|root| Cache::new(root, cache_ttl));
        self
    }

    /// Returns a Client that uses this ClientBuilder configuration.
    ///
    /// # Errors
    /// if required fields are missing:
    ///   - `api_key`
    ///   - `etherscan_api_url`
    ///   - `etherscan_url`
    pub fn build(self) -> Result<Client> {
        let ClientBuilder { client, api_key, etherscan_api_url, etherscan_url, cache } = self;

        let client = Client {
            client: client.unwrap_or_default(),
            api_key: api_key
                .ok_or_else(|| EtherscanError::Builder("etherscan api key".to_string()))?,
            etherscan_api_url: etherscan_api_url
                .ok_or_else(|| EtherscanError::Builder("etherscan api url".to_string()))?,
            etherscan_url: etherscan_url
                .ok_or_else(|| EtherscanError::Builder("etherscan url".to_string()))?,
            cache,
        };
        Ok(client)
    }
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
        let path = self.root.join(prefix).join(format!("{address:?}.json"));
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
        let path = self.root.join(prefix).join(format!("{address:?}.json"));
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
#[derive(Clone, Debug, Serialize)]
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
        let err = Client::new_from_env(Chain::Morden).unwrap_err();

        assert!(matches!(err, EtherscanError::ChainNotSupported(_)));
        assert_eq!(err.to_string(), "Chain morden not supported");
    }

    #[test]
    fn stringifies_block_url() {
        let etherscan = Client::new_from_env(Chain::Mainnet).unwrap();
        let block: u64 = 1;
        let block_url: String = etherscan.block_url(block);
        assert_eq!(block_url, format!("https://etherscan.io/block/{block}"));
    }

    #[test]
    fn stringifies_address_url() {
        let etherscan = Client::new_from_env(Chain::Mainnet).unwrap();
        let addr: Address = Address::zero();
        let address_url: String = etherscan.address_url(addr);
        assert_eq!(address_url, format!("https://etherscan.io/address/{addr:?}"));
    }

    #[test]
    fn stringifies_transaction_url() {
        let etherscan = Client::new_from_env(Chain::Mainnet).unwrap();
        let tx_hash = H256::zero();
        let tx_url: String = etherscan.transaction_url(tx_hash);
        assert_eq!(tx_url, format!("https://etherscan.io/tx/{tx_hash:?}"));
    }

    #[test]
    fn stringifies_token_url() {
        let etherscan = Client::new_from_env(Chain::Mainnet).unwrap();
        let token_hash = Address::zero();
        let token_url: String = etherscan.token_url(token_hash);
        assert_eq!(token_url, format!("https://etherscan.io/token/{token_hash:?}"));
    }

    #[test]
    fn local_networks_not_supported() {
        let err = Client::new_from_env(Chain::Dev).unwrap_err();
        assert!(matches!(err, EtherscanError::LocalNetworksNotSupported));
    }

    #[tokio::test]
    async fn check_wrong_etherscan_api_key() {
        let client = Client::new(Chain::Mainnet, "ABCDEFG").unwrap();
        let resp = client
            .contract_source_code("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
            .await
            .unwrap_err();

        assert!(matches!(resp, EtherscanError::InvalidApiKey));
    }

    pub async fn run_at_least_duration(duration: Duration, block: impl Future) {
        let start = SystemTime::now();
        block.await;
        if let Some(sleep) = duration.checked_sub(start.elapsed().unwrap()) {
            tokio::time::sleep(sleep).await;
        }
    }
}
