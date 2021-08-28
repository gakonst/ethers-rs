use std::{collections::HashMap, fmt, io::BufRead, path::PathBuf, process::Command};

use glob::glob;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{abi::Abi, types::Bytes};

/// The name of the `solc` binary on the system
const SOLC: &str = "solc";

type Result<T> = std::result::Result<T, SolcError>;

#[derive(Debug, Error)]
pub enum SolcError {
    /// Internal solc error
    #[error("Solc Error: {0}")]
    SolcError(String),
    /// Deserialization error
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(Clone, Debug)]
/// The result of a solc compilation
pub struct CompiledContract {
    /// The contract's ABI
    pub abi: Abi,
    /// The contract's bytecode
    pub bytecode: Bytes,
}

/// Solidity Compiler Bindings
///
/// Assumes that `solc` is installed and available in the caller's $PATH. Any calls
/// will **panic** otherwise.
///
/// By default, it uses 200 optimizer runs and Istanbul as the EVM version
///
/// # Examples
///
/// ```no_run
/// use ethers_core::utils::Solc;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Give it a glob
/// let contracts = Solc::new("./contracts/*")
///     .optimizer(200)
///     .build()?;
///
/// // this will return None if the specified contract did not exist in the compiled
/// // files
/// let contract = contracts.get("SimpleStorage").expect("contract not found");
/// # Ok(())
/// # }
/// ```
pub struct Solc {
    /// The path where contracts will be read from
    pub paths: Vec<String>,

    /// Number of runs
    pub optimizer: usize,

    /// Evm Version
    pub evm_version: EvmVersion,

    /// Paths for importing other libraries
    pub allowed_paths: Vec<PathBuf>,
}

impl Solc {
    /// Instantiates the Solc builder for the provided paths
    pub fn new(path: &str) -> Self {
        // Convert the glob to a vector of string paths
        // TODO: This might not be the most robust way to do this
        let paths = glob(path)
            .expect("could not get glob")
            .map(|path| path.expect("path not found").to_string_lossy().to_string())
            .collect::<Vec<String>>();

        Self {
            paths,
            optimizer: 200, // default optimizer runs = 200
            evm_version: EvmVersion::Istanbul,
            allowed_paths: Vec::new(),
        }
    }

    /// Gets the ABI for the contracts
    pub fn build_raw(self) -> Result<HashMap<String, CompiledContractStr>> {
        let mut command = Command::new(SOLC);

        command
            .arg("--evm-version")
            .arg(self.evm_version.to_string())
            .arg("--combined-json")
            .arg("abi,bin");

        for path in self.paths {
            command.arg(path);
        }

        let command = command.output().expect("could not run `solc`");

        if !command.status.success() {
            return Err(SolcError::SolcError(
                String::from_utf8_lossy(&command.stderr).to_string(),
            ));
        }

        // Deserialize the output
        let mut output: serde_json::Value = serde_json::from_slice(&command.stdout)?;
        let contract_values = output["contracts"].as_object_mut().ok_or_else(|| {
            SolcError::SolcError("no contracts found in `solc` output".to_string())
        })?;

        let mut contracts = HashMap::with_capacity(contract_values.len());

        for (name, contract) in contract_values {
            if let serde_json::Value::String(bin) = contract["bin"].take() {
                let name = name
                    .rsplit(':')
                    .next()
                    .expect("could not strip fname")
                    .to_owned();

                // abi could be an escaped string (solc<=0.7) or an array (solc>=0.8)
                let abi = match contract["abi"].take() {
                    serde_json::Value::String(abi) => abi,
                    val @ serde_json::Value::Array(_) => val.to_string(),
                    val => {
                        return Err(SolcError::SolcError(format!(
                            "Expected abi in solc output, found {:?}",
                            val
                        )))
                    }
                };
                contracts.insert(name, CompiledContractStr { abi, bin });
            } else {
                return Err(SolcError::SolcError(
                    "could not find `bin` in solc output".to_string(),
                ));
            }
        }

        Ok(contracts)
    }

    /// Builds the contracts and returns a hashmap for each named contract
    pub fn build(self) -> Result<HashMap<String, CompiledContract>> {
        // Build, and then get the data in the correct format
        let contracts = self
            .build_raw()?
            .into_iter()
            .map(|(name, contract)| {
                // parse the ABI
                let abi = serde_json::from_str(&contract.abi)
                    .expect("could not parse `solc` abi, this should never happen");

                // parse the bytecode
                let bytecode = hex::decode(contract.bin)
                    .expect("solc did not produce valid bytecode")
                    .into();
                (name, CompiledContract { abi, bytecode })
            })
            .collect::<HashMap<String, CompiledContract>>();

        Ok(contracts)
    }

    /// Returns the output of `solc --version`
    ///
    /// # Panics
    ///
    /// If `solc` is not in the user's $PATH
    pub fn version() -> String {
        let command_output = Command::new(SOLC)
            .arg("--version")
            .output()
            .unwrap_or_else(|_| panic!("`{}` not in user's $PATH", SOLC));

        let version = command_output
            .stdout
            .lines()
            .last()
            .expect("expected version in solc output")
            .expect("could not get solc version");

        // Return the version trimmed
        version.replace("Version: ", "")
    }

    /// Sets the EVM version for compilation
    pub fn evm_version(mut self, version: EvmVersion) -> Self {
        self.evm_version = version;
        self
    }

    /// Sets the optimizer runs (default = 200)
    pub fn optimizer(mut self, runs: usize) -> Self {
        self.optimizer = runs;
        self
    }

    /// Sets the allowed paths for using files from outside the same directory
    // TODO: Test this
    pub fn allowed_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.allowed_paths = paths;
        self
    }
}

#[derive(Clone, Debug)]
pub enum EvmVersion {
    Homestead,
    TangerineWhistle,
    SpuriusDragon,
    Constantinople,
    Petersburg,
    Istanbul,
    Berlin,
}

impl fmt::Display for EvmVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            EvmVersion::Homestead => "homestead",
            EvmVersion::TangerineWhistle => "tangerineWhistle",
            EvmVersion::SpuriusDragon => "spuriusDragon",
            EvmVersion::Constantinople => "constantinople",
            EvmVersion::Petersburg => "petersburg",
            EvmVersion::Istanbul => "istanbul",
            EvmVersion::Berlin => "berlin",
        };
        write!(f, "{}", string)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
// Helper struct for deserializing the solc string outputs
struct SolcOutput {
    contracts: HashMap<String, CompiledContractStr>,
    version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Helper struct for deserializing the solc string outputs
pub struct CompiledContractStr {
    /// The contract's raw ABI
    pub abi: String,
    /// The contract's bytecode in hex
    pub bin: String,
}
