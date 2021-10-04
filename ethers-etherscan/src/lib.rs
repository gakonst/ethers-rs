//! Bindings for [etherscan.io web api](https://docs.etherscan.io/)

use ethers_core::abi::{Abi, Address};
use reqwest::{header, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

/// The Etherscan.io api client.
#[derive(Clone)]
pub struct Client {
    /// The client that executes the http requests
    client: reqwest::Client,
    /// The etherscan api key
    api_key: String,
    /// API endpoint like https://api(-chain).etherscan.io/api
    etherscan_api_url: Url,
    /// Base etherscan endpoint like https://etherscan.io/address
    etherscan_url: Url,
}

impl Client {
    /// Create a new client with the correct endpoints based on the chain.
    ///
    /// Supported chains are ethlive, mainnet,ropsten, kovan, rinkeby, goerli
    // TODO make this an enum
    pub fn new(chain: &str, api_key: impl Into<String>) -> anyhow::Result<Self> {
        let (etherscan_api_url, etherscan_url) = match chain {
            "ethlive" | "mainnet" => {
                (
                    Url::parse("https://api.etherscan.io/api"),
                    Url::parse("https://etherscan.io/address"),
                )
            },
            "ropsten"|"kovan"|"rinkeby"|"goerli" => {
                (
                    Url::parse(&format!("https://api-{}.etherscan.io/api", chain)),
                    Url::parse(&format!("https://{}.etherscan.io/address", chain)),
                )
            }
            s => {
                return Err(
                    anyhow::anyhow!("Verification only works on mainnet, ropsten, kovan, rinkeby, and goerli, found `{}` chain", s)
                )
            }
        };

        Ok(Self {
            client: Default::default(),
            api_key: api_key.into(),
            etherscan_api_url: etherscan_api_url.expect("is valid http"),
            etherscan_url: etherscan_url.expect("is valid http"),
        })
    }

    pub fn etherscan_api_url(&self) -> &Url {
        &self.etherscan_api_url
    }

    pub fn etherscan_url(&self) -> &Url {
        &self.etherscan_url
    }

    /// Return the URL for the given address
    pub fn address_url(&self, address: Address) -> String {
        format!("{}/{}", self.etherscan_url, address)
    }

    /// Execute a api POST request with a form
    async fn post_form<T: DeserializeOwned, Form: Serialize>(
        &self,
        form: &Form,
    ) -> anyhow::Result<Response<T>> {
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

    /// Execute a api GET query
    async fn get_json<T: DeserializeOwned, Q: Serialize>(
        &self,
        query: &Q,
    ) -> anyhow::Result<Response<T>> {
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

    /// Submit Source Code for Verification
    pub async fn submit_contract_verification(
        &self,
        contract: &VerifyContract,
    ) -> anyhow::Result<Response<String>> {
        let body = self.create_query("contract", "verifysourcecode", contract);
        Ok(self.post_form(&body).await?)
    }

    /// Check Source Code Verification Status with receipt received from
    /// `[Self::submit_contract_verification]`
    pub async fn check_verify_status(
        &self,
        guid: impl AsRef<str>,
    ) -> anyhow::Result<Response<String>> {
        let mut map = HashMap::new();
        map.insert("guid", guid.as_ref());
        let body = self.create_query("contract", "checkverifystatus", map);
        Ok(self.post_form(&body).await?)
    }

    /// Returns the contract ABI of a verified contract
    ///
    /// ```no_run
    /// # use ethers_etherscan::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    ///     let client = Client::new("mainnet", "API_KEY").unwrap();
    ///     let abi = client
    ///         .contract_abi("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
    ///         .await?;
    ///
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn contract_abi(&self, address: Address) -> anyhow::Result<Abi> {
        let mut map = HashMap::new();
        map.insert("address", address);
        let query = self.create_query("contract", "getabi", map);
        let resp: Response<String> = self.get_json(&query).await?;
        Ok(serde_json::from_str(&resp.result)?)
    }

    /// Get Contract Source Code for Verified Contract Source Codes
    /// ```no_run
    /// # use ethers_etherscan::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    ///     let client = Client::new("mainnet", "API_KEY").unwrap();
    ///     let meta = client
    ///         .contract_source_code("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
    ///         .await?;
    ///     let code = meta.source_code();
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn contract_source_code(&self, address: Address) -> anyhow::Result<ContractMetadata> {
        let mut map = HashMap::new();
        map.insert("address", address);
        let query = self.create_query("contract", "getsourcecode", map);
        let response: Response<Vec<Metadata>> = self.get_json(&query).await?;
        Ok(ContractMetadata {
            items: response.result,
        })
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

/// Arguments for verifying contracts
#[derive(Debug, Clone, Serialize)]
pub struct VerifyContract {
    pub address: Address,
    pub source: String,
    #[serde(rename = "codeformat")]
    pub code_format: CodeFormat,
    /// if codeformat=solidity-standard-json-input, then expected as
    /// `erc20.sol:erc20`
    #[serde(rename = "contractname", skip_serializing_if = "Option::is_none")]
    pub contract_name: Option<String>,
    #[serde(rename = "compilerversion")]
    pub compiler_version: String,
    /// applicable when codeformat=solidity-single-file
    #[serde(rename = "optimizationUsed", skip_serializing_if = "Option::is_none")]
    optimization_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runs: Option<String>,
    /// NOTE: there is a typo in the etherscan API `constructorArguements`
    #[serde(
        rename = "constructorArguements",
        skip_serializing_if = "Option::is_none"
    )]
    pub constructor_arguments: Option<String>,
    #[serde(rename = "evmversion")]
    pub evm_version: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

impl VerifyContract {
    pub fn new(address: Address, source: String, compilerversion: String) -> Self {
        Self {
            address,
            source,
            code_format: Default::default(),
            contract_name: None,
            compiler_version: compilerversion,
            optimization_used: None,
            runs: None,
            constructor_arguments: None,
            evm_version: None,
            other: Default::default(),
        }
    }

    pub fn contract_name(mut self, name: impl Into<String>) -> Self {
        self.contract_name = Some(name.into());
        self
    }

    pub fn runs(mut self, runs: u32) -> Self {
        self.runs = Some(format!("{}", runs));
        self
    }

    pub fn optimization(self, optimization: bool) -> Self {
        if optimization {
            self.optimized()
        } else {
            self.not_optimized()
        }
    }

    pub fn optimized(mut self) -> Self {
        self.optimization_used = Some("1".to_string());
        self
    }

    pub fn not_optimized(mut self) -> Self {
        self.optimization_used = Some("0".to_string());
        self
    }

    pub fn code_format(mut self, code_format: CodeFormat) -> Self {
        self.code_format = code_format;
        self
    }

    pub fn evm_version(mut self, evm_version: impl Into<String>) -> Self {
        self.evm_version = Some(evm_version.into());
        self
    }

    pub fn constructor_arguments(
        mut self,
        constructor_arguments: Option<impl Into<String>>,
    ) -> Self {
        self.constructor_arguments = constructor_arguments.map(|s| {
            s.into()
                // TODO is this correct?
                .trim_start_matches("0x")
                .to_string()
        });
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CodeFormat {
    #[serde(rename = "solidity-single-file")]
    SingleFile,
    #[serde(rename = "solidity-standard-json-inpu")]
    StandardJsonInput,
}

impl AsRef<str> for CodeFormat {
    fn as_ref(&self) -> &str {
        match self {
            CodeFormat::SingleFile => "solidity-single-file",
            CodeFormat::StandardJsonInput => "solidity-standard-json-input",
        }
    }
}

impl Default for CodeFormat {
    fn default() -> Self {
        CodeFormat::SingleFile
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractMetadata {
    #[serde(flatten)]
    pub items: Vec<Metadata>,
}

impl IntoIterator for ContractMetadata {
    type Item = Metadata;
    type IntoIter = std::vec::IntoIter<Metadata>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl ContractMetadata {
    /// All ABI from all contracts in the source file
    pub fn abis(&self) -> anyhow::Result<Vec<Abi>> {
        let mut abis = Vec::with_capacity(self.items.len());
        for item in &self.items {
            abis.push(serde_json::from_str(&item.abi)?);
        }
        Ok(abis)
    }

    /// Combined source code of all contracts
    pub fn source_code(&self) -> String {
        self.items
            .iter()
            .map(|c| c.source_code.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Etherscan contract metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "SourceCode")]
    pub source_code: String,
    #[serde(rename = "ABI")]
    pub abi: String,
    #[serde(rename = "ContractName")]
    pub contract_name: String,
    #[serde(rename = "CompilerVersion")]
    pub compiler_version: String,
    #[serde(rename = "OptimizationUsed")]
    pub optimization_used: String,
    #[serde(rename = "Runs")]
    pub runs: String,
    #[serde(rename = "ConstructorArguments")]
    pub constructor_arguments: String,
    #[serde(rename = "EVMVersion")]
    pub evm_version: String,
    #[serde(rename = "Library")]
    pub library: String,
    #[serde(rename = "LicenseType")]
    pub license_type: String,
    #[serde(rename = "Proxy")]
    pub proxy: String,
    #[serde(rename = "Implementation")]
    pub implementation: String,
    #[serde(rename = "SwarmSource")]
    pub swarm_source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn api_key() -> String {
        std::env::var("ETHERSCAN_API_KEY").expect("ETHERSCAN_API_KEY not found")
    }

    #[tokio::test]
    #[ignore]
    async fn can_fetch_contract_abi() {
        let api = api_key();
        let client = Client::new("mainnet", api).unwrap();

        let _abi = client
            .contract_abi(
                "0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413"
                    .parse()
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn can_fetch_contract_source_code() {
        let api = api_key();
        let client = Client::new("mainnet", api).unwrap();

        let _meta = client
            .contract_source_code(
                "0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413"
                    .parse()
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn can_verify_contract() {
        // TODO this needs further investigation

        // https://etherscan.io/address/0x9e744c9115b74834c0f33f4097f40c02a9ac5c33#code
        let contract = include_str!("../resources/UniswapExchange.sol");
        let address = "0x9e744c9115b74834c0f33f4097f40c02a9ac5c33"
            .parse()
            .unwrap();
        let compiler_version = "v0.5.17+commit.d19bba13";
        let constructor_args = "0x000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000005f5e1000000000000000000000000000000000000000000000000000000000000000007596179537761700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000035941590000000000000000000000000000000000000000000000000000000000";

        let api = api_key();
        let client = Client::new("mainnet", api).unwrap();

        let contract =
            VerifyContract::new(address, contract.to_string(), compiler_version.to_string())
                .constructor_arguments(Some(constructor_args))
                .optimization(true)
                .runs(200);

        let resp = client.submit_contract_verification(&contract).await;
    }
}
