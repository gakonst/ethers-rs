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
        abi::{
            parse_abi, Abi, Detokenize, InvalidOutputType, ParamType, Token, Tokenizable,
            TokenizableItem,
        },
        types::*,
    };
    use ethers_providers::Middleware;
    #[doc = "DsProxyFactory was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;

    pub static DSPROXYFACTORY_ABI: Lazy<Abi> = Lazy::new(|| {
        serde_json :: from_str ("[{\"constant\":true,\"inputs\":[{\"name\":\"\",\"type\":\"address\"}],\"name\":\"isProxy\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"payable\":false,\"stateMutability\":\"view\",\"type\":\"function\"},{\"constant\":true,\"inputs\":[],\"name\":\"cache\",\"outputs\":[{\"name\":\"\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"view\",\"type\":\"function\"},{\"constant\":false,\"inputs\":[],\"name\":\"build\",\"outputs\":[{\"name\":\"proxy\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"owner\",\"type\":\"address\"}],\"name\":\"build\",\"outputs\":[{\"name\":\"proxy\",\"type\":\"address\"}],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"name\":\"sender\",\"type\":\"address\"},{\"indexed\":true,\"name\":\"owner\",\"type\":\"address\"},{\"indexed\":false,\"name\":\"proxy\",\"type\":\"address\"},{\"indexed\":false,\"name\":\"cache\",\"type\":\"address\"}],\"name\":\"Created\",\"type\":\"event\"}]\n") . expect ("invalid abi")
    });
    pub struct DsProxyFactory<M>(Contract<M>);
    impl<M> Clone for DsProxyFactory<M> {
        fn clone(&self) -> Self {
            DsProxyFactory(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for DsProxyFactory<M> {
        type Target = Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M: Middleware> std::fmt::Debug for DsProxyFactory<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(DsProxyFactory)).field(&self.address()).finish()
        }
    }
    impl<M: Middleware> DsProxyFactory<M> {
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
        pub fn build_with_sender(&self) -> ContractCall<M, Address> {
            self.0
                .method_hash([142, 26, 85, 252], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `build` (0xf3701da2) function
        pub fn build(&self, owner: Address) -> ContractCall<M, Address> {
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
        pub fn created_filter(&self) -> Event<M, CreatedFilter> {
            self.0.event()
        }

        /// Returns an [`Event`](ethers_contract::builders::Event) builder for all events of this
        /// contract
        pub fn events(&self) -> Event<M, CreatedFilter> {
            self.0.event_with_filter(Default::default())
        }
    }

    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    pub struct CreatedFilter {
        pub sender: Address,
        pub owner: Address,
        pub proxy: Address,
        pub cache: Address,
    }

    impl ethers_core::abi::AbiType for CreatedFilter {
        fn param_type() -> ParamType {
            ParamType::Tuple(::std::vec![
                ParamType::Address; 4
            ])
        }
    }
    impl ethers_core::abi::AbiArrayType for CreatedFilter {}

    impl Tokenizable for CreatedFilter {
        fn from_token(token: Token) -> Result<Self, InvalidOutputType>
        where
            Self: Sized,
        {
            if let Token::Tuple(tokens) = token {
                if tokens.len() != 4usize {
                    return Err(InvalidOutputType(format!(
                        "Expected {} tokens, got {}: {:?}",
                        4usize,
                        tokens.len(),
                        tokens
                    )))
                }
                let mut iter = tokens.into_iter();
                Ok(Self {
                    sender: Tokenizable::from_token(iter.next().unwrap())?,
                    owner: Tokenizable::from_token(iter.next().unwrap())?,
                    proxy: Tokenizable::from_token(iter.next().unwrap())?,
                    cache: Tokenizable::from_token(iter.next().unwrap())?,
                })
            } else {
                Err(InvalidOutputType(format!("Expected Tuple, got {token:?}")))
            }
        }
        fn into_token(self) -> Token {
            Token::Tuple(::std::vec![
                self.sender.into_token(),
                self.owner.into_token(),
                self.proxy.into_token(),
                self.cache.into_token(),
            ])
        }
    }
    impl TokenizableItem for CreatedFilter {}

    impl ethers_contract::EthEvent for CreatedFilter {
        fn name() -> std::borrow::Cow<'static, str> {
            "Created".into()
        }
        fn signature() -> H256 {
            H256([
                37, 155, 48, 202, 57, 136, 92, 109, 128, 26, 11, 93, 188, 152, 134, 64, 243, 194,
                94, 47, 55, 83, 31, 225, 56, 197, 197, 175, 137, 85, 212, 27,
            ])
        }
        fn abi_signature() -> std::borrow::Cow<'static, str> {
            "Created(address,address,address,address)".into()
        }
        fn decode_log(log: &ethers_core::abi::RawLog) -> Result<Self, ethers_core::abi::Error>
        where
            Self: Sized,
        {
            let ethers_core::abi::RawLog { data, topics } = log;
            let event_signature = topics.get(0).ok_or(ethers_core::abi::Error::InvalidData)?;
            if event_signature != &Self::signature() {
                return Err(ethers_core::abi::Error::InvalidData)
            }
            let topic_types = ::std::vec![ParamType::Address, ParamType::Address];
            let data_types = [ParamType::Address, ParamType::Address];
            let flat_topics =
                topics.iter().skip(1).flat_map(|t| t.as_ref().to_vec()).collect::<Vec<u8>>();
            let topic_tokens = ethers_core::abi::decode(&topic_types, &flat_topics)?;
            if topic_tokens.len() != topics.len() - 1 {
                return Err(ethers_core::abi::Error::InvalidData)
            }
            let data_tokens = ethers_core::abi::decode(&data_types, data)?;
            let tokens: Vec<_> = topic_tokens.into_iter().chain(data_tokens.into_iter()).collect();
            Tokenizable::from_token(Token::Tuple(tokens))
                .map_err(|_| ethers_core::abi::Error::InvalidData)
        }
        fn is_anonymous() -> bool {
            false
        }
    }
}
