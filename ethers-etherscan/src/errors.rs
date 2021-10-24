use ethers_core::types::Chain;
use std::env::VarError;

#[derive(Debug, thiserror::Error)]
pub enum EtherscanError {
    #[error("chain {0} not supported")]
    ChainNotSupported(Chain),
    #[error("contract execution call failed: {0}")]
    ExecutionFailed(String),
    #[error("tx receipt failed")]
    TransactionReceiptFailed,
    #[error("bad status code {0}")]
    BadStatusCode(String),
    #[error(transparent)]
    EnvVarNotFound(#[from] VarError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}
