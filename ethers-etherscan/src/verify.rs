use crate::{Client, Response, Result};
use ethers_core::types::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        self.runs = Some(format!("{runs}"));
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tests::run_at_least_duration, Client};
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
