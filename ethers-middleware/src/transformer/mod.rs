pub mod ds_proxy;
pub use ds_proxy::DsProxy;

mod middleware;
pub use middleware::TransformerMiddleware;

use ethers_contract::AbiError;
use ethers_core::{abi::ParseError, types::transaction::eip2718::TypedTransaction};
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors thrown from the types that implement the `Transformer` trait.
pub enum TransformerError {
    #[error("The field `{0}` is missing")]
    MissingField(String),

    #[error(transparent)]
    AbiParseError(#[from] ParseError),

    #[error(transparent)]
    AbiError(#[from] AbiError),
}

/// `Transformer` is a trait to be implemented by a proxy wallet, eg. [`DsProxy`], that intends to
/// intercept a transaction request and transform it into one that is instead sent via the proxy
/// contract.
pub trait Transformer: Send + Sync + std::fmt::Debug {
    /// Transforms a [`transaction request`] into one that can be broadcasted and execute via the
    /// proxy contract.
    ///
    /// [`transaction request`]: struct@ethers_core::types::TransactionRequest
    fn transform(&self, tx: &mut TypedTransaction) -> Result<(), TransformerError>;
}
