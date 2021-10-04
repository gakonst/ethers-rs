use std::{collections::HashMap, fmt, io::BufRead, path::PathBuf, process::Command, str::FromStr};

use glob::glob;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::{abi::Abi, types::Bytes};
use once_cell::sync::Lazy;
use semver::Version;

/// The name of the `solc` binary on the system
const SOLC: &str = "solc";

/// Support for configuring the EVM version
/// https://blog.soliditylang.org/2018/03/08/solidity-0.4.21-release-announcement/
static CONSTANTINOPLE_SOLC: Lazy<Version> = Lazy::new(|| Version::from_str("0.4.21").unwrap());

/// Petersburg support
/// https://blog.soliditylang.org/2019/03/05/solidity-0.5.5-release-announcement/
static PETERSBURG_SOLC: Lazy<Version> = Lazy::new(|| Version::from_str("0.5.5").unwrap());

/// Istanbul support
/// https://blog.soliditylang.org/2019/12/09/solidity-0.5.14-release-announcement/
static ISTANBUL_SOLC: Lazy<Version> = Lazy::new(|| Version::from_str("0.5.14").unwrap());

/// Berlin support
/// https://blog.soliditylang.org/2021/06/10/solidity-0.8.5-release-announcement/
static BERLIN_SOLC: Lazy<Version> = Lazy::new(|| Version::from_str("0.8.5").unwrap());

/// London support
/// https://blog.soliditylang.org/2021/08/11/solidity-0.8.7-release-announcement/
static LONDON_SOLC: Lazy<Version> = Lazy::new(|| Version::from_str("0.8.7").unwrap());

type Result<T> = std::result::Result<T, SolcError>;

#[derive(Debug, Error)]
pub enum SolcError {
    /// Internal solc error
    #[error("Solc Error: {0}")]
    SolcError(String),
    #[error(transparent)]
    SemverError(#[from] semver::Error),
    /// Deserialization error
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// The result of a solc compilation
pub struct CompiledContract {
    /// The contract's ABI
    pub abi: Abi,
    /// The contract's bytecode
    pub bytecode: Bytes,
    /// The contract's runtime bytecode
    pub runtime_bytecode: Bytes,
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
///     .optimizer(Some(200))
///     .build()?;
///
/// // this will return None if the specified contract did not exist in the compiled
/// // files
/// let contract = contracts.get("SimpleStorage").expect("contract not found");
/// # Ok(())
/// # }
/// ```
pub struct Solc {
    /// The path to the Solc binary
    pub solc_path: Option<PathBuf>,

    /// The path where contracts will be read from
    pub paths: Vec<String>,

    /// Number of optimizer runs. None for no optimization
    pub optimizer: Option<usize>,

    /// Evm Version
    pub evm_version: EvmVersion,

    /// Paths for importing other libraries
    pub allowed_paths: Vec<PathBuf>,

    /// Output a single json document containing the specified information.
    /// Default is `abi,bin,bin-runtime`
    pub combined_json: Option<String>,

    /// Additional arguments to pass to solc
    pub args: Vec<String>,
}

impl Solc {
    /// Instantiates The Solc builder with the provided glob of Solidity files
    pub fn new(path: &str) -> Self {
        // Convert the glob to a vector of string paths
        // TODO: This might not be the most robust way to do this
        let paths = glob(path)
            .expect("could not get glob")
            .map(|path| path.expect("path not found").to_string_lossy().to_string())
            .collect::<Vec<String>>();
        Self::new_with_paths(paths)
    }

    /// Instantiates the Solc builder for the provided paths
    pub fn new_with_paths(paths: Vec<String>) -> Self {
        Self {
            paths,
            solc_path: None,
            optimizer: Some(200), // default optimizer runs = 200
            evm_version: EvmVersion::Istanbul,
            allowed_paths: Vec::new(),
            combined_json: Some("abi,bin,bin-runtime".to_string()),
            args: Vec::new(),
        }
    }

    /// Gets the complete solc output as json object
    pub fn exec(self) -> Result<serde_json::Value> {
        let path = self.solc_path.unwrap_or_else(|| PathBuf::from(SOLC));

        let mut command = Command::new(&path);
        let version = Solc::version(Some(path));

        if let Some(combined_json) = self.combined_json {
            command.arg("--combined-json").arg(combined_json);
        }

        if let Some(evm_version) = normalize_evm_version(&version, self.evm_version) {
            command.arg("--evm-version").arg(evm_version.to_string());
        }

        if let Some(runs) = self.optimizer {
            command
                .arg("--optimize")
                .arg("--optimize-runs")
                .arg(runs.to_string());
        }

        command.args(self.args);

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
        Ok(serde_json::from_slice(&command.stdout)?)
    }

    /// Gets the ABI for the contracts
    pub fn build_raw(self) -> Result<HashMap<String, CompiledContractStr>> {
        let mut output = self.exec()?;
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

                let runtime_bin =
                    if let serde_json::Value::String(bin) = contract["bin-runtime"].take() {
                        bin
                    } else {
                        panic!("no runtime bytecode found")
                    };
                contracts.insert(
                    name,
                    CompiledContractStr {
                        abi,
                        bin,
                        runtime_bin,
                    },
                );
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

                // parse the runtime bytecode
                let runtime_bytecode = hex::decode(contract.runtime_bin)
                    .expect("solc did not produce valid runtime-bytecode")
                    .into();
                (
                    name,
                    CompiledContract {
                        abi,
                        bytecode,
                        runtime_bytecode,
                    },
                )
            })
            .collect::<HashMap<String, CompiledContract>>();

        Ok(contracts)
    }

    /// Returns the output of `solc --version`
    ///
    /// # Panics
    ///
    /// If `solc` is not found
    pub fn version(solc_path: Option<PathBuf>) -> Version {
        let solc_path = solc_path.unwrap_or_else(|| PathBuf::from(SOLC));
        let command_output = Command::new(&solc_path)
            .arg("--version")
            .output()
            .unwrap_or_else(|_| panic!("`{:?}` not found", solc_path));

        let version = command_output
            .stdout
            .lines()
            .last()
            .expect("expected version in solc output")
            .expect("could not get solc version");

        // Return the version trimmed
        let version = version.replace("Version: ", "");
        Version::from_str(&version[0..5]).expect("not a version")
    }

    /// Sets the EVM version for compilation
    pub fn evm_version(mut self, version: EvmVersion) -> Self {
        self.evm_version = version;
        self
    }

    /// Sets the path to the solc binary
    pub fn solc_path(mut self, path: PathBuf) -> Self {
        self.solc_path = Some(std::fs::canonicalize(path).unwrap());
        self
    }

    /// Sets the `combined-json` option, by default this is set to `abi,bin,bin-runtime`
    /// NOTE: In order to get the `CompiledContract` from `Self::build`, this _must_ contain `abi,bin`.
    pub fn combined_json(mut self, combined_json: impl Into<String>) -> Self {
        self.combined_json = Some(combined_json.into());
        self
    }

    /// Sets the optimizer runs (default = 200). None indicates no optimization
    ///
    /// ```rust,no_run
    /// use ethers_core::utils::Solc;
    ///
    /// // No optimization
    /// let contracts = Solc::new("./contracts/*")
    ///     .optimizer(None)
    ///     .build().unwrap();
    ///
    /// // Some(200) is default, optimizer on with 200 runs
    /// // .arg() allows passing arbitrary args to solc command
    /// let optimized_contracts = Solc::new("./contracts/*")
    ///     .optimizer(Some(200))
    ///     .arg("--metadata-hash=none")
    ///     .build().unwrap();
    /// ```
    pub fn optimizer(mut self, runs: Option<usize>) -> Self {
        self.optimizer = runs;
        self
    }

    /// Sets the allowed paths for using files from outside the same directory
    // TODO: Test this
    pub fn allowed_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.allowed_paths = paths;
        self
    }

    /// Adds an argument to pass to solc
    pub fn arg<T: Into<String>>(mut self, arg: T) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Adds multiple arguments to pass to solc
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for arg in args {
            self = self.arg(arg);
        }
        self
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvmVersion {
    Homestead,
    TangerineWhistle,
    SpuriusDragon,
    Constantinople,
    Petersburg,
    Istanbul,
    Berlin,
    London,
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
            EvmVersion::London => "london",
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
    /// The contract's runtime bytecode in hex
    pub runtime_bin: String,
}

fn normalize_evm_version(version: &Version, evm_version: EvmVersion) -> Option<EvmVersion> {
    // the EVM version flag was only added at 0.4.21
    // we work our way backwards
    if version >= &CONSTANTINOPLE_SOLC {
        // If the Solc is at least at london, it supports all EVM versions
        Some(if version >= &LONDON_SOLC {
            evm_version
        // For all other cases, cap at the at-the-time highest possible fork
        } else if version >= &BERLIN_SOLC && evm_version >= EvmVersion::Berlin {
            EvmVersion::Berlin
        } else if version >= &ISTANBUL_SOLC && evm_version >= EvmVersion::Istanbul {
            EvmVersion::Istanbul
        } else if version >= &PETERSBURG_SOLC && evm_version >= EvmVersion::Petersburg {
            EvmVersion::Petersburg
        } else if evm_version >= EvmVersion::Constantinople {
            EvmVersion::Constantinople
        } else {
            evm_version
        })
    } else {
        None
    }
}

/// General `solc` contract output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub abi: Abi,
    pub evm: Evm,
    #[serde(
        deserialize_with = "de_from_json_opt",
        serialize_with = "ser_to_inner_json",
        skip_serializing_if = "Option::is_none"
    )]
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evm {
    pub bytecode: Bytecode,
    pub deployed_bytecode: Bytecode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bytecode {
    #[serde(deserialize_with = "deserialize_bytes")]
    pub object: Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub compiler: Compiler,
    pub language: String,
    pub output: Output,
    pub settings: Settings,
    pub sources: Sources,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compiler {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub abi: Vec<SolcAbi>,
    pub devdoc: Option<Doc>,
    pub userdoc: Option<Doc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolcAbi {
    pub inputs: Vec<Item>,
    #[serde(rename = "stateMutability")]
    pub state_mutability: Option<String>,
    #[serde(rename = "type")]
    pub abi_type: String,
    pub name: Option<String>,
    pub outputs: Option<Vec<Item>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "internalType")]
    pub internal_type: String,
    pub name: String,
    #[serde(rename = "type")]
    pub put_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doc {
    pub kind: String,
    pub methods: Libraries,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Libraries {
    #[serde(flatten)]
    pub libs: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(rename = "compilationTarget")]
    pub compilation_target: CompilationTarget,
    #[serde(rename = "evmVersion")]
    pub evm_version: String,
    pub libraries: Libraries,
    pub metadata: MetadataClass,
    pub optimizer: Optimizer,
    pub remappings: Vec<Option<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationTarget {
    #[serde(flatten)]
    pub inner: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataClass {
    #[serde(rename = "bytecodeHash")]
    pub bytecode_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Optimizer {
    pub enabled: bool,
    pub runs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sources {
    #[serde(flatten)]
    pub inner: HashMap<String, serde_json::Value>,
}

pub fn deserialize_bytes<'de, D>(d: D) -> std::result::Result<Bytes, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(d)?;

    Ok(hex::decode(&value)
        .map_err(|e| serde::de::Error::custom(e.to_string()))?
        .into())
}

fn de_from_json_opt<'de, D, T>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    if let Some(val) = <Option<String>>::deserialize(deserializer)? {
        serde_json::from_str(&val).map_err(serde::de::Error::custom)
    } else {
        Ok(None)
    }
}

fn ser_to_inner_json<S, T>(val: &T, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    let val = serde_json::to_string(val).map_err(serde::ser::Error::custom)?;
    s.serialize_str(&val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solc_version() {
        Solc::version(None);
    }

    #[test]
    fn test_evm_version_normalization() {
        for (solc_version, evm_version, expected) in &[
            // Ensure 0.4.21 it always returns None
            ("0.4.20", EvmVersion::Homestead, None),
            // Constantinople clipping
            ("0.4.21", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            (
                "0.4.21",
                EvmVersion::Constantinople,
                Some(EvmVersion::Constantinople),
            ),
            (
                "0.4.21",
                EvmVersion::London,
                Some(EvmVersion::Constantinople),
            ),
            // Petersburg
            ("0.5.5", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            (
                "0.5.5",
                EvmVersion::Petersburg,
                Some(EvmVersion::Petersburg),
            ),
            ("0.5.5", EvmVersion::London, Some(EvmVersion::Petersburg)),
            // Istanbul
            ("0.5.14", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            ("0.5.14", EvmVersion::Istanbul, Some(EvmVersion::Istanbul)),
            ("0.5.14", EvmVersion::London, Some(EvmVersion::Istanbul)),
            // Berlin
            ("0.8.5", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            ("0.8.5", EvmVersion::Berlin, Some(EvmVersion::Berlin)),
            ("0.8.5", EvmVersion::London, Some(EvmVersion::Berlin)),
            // London
            ("0.8.7", EvmVersion::Homestead, Some(EvmVersion::Homestead)),
            ("0.8.7", EvmVersion::London, Some(EvmVersion::London)),
            ("0.8.7", EvmVersion::London, Some(EvmVersion::London)),
        ] {
            assert_eq!(
                &normalize_evm_version(&Version::from_str(solc_version).unwrap(), *evm_version),
                expected
            )
        }
    }
}
