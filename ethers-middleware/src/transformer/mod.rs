mod ds_proxy;
pub use ds_proxy::DsProxy;

mod gnosis_safe;
pub use gnosis_safe::GnosisSafe;

mod middleware;
pub use middleware::TransformerMiddleware;

use ethers_contract::AbiError;
use ethers_core::{abi::ParseError, types::*};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransformerError {
    #[error("dummy error")]
    Dummy,

    #[error(transparent)]
    AbiParseError(#[from] ParseError),

    #[error(transparent)]
    AbiError(#[from] AbiError),
}

pub trait Transformer: Send + Sync + std::fmt::Debug {
    fn transform(&self, tx: TransactionRequest) -> Result<TransactionRequest, TransformerError>;
}
