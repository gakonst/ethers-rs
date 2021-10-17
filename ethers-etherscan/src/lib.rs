//! Bindings for [etherscan.io web api](https://docs.etherscan.io/)

mod contract;
mod errors;
mod transaction;

use errors::EtherscanError;
use ethers_core::abi::Address;
use reqwest::{header, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{borrow::Cow, fmt};

pub type Result<T> = std::result::Result<T, EtherscanError>;

/// The Etherscan.io API client.
#[derive(Clone)]
pub struct Client {
    /// Client that executes HTTP requests
    client: reqwest::Client,
    /// Etherscan API key
    api_key: String,
    /// Etherscan API endpoint like https://api(-chain).etherscan.io/api
    etherscan_api_url: Url,
    /// Etherscan base endpoint like https://etherscan.io
    etherscan_url: Url,
}

#[derive(Debug)]
pub enum Chain {
    Mainnet,
    Ropsten,
    Kovan,
    Rinkeby,
    Goerli,
}

impl fmt::Display for Chain {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl Client {
    /// Create a new client with the correct endpoints based on the chain and provided API key
    pub fn new(chain: Chain, api_key: impl Into<String>) -> Self {
        let (etherscan_api_url, etherscan_url) = match chain {
            Chain::Mainnet => {
                (Url::parse("https://api.etherscan.io/api"), Url::parse("https://etherscan.io"))
            }
            Chain::Ropsten | Chain::Kovan | Chain::Rinkeby | Chain::Goerli => (
                Url::parse(&format!("https://api-{}.etherscan.io/api", chain)),
                Url::parse(&format!("https://{}.etherscan.io", chain)),
            ),
        };

        Self {
            client: Default::default(),
            api_key: api_key.into(),
            etherscan_api_url: etherscan_api_url.expect("is valid http"),
            etherscan_url: etherscan_url.expect("is valid http"),
        }
    }

    /// Create a new client with the correct endpoints based on the chain and API key
    /// from ETHERSCAN_API_KEY environment variable
    pub fn new_from_env(chain: Chain) -> Result<Self> {
        Ok(Self::new(chain, std::env::var("ETHERSCAN_API_KEY")?))
    }

    pub fn etherscan_api_url(&self) -> &Url {
        &self.etherscan_api_url
    }

    pub fn etherscan_url(&self) -> &Url {
        &self.etherscan_url
    }

    /// Return the URL for the given block number
    pub fn block_url(&self, block: u64) -> String {
        format!("{}/block/{}", self.etherscan_url, block)
    }

    /// Return the URL for the given address
    pub fn address_url(&self, address: Address) -> String {
        format!("{}/address/{}", self.etherscan_url, address)
    }

    /// Return the URL for the given transaction hash
    pub fn transaction_url(&self, tx_hash: impl AsRef<str>) -> String {
        format!("{}/tx/{}", self.etherscan_url, tx_hash.as_ref())
    }

    /// Return the URL for the given token hash
    pub fn token_url(&self, token_hash: impl AsRef<str>) -> String {
        format!("{}/token/{}", self.etherscan_url, token_hash.as_ref())
    }

    /// Execute an API POST request with a form
    async fn post_form<T: DeserializeOwned, Form: Serialize>(
        &self,
        form: &Form,
    ) -> Result<Response<T>> {
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
        Ok(self
            .client
            .get(self.etherscan_api_url.clone())
            .header(header::ACCEPT, "application/json")
            .query(query)
            .send()
            .await?
            .json()
            .await?)
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

/// The type that gets serialized as query
#[derive(Debug, Serialize)]
struct Query<'a, T: Serialize> {
    apikey: Cow<'a, str>,
    module: Cow<'a, str>,
    action: Cow<'a, str>,
    #[serde(flatten)]
    other: T,
}
