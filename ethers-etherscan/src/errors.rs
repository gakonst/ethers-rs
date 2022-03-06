use ethers_core::types::{Address, Chain};
use std::env::VarError;

#[derive(Debug, thiserror::Error)]
pub enum EtherscanError {
    #[error("chain {0} not supported")]
    ChainNotSupported(Chain),
    #[error("contract execution call failed: {0}")]
    ExecutionFailed(String),
    #[error("balance failed")]
    BalanceFailed,
    #[error("tx receipt failed")]
    TransactionReceiptFailed,
    #[error("gas estimation failed")]
    GasEstimationFailed,
    #[error("bad status code {0}")]
    BadStatusCode(String),
    #[error(transparent)]
    EnvVarNotFound(#[from] VarError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("Contract source code not verified: {0}")]
    ContractCodeNotVerified(Address),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}
