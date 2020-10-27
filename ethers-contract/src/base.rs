use crate::Contract;

use ethers_core::{
    abi::{Abi, Detokenize, Error, Function, FunctionExt, InvalidOutputType, Tokenize},
    types::{Address, Bytes, Selector},
};
use ethers_providers::Middleware;

use rustc_hex::ToHex;
use std::{collections::HashMap, fmt::Debug, hash::Hash, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AbiError {
    /// Thrown when the ABI decoding fails
    #[error(transparent)]
    DecodingError(#[from] ethers_core::abi::Error),

    /// Thrown when detokenizing an argument
    #[error(transparent)]
    DetokenizationError(#[from] InvalidOutputType),
}

/// A reduced form of `Contract` which just takes the `abi` and produces
/// ABI encoded data for its functions.
#[derive(Debug, Clone)]
pub struct BaseContract {
    pub(crate) abi: Abi,

    /// A mapping from method signature to a name-index pair for accessing
    /// functions in the contract ABI. This is used to avoid allocation when
    /// searching for matching functions by signature.
    // Adapted from: https://github.com/gnosis/ethcontract-rs/blob/master/src/contract.rs
    pub(crate) methods: HashMap<Selector, (String, usize)>,
}

impl From<Abi> for BaseContract {
    /// Creates a new `BaseContract` from the abi.
    fn from(abi: Abi) -> Self {
        let methods = create_mapping(&abi.functions, |function| function.selector());
        Self { abi, methods }
    }
}

impl BaseContract {
    /// Returns the ABI encoded data for the provided function and arguments
    ///
    /// If the function exists multiple times and you want to use one of the overloaded
    /// versions, consider using `encode_with_selector`
    pub fn encode<T: Tokenize>(&self, name: &str, args: T) -> Result<Bytes, AbiError> {
        let function = self.abi.function(name)?;
        encode_fn(function, args)
    }

    /// Returns the ABI encoded data for the provided function selector and arguments
    pub fn encode_with_selector<T: Tokenize>(
        &self,
        signature: Selector,
        args: T,
    ) -> Result<Bytes, AbiError> {
        let function = self.get_from_signature(signature)?;
        encode_fn(function, args)
    }

    /// Decodes the provided ABI encoded bytes with the selected function name.
    ///
    /// If the function exists multiple times and you want to use one of the overloaded
    /// versions, consider using `decode_with_selector`
    pub fn decode<D: Detokenize>(
        &self,
        name: &str,
        bytes: impl AsRef<[u8]>,
    ) -> Result<D, AbiError> {
        let function = self.abi.function(name)?;
        decode_fn(function, bytes)
    }

    /// Decodes the provided ABI encoded bytes with the selected function selector
    pub fn decode_with_selector<D: Detokenize>(
        &self,
        signature: Selector,
        bytes: impl AsRef<[u8]>,
    ) -> Result<D, AbiError> {
        let function = self.get_from_signature(signature)?;
        decode_fn(function, bytes)
    }

    fn get_from_signature(&self, signature: Selector) -> Result<&Function, AbiError> {
        Ok(self
            .methods
            .get(&signature)
            .map(|(name, index)| &self.abi.functions[name][*index])
            .ok_or_else(|| Error::InvalidName(signature.to_hex::<String>()))?)
    }

    /// Returns a reference to the contract's ABI
    pub fn abi(&self) -> &Abi {
        &self.abi
    }

    /// Upgrades a `BaseContract` into a full fledged contract with an address and middleware.
    pub fn into_contract<M: Middleware>(
        self,
        address: Address,
        client: impl Into<Arc<M>>,
    ) -> Contract<M> {
        Contract::new(address, self, client)
    }
}

impl AsRef<Abi> for BaseContract {
    fn as_ref(&self) -> &Abi {
        self.abi()
    }
}

// Helper for encoding arguments for a specific function
pub(crate) fn encode_fn<T: Tokenize>(function: &Function, args: T) -> Result<Bytes, AbiError> {
    let tokens = args.into_tokens();
    Ok(function.encode_input(&tokens).map(Into::into)?)
}

// Helper for decoding bytes from a specific function
pub(crate) fn decode_fn<D: Detokenize>(
    function: &Function,
    bytes: impl AsRef<[u8]>,
) -> Result<D, AbiError> {
    let tokens = function.decode_output(bytes.as_ref())?;
    Ok(D::from_tokens(tokens)?)
}

/// Utility function for creating a mapping between a unique signature and a
/// name-index pair for accessing contract ABI items.
fn create_mapping<T, S, F>(
    elements: &HashMap<String, Vec<T>>,
    signature: F,
) -> HashMap<S, (String, usize)>
where
    S: Hash + Eq,
    F: Fn(&T) -> S,
{
    let signature = &signature;
    elements
        .iter()
        .flat_map(|(name, sub_elements)| {
            sub_elements
                .iter()
                .enumerate()
                .map(move |(index, element)| (signature(element), (name.to_owned(), index)))
        })
        .collect()
}
