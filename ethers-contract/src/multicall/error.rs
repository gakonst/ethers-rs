use ethers_core::{
    abi::{self, InvalidOutputType},
    types::Bytes,
};

use ethers_providers::{Middleware, ProviderError};

use crate::{ContractError, EthError};

#[derive(Debug, thiserror::Error)]
pub enum MulticallError<M: Middleware> {
    #[error(transparent)]
    ContractError(#[from] ContractError<M>),

    #[error("Chain ID {0} is currently not supported by Multicall. Provide an address instead.")]
    InvalidChainId(u64),

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
    pub fn as_contract_error(&self) -> Option<&ContractError<M>> {
        match self {
            MulticallError::ContractError(e) => Some(e),
            _ => None,
        }
    }

    pub fn is_contract_error(&self) -> bool {
        matches!(self, MulticallError::ContractError(_))
    }

    pub fn as_middleware_error(&self) -> Option<&M::Error> {
        self.as_contract_error().and_then(ContractError::as_middleware_error)
    }

    pub fn is_middleware_error(&self) -> bool {
        self.as_contract_error().map(ContractError::is_middleware_error).unwrap_or_default()
    }

    pub fn as_provider_error(&self) -> Option<&ProviderError> {
        self.as_contract_error().and_then(ContractError::as_provider_error)
    }

    pub fn is_provider_error(&self) -> bool {
        self.as_contract_error().map(ContractError::is_provider_error).unwrap_or_default()
    }

    pub fn as_revert(&self) -> Option<&Bytes> {
        self.as_contract_error().and_then(ContractError::as_revert)
    }

    pub fn is_revert(&self) -> bool {
        self.as_contract_error().map(ContractError::is_revert).unwrap_or_default()
    }

    pub fn decode_revert<Err: EthError>(&self) -> Option<Err> {
        self.as_revert().and_then(|data| Err::decode(data).ok())
    }
}
