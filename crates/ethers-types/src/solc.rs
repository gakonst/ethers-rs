//! Solidity Compiler Bindings
//!
//! Assumes that `solc` is installed and available in the caller's $PATH. Any calls
//! will fail otherwise.
//!
//! # Examples
//!
//! ```rust,ignore
//! // Give it a glob
//! let contracts = Solc::new("./contracts/*")
//!     .optimizer(200)
//!     .build();
//! let contract = contracts.get("SimpleStorage").unwrap();
//! ```
use crate::{abi::Abi, Bytes};
use glob::glob;
use rustc_hex::FromHex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, io::BufRead, path::PathBuf, process::Command};
use thiserror::Error;

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

/// Solc builder
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

    /// Builds the contracts and returns a hashmap for each named contract
    pub fn build(self) -> Result<HashMap<String, CompiledContract>> {
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
        let output: SolcOutput = serde_json::from_slice(&command.stdout)?;

        // Get the data in the correct format
        let contracts = output
            .contracts
            .into_iter()
            .map(|(name, contract)| {
                let abi = serde_json::from_str(&contract.abi)
                    .expect("could not parse `solc` abi, this should never happen");

                let bytecode = contract
                    .bin
                    .from_hex::<Vec<u8>>()
                    .expect("solc did not produce valid bytecode")
                    .into();

                let name = name
                    .rsplit(":")
                    .next()
                    .expect("could not strip fname")
                    .to_owned();
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
            .expect(&format!("`{}` not in user's $PATH", SOLC));

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
    pub fn optimizer_runs(mut self, runs: usize) -> Self {
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
// Helper struct for deserializing the solc string outputs
struct CompiledContractStr {
    abi: String,
    bin: String,
}
