use crate::{
    source_tree::{SourceTree, SourceTreeEntry},
    utils::{deserialize_address_opt, deserialize_string_or_struct, deserialize_version},
    Client, EtherscanError, Response, Result,
};
use ethers_core::{
    abi::{Abi, Address},
    types::{serde_helpers::deserialize_stringified_u64, Bytes},
};
use ethers_solc::{artifacts::Settings, EvmVersion, Project, ProjectBuilder, SolcConfig};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::Infallible, path::Path, str::FromStr};

/* --------------------------------------- VerifyContract --------------------------------------- */

/// Arguments for verifying contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyContract {
    #[serde(rename = "contractaddress")]
    pub address: Address,
    #[serde(rename = "sourceCode")]
    pub source: String,
    #[serde(rename = "codeformat")]
    pub code_format: CodeFormat,
    /// if codeformat=solidity-standard-json-input, then expected as
    /// `erc20.sol:erc20`
    #[serde(rename = "contractname")]
    pub contract_name: String,
    #[serde(rename = "compilerversion")]
    pub compiler_version: String,
    /// applicable when codeformat=solidity-single-file
    #[serde(rename = "optimizationUsed", skip_serializing_if = "Option::is_none")]
    pub optimization_used: Option<String>,
    /// applicable when codeformat=solidity-single-file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runs: Option<String>,
    /// NOTE: there is a typo in the etherscan API `constructorArguements`
    #[serde(rename = "constructorArguements", skip_serializing_if = "Option::is_none")]
    pub constructor_arguments: Option<String>,
    /// applicable when codeformat=solidity-single-file
    #[serde(rename = "evmversion", skip_serializing_if = "Option::is_none")]
    pub evm_version: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

impl VerifyContract {
    pub fn new(
        address: Address,
        contract_name: String,
        source: String,
        compiler_version: String,
    ) -> Self {
        Self {
            address,
            source,
            code_format: Default::default(),
            contract_name,
            compiler_version,
            optimization_used: None,
            runs: None,
            constructor_arguments: None,
            evm_version: None,
            other: Default::default(),
        }
    }

    #[must_use]
    pub fn runs(mut self, runs: u32) -> Self {
        self.runs = Some(format!("{}", runs));
        self
    }

    #[must_use]
    pub fn optimization(self, optimization: bool) -> Self {
        if optimization {
            self.optimized()
        } else {
            self.not_optimized()
        }
    }

    #[must_use]
    pub fn optimized(mut self) -> Self {
        self.optimization_used = Some("1".to_string());
        self
    }

    #[must_use]
    pub fn not_optimized(mut self) -> Self {
        self.optimization_used = Some("0".to_string());
        self
    }

    #[must_use]
    pub fn code_format(mut self, code_format: CodeFormat) -> Self {
        self.code_format = code_format;
        self
    }

    #[must_use]
    pub fn evm_version(mut self, evm_version: impl Into<String>) -> Self {
        self.evm_version = Some(evm_version.into());
        self
    }

    #[must_use]
    pub fn constructor_arguments(
        mut self,
        constructor_arguments: Option<impl Into<String>>,
    ) -> Self {
        self.constructor_arguments = constructor_arguments.map(|s| {
            s.into()
                .trim()
                // TODO is this correct?
                .trim_start_matches("0x")
                .to_string()
        });
        self
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeFormat {
    #[serde(rename = "solidity-single-file")]
    SingleFile,

    #[default]
    #[serde(rename = "solidity-standard-json-input")]
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

/* -------------------------------------- ContractMetadata -------------------------------------- */

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum SourceCodeLanguage {
    #[default]
    Solidity,
    Vyper,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceCodeEntry {
    pub content: String,
}

impl From<&str> for SourceCodeEntry {
    fn from(s: &str) -> Self {
        Self { content: s.to_string() }
    }
}

impl From<String> for SourceCodeEntry {
    fn from(s: String) -> Self {
        Self { content: s }
    }
}

impl From<&String> for SourceCodeEntry {
    fn from(s: &String) -> Self {
        Self { content: s.clone() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceCodeMetadata {
    /// Programming language of the sources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<SourceCodeLanguage>,
    /// Source path => source code
    #[serde(default)]
    pub sources: HashMap<String, SourceCodeEntry>,
    /// Compiler settings, None if it's the language is not Solidity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<Settings>,
}

impl FromStr for SourceCodeMetadata {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut sources = HashMap::with_capacity(1);
        sources.insert("Contract".into(), s.into());
        Ok(Self { language: None, sources, settings: None })
    }
}

impl SourceCodeMetadata {
    pub fn source_code(&self) -> String {
        self.sources.values().map(|s| s.content.clone()).collect::<Vec<_>>().join("\n")
    }
}

/// Etherscan contract metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Metadata {
    /// Includes metadata for compiler settings and language.
    #[serde(deserialize_with = "deserialize_string_or_struct")]
    pub source_code: SourceCodeMetadata,

    /// The ABI of the contract.
    #[serde(rename = "ABI")]
    pub abi: String,

    /// The name of the contract.
    pub contract_name: String,

    /// The version that this contract was compiled with.
    #[serde(deserialize_with = "deserialize_version")]
    pub compiler_version: Version,

    /// Whether the optimizer was used. This value should only be 0 or 1.
    #[serde(deserialize_with = "deserialize_stringified_u64")]
    pub optimization_used: u64,

    /// The number of optimizations performed.
    #[serde(deserialize_with = "deserialize_stringified_u64")]
    pub runs: u64,

    /// The constructor arguments the contract was deployed with.
    pub constructor_arguments: Bytes,

    /// The version of the EVM the contract was deployed in. Can be either a variant of
    /// [EvmVersion] or "Default" which indicates the compiler's default.
    #[serde(rename = "EVMVersion")]
    pub evm_version: String,

    // ?
    pub library: String,

    /// The license of the contract.
    pub license_type: String,

    /// Whether this contract is a proxy. This value should only be 0 or 1.
    #[serde(deserialize_with = "deserialize_stringified_u64")]
    pub proxy: u64,

    /// If this contract is a proxy, the address of its implementation.
    #[serde(deserialize_with = "deserialize_address_opt")]
    pub implementation: Option<Address>,

    /// The swarm source of the contract.
    pub swarm_source: String,
}

impl Metadata {
    /// Creates a Solc [ProjectBuilder] with this contract's settings.
    pub fn project_builder(&self) -> ProjectBuilder {
        let mut settings = self.source_code.settings.clone().unwrap_or_default();

        if self.optimization_used == 1 && !settings.optimizer.enabled.unwrap_or_default() {
            settings.optimizer.enable();
            settings.optimizer.runs(self.runs as usize);
        }

        let evm_version = match self.evm_version.as_str() {
            "" | "Default" => {
                EvmVersion::normalize_version(Default::default(), &self.compiler_version)
            }
            _ => self.evm_version.parse().ok(),
        };
        settings.evm_version = evm_version;

        let solc_config = SolcConfig::builder().settings(settings).build();

        Project::builder().solc_config(solc_config)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContractMetadata {
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
    // /// All ABI from all contracts in the source file
    pub fn abis(&self) -> Result<Vec<Abi>> {
        self.items
            .iter()
            .map(|c| serde_json::from_str(&c.abi).map_err(EtherscanError::Serde))
            .collect()
    }

    // /// Combined source code of all contracts
    pub fn source_code(&self) -> String {
        self.items.iter().map(|c| c.source_code.source_code()).collect::<Vec<_>>().join("\n")
    }

    pub fn source_tree(&self) -> SourceTree {
        let mut entries = vec![];
        for item in self.items.iter() {
            let contract_root = Path::new(&item.contract_name);
            for (path, entry) in item.source_code.sources.iter() {
                let joined = contract_root.join(&path);
                entries.push(SourceTreeEntry { path: joined, contents: entry.content.clone() });
            }
        }
        SourceTree { entries }
    }
}

/* ------------------------------------------- Client ------------------------------------------- */

impl Client {
    /// Submit Source Code for Verification
    pub async fn submit_contract_verification(
        &self,
        contract: &VerifyContract,
    ) -> Result<Response<String>> {
        let body = self.create_query("contract", "verifysourcecode", contract);
        self.post_form(&body).await
    }

    /// Check Source Code Verification Status with receipt received from
    /// `[Self::submit_contract_verification]`
    pub async fn check_contract_verification_status(
        &self,
        guid: impl AsRef<str>,
    ) -> Result<Response<String>> {
        let body = self.create_query(
            "contract",
            "checkverifystatus",
            HashMap::from([("guid", guid.as_ref())]),
        );
        self.post_form(&body).await
    }

    /// Returns the contract ABI of a verified contract
    ///
    /// ```no_run
    /// # use ethers_etherscan::Client;
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let abi = client
    ///         .contract_abi("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
    ///         .await.unwrap();
    /// # }
    /// ```
    pub async fn contract_abi(&self, address: Address) -> Result<Abi> {
        // apply caching
        if let Some(ref cache) = self.cache {
            // If this is None, then we have a cache miss
            if let Some(src) = cache.get_abi(address) {
                // If this is None, then the contract is not verified
                return match src {
                    Some(src) => Ok(src),
                    None => Err(EtherscanError::ContractCodeNotVerified(address)),
                }
            }
        }

        let query = self.create_query("contract", "getabi", HashMap::from([("address", address)]));
        let resp: Response<String> = self.get_json(&query).await?;
        if resp.result.starts_with("Max rate limit reached") {
            return Err(EtherscanError::RateLimitExceeded)
        }
        if resp.result.starts_with("Contract source code not verified") {
            if let Some(ref cache) = self.cache {
                cache.set_abi(address, None);
            }
            return Err(EtherscanError::ContractCodeNotVerified(address))
        }
        let abi = serde_json::from_str(&resp.result)?;

        if let Some(ref cache) = self.cache {
            cache.set_abi(address, Some(&abi));
        }

        Ok(abi)
    }

    /// Get Contract Source Code for Verified Contract Source Codes
    /// ```no_run
    /// # use ethers_etherscan::Client;
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let meta = client
    ///         .contract_source_code("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
    ///         .await.unwrap();
    ///     let code = meta.source_code();
    /// # }
    /// ```
    pub async fn contract_source_code(&self, address: Address) -> Result<ContractMetadata> {
        // apply caching
        if let Some(ref cache) = self.cache {
            // If this is None, then we have a cache miss
            if let Some(src) = cache.get_source(address) {
                // If this is None, then the contract is not verified
                return match src {
                    Some(src) => Ok(src),
                    None => Err(EtherscanError::ContractCodeNotVerified(address)),
                }
            }
        }

        let query =
            self.create_query("contract", "getsourcecode", HashMap::from([("address", address)]));
        let mut res = self.get(&query).await?;

        // Source code is not verified
        if res.contains("Contract source code not verified") {
            if let Some(ref cache) = self.cache {
                cache.set_source(address, None);
            }
            return Err(EtherscanError::ContractCodeNotVerified(address))
        }

        // Etherscan can either return one raw string that includes all of the source code for a
        // verified contract or a [SourceCodeMetadata] struct surrounded in an extra set of {}.
        // {"SourceCode": "{{}}", ..} -> {"SourceCode": {}, ..}
        let start = r#""SourceCode":"{{"#;
        let end = r#"}}","ABI""#;
        if let Some(start_idx) = res.find(start) {
            // this should not fail
            let end_idx = res
                .find(end)
                .ok_or(EtherscanError::Unknown(format!("Malformed response {}", res)))?;
            // the SourceCode string value
            let range = start_idx + 13..end_idx + 3;
            let source_code = &res[range.clone()];
            // parse the escaped characters
            let parsed: String = serde_json::from_str(source_code)?;
            // skip the first `{` and last `}`
            let source_code = &parsed[1..parsed.len() - 1];
            // replace
            res.replace_range(range, source_code);
        }
        let res: Response<ContractMetadata> = self.sanitize_response(res)?;
        let res = res.result;

        if let Some(ref cache) = self.cache {
            cache.set_source(address, Some(&res));
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::{contract::VerifyContract, tests::run_at_least_duration, Client, EtherscanError};
    use ethers_core::types::Chain;
    use ethers_solc::{Project, ProjectPathsConfig};
    use serial_test::serial;
    use std::{path::PathBuf, time::Duration};

    #[allow(unused)]
    fn init_tracing() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn can_fetch_ftm_contract_abi() {
        init_tracing();
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Fantom).unwrap();

            let _abi = client
                .contract_abi("0x80AA7cb0006d5DDD91cce684229Ac6e398864606".parse().unwrap())
                .await
                .unwrap();
        })
        .await;
    }

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn can_fetch_contract_abi() {
        init_tracing();
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let _abi = client
                .contract_abi("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
                .await
                .unwrap();
        })
        .await;
    }

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn can_fetch_contract_source_code() {
        init_tracing();
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let meta = client
                .contract_source_code("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
                .await
                .unwrap();
            assert_eq!(meta.items[0].source_code.sources.len(), 1);
        })
        .await
    }

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn can_get_error_on_unverified_contract() {
        init_tracing();
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();
            let addr = "0xb5c31a0e22cae98ac08233e512bd627885aa24e5".parse().unwrap();
            let err = client.contract_source_code(addr).await.unwrap_err();
            assert!(matches!(err, EtherscanError::ContractCodeNotVerified(_)));
        })
        .await
    }

    /// Query a contract that has a single string source entry instead of underlying JSON metadata.
    #[tokio::test]
    #[serial]
    #[ignore]
    async fn can_fetch_contract_source_tree_for_singleton_contract() {
        init_tracing();
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let meta = client
                .contract_source_code("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
                .await
                .unwrap();

            let source_tree = meta.source_tree();
            assert_eq!(source_tree.entries.len(), 1);
        })
        .await
    }

    /// Query a contract that has many source entries as JSON metadata and ensure they are
    /// reflected.
    #[tokio::test]
    #[serial]
    #[ignore]
    async fn can_fetch_contract_source_tree_for_multi_entry_contract() {
        init_tracing();
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let meta = client
                .contract_source_code("0x8d04a8c79cEB0889Bdd12acdF3Fa9D207eD3Ff63".parse().unwrap())
                .await
                .unwrap();

            let source_tree = meta.source_tree();
            assert_eq!(source_tree.entries.len(), 15);
        })
        .await
    }

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn can_flatten_and_verify_contract() {
        init_tracing();
        run_at_least_duration(Duration::from_millis(250), async {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources");
            let paths = ProjectPathsConfig::builder()
                .sources(&root)
                .build()
                .expect("failed to resolve project paths");
            let project = Project::builder()
                .paths(paths)
                .build()
                .expect("failed to build the project");

            let address = "0x9e744c9115b74834c0f33f4097f40c02a9ac5c33".parse().unwrap();
            let compiler_version = "v0.5.17+commit.d19bba13";
            let constructor_args = "0x000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000005f5e1000000000000000000000000000000000000000000000000000000000000000007596179537761700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000035941590000000000000000000000000000000000000000000000000000000000";
            let contract = project.flatten(&root.join("UniswapExchange.sol")).expect("failed to flatten contract");
            let contract_name = "UniswapExchange".to_owned();

            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let contract =
                VerifyContract::new(address, contract_name, contract, compiler_version.to_string())
                    .constructor_arguments(Some(constructor_args))
                    .optimization(true)
                    .runs(200);
            let resp = client.submit_contract_verification(&contract).await.expect("failed to send the request");
            assert_ne!(resp.result, "Error!"); // `Error!` result means that request was malformatted
        })
        .await
    }
}
