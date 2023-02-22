use ethers_core::{
    abi::{self, InvalidOutputType},
    types::Bytes,
};

use ethers_providers::{Middleware, ProviderError};

use crate::{ContractError, EthError};

/// Errors using the [`crate::Multicall`] system
#[derive(Debug, thiserror::Error)]
pub enum MulticallError<M: Middleware> {
    /// Contract call returned an error
    #[error(transparent)]
    ContractError(#[from] ContractError<M>),

    /// Unsupported chain
    #[error("Chain ID {0} is currently not supported by Multicall. Provide an address instead.")]
    InvalidChainId(u64),

    /// Contract call reverted when not allowed
    #[error("Illegal revert: Multicall2 call reverted when it wasn't allowed to.")]
    IllegalRevert,
}

impl<M: Middleware> From<abi::Error> for MulticallError<M> {
    fn from(value: abi::Error) -> Self {
        Self::ContractError(ContractError::DecodingError(value))
    }
}

impl<M: Middleware> From<InvalidOutputType> for MulticallError<M> {
    fn from(value: InvalidOutputType) -> Self {
        Self::ContractError(ContractError::DetokenizationError(value))
    }
}

impl<M: Middleware> MulticallError<M> {
    /// Convert a `MulticallError` to a the underlying error if possible.
    pub fn as_contract_error(&self) -> Option<&ContractError<M>> {
        match self {
            MulticallError::ContractError(e) => Some(e),
            _ => None,
        }
    }

    /// True if the underlying error is a [`ContractError`]
    pub fn is_contract_error(&self) -> bool {
        matches!(self, MulticallError::ContractError(_))
    }

    /// Convert a `MulticallError` to a the underlying error if possible.
    pub fn as_middleware_error(&self) -> Option<&M::Error> {
        self.as_contract_error().and_then(ContractError::as_middleware_error)
    }

    /// True if the underlying error is a MiddlewareError
    pub fn is_middleware_error(&self) -> bool {
        self.as_contract_error().map(ContractError::is_middleware_error).unwrap_or_default()
    }

    /// Convert a `MulticallError` to a [`ProviderError`] if possible.
    pub fn as_provider_error(&self) -> Option<&ProviderError> {
        self.as_contract_error().and_then(ContractError::as_provider_error)
    }

    /// True if the error is a provider error
    pub fn is_provider_error(&self) -> bool {
        self.as_contract_error().map(ContractError::is_provider_error).unwrap_or_default()
    }

    /// If this `MulticallError` is a revert, this method will retrieve a
    /// reference to the underlying revert data. This ABI-encoded data could be
    /// a String, or a custom Solidity error type.
    ///
    /// ## Returns
    ///
    /// `None` if the error is not a revert
    /// `Some(data)` with the revert data, if the error is a revert
    ///
    /// ## Note
    ///
    /// To skip this step, consider using [`MulticallError::decode_revert`]
    pub fn as_revert(&self) -> Option<&Bytes> {
        self.as_contract_error().and_then(ContractError::as_revert)
    }

    /// True if the error is a revert, false otherwise
    pub fn is_revert(&self) -> bool {
        self.as_contract_error().map(ContractError::is_revert).unwrap_or_default()
    }

    /// Decode revert data into an [`EthError`] type. Returns `None` if
    /// decoding fails, or if this is not a revert
    pub fn decode_revert<Err: EthError>(&self) -> Option<Err> {
        self.as_revert().and_then(|data| Err::decode_with_selector(data))
    }
}
