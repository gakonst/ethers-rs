use ethers_contract::Lazy;
use ethers_core::types::*;
use std::{collections::HashMap, str::FromStr};

/// A lazily computed hash map with the Ethereum network IDs as keys and the corresponding
/// DsProxyFactory contract addresses as values
pub static ADDRESS_BOOK: Lazy<HashMap<U256, Address>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // mainnet
    let addr =
        Address::from_str("eefba1e63905ef1d7acba5a8513c70307c1ce441").expect("Decoding failed");
    m.insert(U256::from(1u8), addr);

    m
});

///
/// Generated with
/// ```ignore
/// # use ethers_contract::abigen;
/// abigen!(DsProxyFactory,
///         "ethers-middleware/contracts/DsProxyFactory.json",
///         methods {
///             build() as build_with_sender;
///         }
///     );
/// ```
///
// Auto-generated type-safe bindings
pub use dsproxyfactory_mod::*;
#[allow(clippy::too_many_arguments)]
mod dsproxyfactory_mod {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    use ethers_contract::{
        builders::{ContractCall, Event},
        Contract, EthEvent, Lazy,
    };
    use ethers_core::{
        abi::{parse_abi, Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers_providers::Middleware;
    #[doc = "DsProxyFactory was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    pub static DSPROXYFACTORY_ABI: Lazy<Abi> = Lazy::new(|| {
        serde_json :: from_str ("[{\"constant\":true,\"inputs\":[{\"name\":\"\",\"type\":\"address\"}],\"name\":\"isProxy\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"payable\":false,\"stateMutability\":\"view\",\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"cache\",\"outputs\":[{\"name\":\"\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"view\",\"type\":\"function\"},{\"constant\":false,\"inputs\":[],\"name\":\"build\",\"outputs\":[{\"name\":\"proxy\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"owner\",\"type\":\"address\"}],\"name\":\"build\",\"outputs\":[{\"name\":\"proxy\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"sender\",\"type\":\"address\"},{\"indexed\":true,\"name\":\"owner\",\"type\":\"address\"},{\"indexed\":false,\"name\":\"proxy\",\"type\":\"address\"},{\"indexed\":false,\"name\":\"cache\",\"type\":\"address\"}],\"name\":\"Created\",\"type\":\"event\"}]\n") . expect ("invalid abi")
    });
    #[derive(Clone)]
    pub struct DsProxyFactory<M>(Contract<M>);
    impl<M> std::ops::Deref for DsProxyFactory<M> {
        type Target = Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M: Middleware> std::fmt::Debug for DsProxyFactory<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(DsProxyFactory))
                .field(&self.address())
                .finish()
        }
    }
    impl<'a, M: Middleware> DsProxyFactory<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<Address>>(address: T, client: Arc<M>) -> Self {
            let contract = Contract::new(address.into(), DSPROXYFACTORY_ABI.clone(), client);
            Self(contract)
        }
        #[doc = "Calls the contract's `isProxy` (0x29710388) function"]
        pub fn is_proxy(&self, p0: Address) -> ContractCall<M, bool> {
            self.0
                .method_hash([41, 113, 3, 136], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `build` (0x8e1a55fc) function
        pub fn build_with_sender(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::Address> {
            self.0
                .method_hash([142, 26, 85, 252], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `build` (0xf3701da2) function
        pub fn build(
            &self,
            owner: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::Address> {
            self.0
                .method_hash([243, 112, 29, 162], owner)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `cache` (0x60c7d295) function"]
        pub fn cache(&self) -> ContractCall<M, Address> {
            self.0
                .method_hash([96, 199, 210, 149], ())
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `Created` event
        pub fn created_filter(&self) -> ethers_contract::builders::Event<M, CreatedFilter> {
            self.0.event()
        }

        /// Returns an [`Event`](ethers_contract::builders::Event) builder for all events of this contract
        pub fn events(&self) -> ethers_contract::builders::Event<M, CreatedFilter> {
            self.0.event_with_filter(Default::default())
        }
    }
    #[derive(Clone, Debug, Default, Eq, PartialEq, EthEvent)]
    #[ethevent(name = "Created")]
    pub struct CreatedFilter {
        #[ethevent(indexed)]
        pub sender: Address,
        #[ethevent(indexed)]
        pub owner: Address,
        pub proxy: Address,
        pub cache: Address,
    }
}
