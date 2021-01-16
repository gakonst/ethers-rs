mod ds_proxy;
mod gnosis_safe;
mod middleware;

use ethers_contract::AbiError;
use ethers_core::{abi::ParseError, types::*};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyWalletError {
    #[error("dummy error")]
    Dummy,

    #[error(transparent)]
    AbiParseError(#[from] ParseError),

    #[error(transparent)]
    AbiError(#[from] AbiError),
}

pub trait ProxyWallet: Send + Sync + std::fmt::Debug {
    fn get_proxy_tx(&self, tx: TransactionRequest) -> Result<TransactionRequest, ProxyWalletError>;
}
