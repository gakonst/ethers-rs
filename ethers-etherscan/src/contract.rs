use crate::{
    source_tree::{SourceTree, SourceTreeEntry},
    utils::{deserialize_address_opt, deserialize_stringified_source_code},
    Client, EtherscanError, Response, Result,
};
use ethers_core::{
    abi::{Abi, Address, RawAbi},
    types::{serde_helpers::deserialize_stringified_u64, Bytes},
};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[cfg(feature = "ethers-solc")]
use ethers_solc::{artifacts::Settings, EvmVersion, Project, ProjectBuilder, SolcConfig};

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

impl<T: Into<String>> From<T> for SourceCodeEntry {
    fn from(s: T) -> Self {
        Self { content: s.into() }
    }
}

/// The contract metadata's SourceCode field.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SourceCodeMetadata {
    /// Contains metadata and path mapped source code.
    Metadata {
        /// Programming language of the sources.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        language: Option<SourceCodeLanguage>,
        /// Source path => source code
        #[serde(default)]
        sources: HashMap<String, SourceCodeEntry>,
        /// Compiler settings, None if the language is not Solidity.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        settings: Option<serde_json::Value>,
    },
    /// Contains only the source code.
    SourceCode(String),
}

impl SourceCodeMetadata {
    pub fn source_code(&self) -> String {
        match self {
            Self::Metadata { sources, .. } => {
                sources.values().map(|s| s.content.clone()).collect::<Vec<_>>().join("\n")
            }
            Self::SourceCode(s) => s.clone(),
        }
    }

    pub fn language(&self) -> Option<SourceCodeLanguage> {
        match self {
            Self::Metadata { language, .. } => language.clone(),
            Self::SourceCode(_) => None,
        }
    }

    pub fn sources(&self) -> HashMap<String, SourceCodeEntry> {
        match self {
            Self::Metadata { sources, .. } => sources.clone(),
            Self::SourceCode(s) => HashMap::from([("Contract".into(), s.into())]),
        }
    }

    #[cfg(feature = "ethers-solc")]
    pub fn settings(&self) -> Result<Option<Settings>> {
        match self {
            Self::Metadata { settings, .. } => match settings {
                Some(value) => {
                    if value.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(serde_json::from_value(value.to_owned())?))
                    }
                }
                None => Ok(None),
            },
            Self::SourceCode(_) => Ok(None),
        }
    }

    #[cfg(not(feature = "ethers-solc"))]
    pub fn settings(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Metadata { settings, .. } => settings.as_ref(),
            Self::SourceCode(_) => None,
        }
    }
}

/// Etherscan contract metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Metadata {
    /// Includes metadata for compiler settings and language.
    #[serde(deserialize_with = "deserialize_stringified_source_code")]
    pub source_code: SourceCodeMetadata,

    /// The ABI of the contract.
    #[serde(rename = "ABI")]
    pub abi: String,

    /// The name of the contract.
    pub contract_name: String,

    /// The version that this contract was compiled with. If it is a Vyper contract, it will start
    /// with "vyper:".
    pub compiler_version: String,

    /// Whether the optimizer was used. This value should only be 0 or 1.
    #[serde(deserialize_with = "deserialize_stringified_u64")]
    pub optimization_used: u64,

    /// The number of optimizations performed.
    #[serde(deserialize_with = "deserialize_stringified_u64")]
    pub runs: u64,

    /// The constructor arguments the contract was deployed with.
    pub constructor_arguments: Bytes,

    /// The version of the EVM the contract was deployed in. Can be either a variant of EvmVersion
    /// or "Default" which indicates the compiler's default.
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
    /// Returns the contract's source code.
    pub fn source_code(&self) -> String {
        self.source_code.source_code()
    }

    /// Returns the contract's programming language.
    pub fn language(&self) -> SourceCodeLanguage {
        self.source_code.language().unwrap_or_else(|| {
            if self.is_vyper() {
                SourceCodeLanguage::Vyper
            } else {
                SourceCodeLanguage::Solidity
            }
        })
    }

    /// Returns the contract's path mapped source code.
    pub fn sources(&self) -> HashMap<String, SourceCodeEntry> {
        self.source_code.sources()
    }

    /// Parses the Abi String as an [RawAbi] struct.
    pub fn raw_abi(&self) -> Result<RawAbi> {
        Ok(serde_json::from_str(&self.abi)?)
    }

    /// Parses the Abi String as an [Abi] struct.
    pub fn abi(&self) -> Result<Abi> {
        Ok(serde_json::from_str(&self.abi)?)
    }

    /// Parses the compiler version.
    pub fn compiler_version(&self) -> Result<Version> {
        let v = &self.compiler_version;
        let v = v.strip_prefix("vyper:").unwrap_or(v);
        let v = v.strip_prefix('v').unwrap_or(v);
        match v.parse() {
            Err(e) => {
                let v = v.replace('a', "-alpha.");
                let v = v.replace('b', "-beta.");
                v.parse().map_err(|_| EtherscanError::Unknown(format!("bad compiler version: {e}")))
            }
            Ok(v) => Ok(v),
        }
    }

    /// Returns whether this contract is a Vyper or a Solidity contract.
    pub fn is_vyper(&self) -> bool {
        self.compiler_version.starts_with("vyper:")
    }

    /// Maps this contract's sources to a [SourceTreeEntry] vector.
    pub fn source_entries(&self) -> Vec<SourceTreeEntry> {
        let root = Path::new(&self.contract_name);
        self.sources()
            .into_iter()
            .map(|(path, entry)| {
                let path = root.join(path);
                SourceTreeEntry { path, contents: entry.content }
            })
            .collect()
    }

    /// Returns the source tree of this contract's sources.
    pub fn source_tree(&self) -> SourceTree {
        SourceTree { entries: self.source_entries() }
    }

    /// Returns the contract's compiler settings.
    #[cfg(feature = "ethers-solc")]
    pub fn settings(&self) -> Result<Settings> {
        let mut settings = self.source_code.settings()?.unwrap_or_default();

        if self.optimization_used == 1 && !settings.optimizer.enabled.unwrap_or_default() {
            settings.optimizer.enable();
            settings.optimizer.runs(self.runs as usize);
        }

        settings.evm_version = self.evm_version()?;

        Ok(settings)
    }

    /// Creates a Solc [ProjectBuilder] with this contract's settings.
    #[cfg(feature = "ethers-solc")]
    pub fn project_builder(&self) -> Result<ProjectBuilder> {
        let solc_config = SolcConfig::builder().settings(self.settings()?).build();

        Ok(Project::builder().solc_config(solc_config))
    }

    /// Parses the EVM version.
    #[cfg(feature = "ethers-solc")]
    pub fn evm_version(&self) -> Result<Option<EvmVersion>> {
        match self.evm_version.as_str() {
            "" | "Default" => {
                Ok(EvmVersion::default().normalize_version(&self.compiler_version()?))
            }
            _ => {
                let evm_version = self
                    .evm_version
                    .parse()
                    .map_err(|e| EtherscanError::Unknown(format!("bad evm version: {e}")))?;
                Ok(Some(evm_version))
            }
        }
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
    /// Returns the ABI of all contracts.
    pub fn abis(&self) -> Result<Vec<Abi>> {
        self.items.iter().map(|c| c.abi()).collect()
    }

    /// Returns the raw ABI of all contracts.
    pub fn raw_abis(&self) -> Result<Vec<RawAbi>> {
        self.items.iter().map(|c| c.raw_abi()).collect()
    }

    /// Returns the combined source code of all contracts.
    pub fn source_code(&self) -> String {
        self.items.iter().map(|c| c.source_code()).collect::<Vec<_>>().join("\n")
    }

    /// Returns the combined [SourceTree] of all contracts.
    pub fn source_tree(&self) -> SourceTree {
        SourceTree { entries: self.items.iter().flat_map(|item| item.source_entries()).collect() }
    }
}

impl Client {
    /// Fetches a verified contract's ABI.
    ///
    /// # Example
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

    /// Fetches a contract's verified source code and its metadata.
    ///
    /// # Example
    ///
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
        let response = self.get(&query).await?;

        // Source code is not verified
        if response.contains("Contract source code not verified") {
            if let Some(ref cache) = self.cache {
                cache.set_source(address, None);
            }
            return Err(EtherscanError::ContractCodeNotVerified(address))
        }

        let response: Response<ContractMetadata> = self.sanitize_response(response)?;
        let result = response.result;

        if let Some(ref cache) = self.cache {
            cache.set_source(address, Some(&result));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::run_at_least_duration;
    use ethers_core::types::Chain;
    use serial_test::serial;
    use std::time::Duration;

    /// Abi of [0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413](https://api.etherscan.io/api?module=contract&action=getsourcecode&address=0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413).
    const DAO_ABI: &str = "[{\"constant\":true,\"inputs\":[{\"name\":\"\",\"type\":\"uint256\"}],\"name\":\"proposals\",\"outputs\":[{\"name\":\"recipient\",\"type\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\"},{\"name\":\"description\",\"type\":\"string\"},{\"name\":\"votingDeadline\",\"type\":\"uint256\"},{\"name\":\"open\",\"type\":\"bool\"},{\"name\":\"proposalPassed\",\"type\":\"bool\"},{\"name\":\"proposalHash\",\"type\":\"bytes32\"},{\"name\":\"proposalDeposit\",\"type\":\"uint256\"},{\"name\":\"newCurator\",\"type\":\"bool\"},{\"name\":\"yea\",\"type\":\"uint256\"},{\"name\":\"nay\",\"type\":\"uint256\"},{\"name\":\"creator\",\"type\":\"address\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_spender\",\"type\":\"address\"},{\"name\":\"_amount\",\"type\":\"uint256\"}],\"name\":\"approve\",\"outputs\":[{\"name\":\"success\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"minTokensToCreate\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"rewardAccount\",\"outputs\":[{\"name\":\"\",\"type\":\"address\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"daoCreator\",\"outputs\":[{\"name\":\"\",\"type\":\"address\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"totalSupply\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"divisor\",\"outputs\":[{\"name\":\"divisor\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"extraBalance\",\"outputs\":[{\"name\":\"\",\"type\":\"address\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_proposalID\",\"type\":\"uint256\"},{\"name\":\"_transactionData\",\"type\":\"bytes\"}],\"name\":\"executeProposal\",\"outputs\":[{\"name\":\"_success\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_from\",\"type\":\"address\"},{\"name\":\"_to\",\"type\":\"address\"},{\"name\":\"_value\",\"type\":\"uint256\"}],\"name\":\"transferFrom\",\"outputs\":[{\"name\":\"success\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[],\"name\":\"unblockMe\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"totalRewardToken\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"actualBalance\",\"outputs\":[{\"name\":\"_actualBalance\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"closingTime\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"name\":\"\",\"type\":\"address\"}],\"name\":\"allowedRecipients\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_to\",\"type\":\"address\"},{\"name\":\"_value\",\"type\":\"uint256\"}],\"name\":\"transferWithoutReward\",\"outputs\":[{\"name\":\"success\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[],\"name\":\"refund\",\"outputs\":[],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_recipient\",\"type\":\"address\"},{\"name\":\"_amount\",\"type\":\"uint256\"},{\"name\":\"_description\",\"type\":\"string\"},{\"name\":\"_transactionData\",\"type\":\"bytes\"},{\"name\":\"_debatingPeriod\",\"type\":\"uint256\"},{\"name\":\"_newCurator\",\"type\":\"bool\"}],\"name\":\"newProposal\",\"outputs\":[{\"name\":\"_proposalID\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"name\":\"\",\"type\":\"address\"}],\"name\":\"DAOpaidOut\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"minQuorumDivisor\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_newContract\",\"type\":\"address\"}],\"name\":\"newContract\",\"outputs\":[],\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"name\":\"_owner\",\"type\":\"address\"}],\"name\":\"balanceOf\",\"outputs\":[{\"name\":\"balance\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_recipient\",\"type\":\"address\"},{\"name\":\"_allowed\",\"type\":\"bool\"}],\"name\":\"changeAllowedRecipients\",\"outputs\":[{\"name\":\"_success\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[],\"name\":\"halveMinQuorum\",\"outputs\":[{\"name\":\"_success\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"name\":\"\",\"type\":\"address\"}],\"name\":\"paidOut\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_proposalID\",\"type\":\"uint256\"},{\"name\":\"_newCurator\",\"type\":\"address\"}],\"name\":\"splitDAO\",\"outputs\":[{\"name\":\"_success\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"DAOrewardAccount\",\"outputs\":[{\"name\":\"\",\"type\":\"address\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"proposalDeposit\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"numberOfProposals\",\"outputs\":[{\"name\":\"_numberOfProposals\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"lastTimeMinQuorumMet\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_toMembers\",\"type\":\"bool\"}],\"name\":\"retrieveDAOReward\",\"outputs\":[{\"name\":\"_success\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[],\"name\":\"receiveEther\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_to\",\"type\":\"address\"},{\"name\":\"_value\",\"type\":\"uint256\"}],\"name\":\"transfer\",\"outputs\":[{\"name\":\"success\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"isFueled\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_tokenHolder\",\"type\":\"address\"}],\"name\":\"createTokenProxy\",\"outputs\":[{\"name\":\"success\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"name\":\"_proposalID\",\"type\":\"uint256\"}],\"name\":\"getNewDAOAddress\",\"outputs\":[{\"name\":\"_newDAO\",\"type\":\"address\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_proposalID\",\"type\":\"uint256\"},{\"name\":\"_supportsProposal\",\"type\":\"bool\"}],\"name\":\"vote\",\"outputs\":[{\"name\":\"_voteID\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[],\"name\":\"getMyReward\",\"outputs\":[{\"name\":\"_success\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"name\":\"\",\"type\":\"address\"}],\"name\":\"rewardToken\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_from\",\"type\":\"address\"},{\"name\":\"_to\",\"type\":\"address\"},{\"name\":\"_value\",\"type\":\"uint256\"}],\"name\":\"transferFromWithoutReward\",\"outputs\":[{\"name\":\"success\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"name\":\"_owner\",\"type\":\"address\"},{\"name\":\"_spender\",\"type\":\"address\"}],\"name\":\"allowance\",\"outputs\":[{\"name\":\"remaining\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"_proposalDeposit\",\"type\":\"uint256\"}],\"name\":\"changeProposalDeposit\",\"outputs\":[],\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"name\":\"\",\"type\":\"address\"}],\"name\":\"blocked\",\"outputs\":[{\"name\":\"\",\"type\":\"uint256\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"curator\",\"outputs\":[{\"name\":\"\",\"type\":\"address\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[{\"name\":\"_proposalID\",\"type\":\"uint256\"},{\"name\":\"_recipient\",\"type\":\"address\"},{\"name\":\"_amount\",\"type\":\"uint256\"},{\"name\":\"_transactionData\",\"type\":\"bytes\"}],\"name\":\"checkProposalCode\",\"outputs\":[{\"name\":\"_codeChecksOut\",\"type\":\"bool\"}],\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"privateCreation\",\"outputs\":[{\"name\":\"\",\"type\":\"address\"}],\"type\":\"function\"},{\"inputs\":[{\"name\":\"_curator\",\"type\":\"address\"},{\"name\":\"_daoCreator\",\"type\":\"address\"},{\"name\":\"_proposalDeposit\",\"type\":\"uint256\"},{\"name\":\"_minTokensToCreate\",\"type\":\"uint256\"},{\"name\":\"_closingTime\",\"type\":\"uint256\"},{\"name\":\"_privateCreation\",\"type\":\"address\"}],\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"_from\",\"type\":\"address\"},{\"indexed\":true,\"name\":\"_to\",\"type\":\"address\"},{\"indexed\":false,\"name\":\"_amount\",\"type\":\"uint256\"}],\"name\":\"Transfer\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"_owner\",\"type\":\"address\"},{\"indexed\":true,\"name\":\"_spender\",\"type\":\"address\"},{\"indexed\":false,\"name\":\"_amount\",\"type\":\"uint256\"}],\"name\":\"Approval\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"FuelingToDate\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"to\",\"type\":\"address\"},{\"indexed\":false,\"name\":\"amount\",\"type\":\"uint256\"}],\"name\":\"CreatedToken\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"to\",\"type\":\"address\"},{\"indexed\":false,\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"Refund\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"proposalID\",\"type\":\"uint256\"},{\"indexed\":false,\"name\":\"recipient\",\"type\":\"address\"},{\"indexed\":false,\"name\":\"amount\",\"type\":\"uint256\"},{\"indexed\":false,\"name\":\"newCurator\",\"type\":\"bool\"},{\"indexed\":false,\"name\":\"description\",\"type\":\"string\"}],\"name\":\"ProposalAdded\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"proposalID\",\"type\":\"uint256\"},{\"indexed\":false,\"name\":\"position\",\"type\":\"bool\"},{\"indexed\":true,\"name\":\"voter\",\"type\":\"address\"}],\"name\":\"Voted\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"proposalID\",\"type\":\"uint256\"},{\"indexed\":false,\"name\":\"result\",\"type\":\"bool\"},{\"indexed\":false,\"name\":\"quorum\",\"type\":\"uint256\"}],\"name\":\"ProposalTallied\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"_newCurator\",\"type\":\"address\"}],\"name\":\"NewCurator\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"_recipient\",\"type\":\"address\"},{\"indexed\":false,\"name\":\"_allowed\",\"type\":\"bool\"}],\"name\":\"AllowedRecipientChanged\",\"type\":\"event\"}]";

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

            let abi = client
                .contract_abi("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
                .await
                .unwrap();
            assert_eq!(abi, serde_json::from_str(DAO_ABI).unwrap());
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

            assert_eq!(meta.items.len(), 1);
            let item = &meta.items[0];
            assert!(matches!(item.source_code, SourceCodeMetadata::SourceCode(_)));
            assert_eq!(item.source_code.sources().len(), 1);
            assert_eq!(item.abi().unwrap(), serde_json::from_str(DAO_ABI).unwrap());
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

            assert_eq!(meta.items.len(), 1);
            let item = &meta.items[0];
            assert!(matches!(item.source_code, SourceCodeMetadata::SourceCode(_)));
            assert_eq!(item.source_code.sources().len(), 1);
            assert_eq!(item.abi().unwrap(), serde_json::from_str(DAO_ABI).unwrap());
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

            assert_eq!(meta.items.len(), 1);
            assert!(matches!(meta.items[0].source_code, SourceCodeMetadata::Metadata { .. }));
            let source_tree = meta.source_tree();
            assert_eq!(source_tree.entries.len(), 15);
        })
        .await
    }
}
