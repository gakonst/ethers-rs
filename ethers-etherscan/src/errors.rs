use ethers_core::types::{Address, Chain};
use std::env::VarError;

#[derive(Debug, thiserror::Error)]
pub enum EtherscanError {
    #[error("Chain {0} not supported")]
    ChainNotSupported(Chain),
    #[error("Contract execution call failed: {0}")]
    ExecutionFailed(String),
    #[error("Balance failed")]
    BalanceFailed,
    #[error("Transaction receipt failed")]
    TransactionReceiptFailed,
    #[error("Gas estimation failed")]
    GasEstimationFailed,
    #[error("Bad status code: {0}")]
    BadStatusCode(String),
    #[error(transparent)]
    EnvVarNotFound(#[from] VarError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("Contract source code not verified: {0}")]
    ContractCodeNotVerified(Address),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Local networks (e.g. anvil, ganache, geth --dev) cannot be indexed by etherscan")]
    LocalNetworksNotSupported,
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("Missing field: {0}")]
    Builder(String),
    #[error("Missing solc version: {0}")]
    MissingSolcVersion(String),
}
