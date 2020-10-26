use crate::Contract;

use ethers_core::{
    abi::{Abi, FunctionExt},
    types::{Address, Selector},
};
use ethers_providers::Middleware;

use std::{collections::HashMap, fmt::Debug, hash::Hash, sync::Arc};

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
