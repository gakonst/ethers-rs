/Users/Matthias/.cargo/bin/cargo expand --test abigen --color=always --theme=Dracula --tests
Checking arrayvec v0.7.2
Checking crypto-mac v0.8.0
Checking password-hash v0.2.3
Checking futures-util v0.3.17
Checking tokio v1.12.0
Checking elliptic-curve v0.10.6
Checking tracing-futures v0.2.5
Checking tungstenite v0.14.0
Checking blake2 v0.9.2
Checking parity-scale-codec v2.3.1
Checking pbkdf2 v0.8.0
Checking ecdsa v0.12.4
Checking coins-core v0.2.2
Checking scrypt v0.7.0
Checking k256 v0.9.6
Checking eth-keystore v0.3.0
Checking impl-codec v0.5.1
Checking primitive-types v0.10.1
Checking coins-bip32 v0.3.0
Checking coins-bip39 v0.3.0
Checking ethereum-types v0.12.1
Checking futures-executor v0.3.17
Checking ethabi v15.0.0
Checking tokio-util v0.6.8
Checking tokio-native-tls v0.3.0
Checking tokio-rustls v0.22.0
Checking tokio-tungstenite v0.15.0
Checking h2 v0.3.6
Checking ethers-core v0.5.4 (/Users/Matthias/git/rust/ethers-rs/ethers-core)
Checking hyper v0.14.13
Checking ethers-signers v0.5.3 (/Users/Matthias/git/rust/ethers-rs/ethers-signers)
Checking hyper-rustls v0.22.1
Checking hyper-tls v0.5.0
Checking reqwest v0.11.6
Checking ethers-contract-abigen v0.5.3 (/Users/Matthias/git/rust/ethers-rs/ethers-contract/ethers-contract-abigen)
Checking ethers-providers v0.5.4 (/Users/Matthias/git/rust/ethers-rs/ethers-providers)
Checking ethers-contract v0.5.3 (/Users/Matthias/git/rust/ethers-rs/ethers-contract)
Checking ethers-middleware v0.5.3 (/Users/Matthias/git/rust/ethers-rs/ethers-middleware)
Finished dev [unoptimized + debuginfo] target(s) in 20.78s

#![feature(prelude_import)]
#![cfg(feature = "abigen")]
//! Test cases to validate the `abigen!` macro
#[prelude_import]
use std::prelude::rust_2018::*;
#[macro_use]
extern crate std;
use ethers_contract::{abigen, EthEvent};
use ethers_core::{
    abi::{AbiDecode, AbiEncode, Address, Tokenizable},
    types::{transaction::eip2718::TypedTransaction, Eip1559TransactionRequest, U256},
    utils::Solc,
};
use ethers_providers::Provider;
use std::{convert::TryFrom, sync::Arc};
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const can_gen_human_readable: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("can_gen_human_readable"),
        ignore: false,
        allow_fail: false,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(can_gen_human_readable())),
};
fn can_gen_human_readable() {
    pub use simplecontract_mod::*;
    #[allow(clippy::too_many_arguments)]
    mod simplecontract_mod {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
        #![allow(clippy::type_complexity)]
        #![allow(unused_imports)]
        ///SimpleContract was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
        use std::sync::Arc;
        use ethers_core::{
            abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
            types::*,
        };
        use ethers_contract::{
            Contract,
            builders::{ContractCall, Event},
            Lazy,
        };
        use ethers_providers::Middleware;
        pub static SIMPLECONTRACT_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
            ethers_contract::Lazy::new(|| {
                ethers_core :: abi :: parse_abi_str ("[\n        event ValueChanged(address indexed author, string oldValue, string newValue)\n    ]") . expect ("invalid abi")
            });
        pub struct SimpleContract<M>(ethers_contract::Contract<M>);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<M: ::core::clone::Clone> ::core::clone::Clone for SimpleContract<M> {
            #[inline]
            fn clone(&self) -> SimpleContract<M> {
                match *self {
                    SimpleContract(ref __self_0_0) => {
                        SimpleContract(::core::clone::Clone::clone(&(*__self_0_0)))
                    }
                }
            }
        }
        impl<M> std::ops::Deref for SimpleContract<M> {
            type Target = ethers_contract::Contract<M>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<M: ethers_providers::Middleware> std::fmt::Debug for SimpleContract<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple("SimpleContract")
                    .field(&self.address())
                    .finish()
            }
        }
        impl<'a, M: ethers_providers::Middleware> SimpleContract<M> {
            /// Creates a new contract instance with the specified `ethers`
            /// client at the given `Address`. The contract derefs to a `ethers::Contract`
            /// object
            pub fn new<T: Into<ethers_core::types::Address>>(
                address: T,
                client: ::std::sync::Arc<M>,
            ) -> Self {
                let contract = ethers_contract::Contract::new(
                    address.into(),
                    SIMPLECONTRACT_ABI.clone(),
                    client,
                );
                Self(contract)
            }
            ///Gets the contract's `ValueChanged` event
            pub fn value_changed_filter(
                &self,
            ) -> ethers_contract::builders::Event<M, ValueChangedFilter> {
                self.0.event()
            }
            /// Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract
            pub fn events(&self) -> ethers_contract::builders::Event<M, ValueChangedFilter> {
                self.0.event_with_filter(Default::default())
            }
        }
        #[ethevent(name = "ValueChanged", abi = "ValueChanged(address,string,string)")]
        pub struct ValueChangedFilter {
            #[ethevent(indexed)]
            pub author: ethers_core::types::Address,
            pub old_value: String,
            pub new_value: String,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for ValueChangedFilter {
            #[inline]
            fn clone(&self) -> ValueChangedFilter {
                match *self {
                    ValueChangedFilter {
                        author: ref __self_0_0,
                        old_value: ref __self_0_1,
                        new_value: ref __self_0_2,
                    } => ValueChangedFilter {
                        author: ::core::clone::Clone::clone(&(*__self_0_0)),
                        old_value: ::core::clone::Clone::clone(&(*__self_0_1)),
                        new_value: ::core::clone::Clone::clone(&(*__self_0_2)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for ValueChangedFilter {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    ValueChangedFilter {
                        author: ref __self_0_0,
                        old_value: ref __self_0_1,
                        new_value: ref __self_0_2,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "ValueChangedFilter");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "author",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "old_value",
                            &&(*__self_0_1),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "new_value",
                            &&(*__self_0_2),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for ValueChangedFilter {
            #[inline]
            fn default() -> ValueChangedFilter {
                ValueChangedFilter {
                    author: ::core::default::Default::default(),
                    old_value: ::core::default::Default::default(),
                    new_value: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for ValueChangedFilter {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for ValueChangedFilter {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::Address>;
                    let _: ::core::cmp::AssertParamIsEq<String>;
                    let _: ::core::cmp::AssertParamIsEq<String>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for ValueChangedFilter {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for ValueChangedFilter {
            #[inline]
            fn eq(&self, other: &ValueChangedFilter) -> bool {
                match *other {
                    ValueChangedFilter {
                        author: ref __self_1_0,
                        old_value: ref __self_1_1,
                        new_value: ref __self_1_2,
                    } => match *self {
                        ValueChangedFilter {
                            author: ref __self_0_0,
                            old_value: ref __self_0_1,
                            new_value: ref __self_0_2,
                        } => {
                            (*__self_0_0) == (*__self_1_0)
                                && (*__self_0_1) == (*__self_1_1)
                                && (*__self_0_2) == (*__self_1_2)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &ValueChangedFilter) -> bool {
                match *other {
                    ValueChangedFilter {
                        author: ref __self_1_0,
                        old_value: ref __self_1_1,
                        new_value: ref __self_1_2,
                    } => match *self {
                        ValueChangedFilter {
                            author: ref __self_0_0,
                            old_value: ref __self_0_1,
                            new_value: ref __self_0_2,
                        } => {
                            (*__self_0_0) != (*__self_1_0)
                                || (*__self_0_1) != (*__self_1_1)
                                || (*__self_0_2) != (*__self_1_2)
                        }
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for ValueChangedFilter
            where
                ethers_core::types::Address: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 3usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&3usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        author: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        old_value: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        new_value: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.author.into_token(),
                    self.old_value.into_token(),
                    self.new_value.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for ValueChangedFilter
            where
                ethers_core::types::Address: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
        }
        impl ethers_contract::EthEvent for ValueChangedFilter {
            fn name() -> ::std::borrow::Cow<'static, str> {
                "ValueChanged".into()
            }
            fn signature() -> ethers_core::types::H256 {
                ethers_core::types::H256([
                    232, 38, 247, 22, 71, 184, 72, 111, 43, 174, 89, 131, 33, 36, 199, 7, 146, 251,
                    160, 68, 3, 103, 32, 165, 78, 200, 218, 205, 213, 223, 79, 203,
                ])
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "ValueChanged(address,string,string)".into()
            }
            fn decode_log(log: &ethers_core::abi::RawLog) -> Result<Self, ethers_core::abi::Error>
                where
                    Self: Sized,
            {
                let ethers_core::abi::RawLog { data, topics } = log;
                let event_signature = topics.get(0).ok_or(ethers_core::abi::Error::InvalidData)?;
                if event_signature != &Self::signature() {
                    return Err(ethers_core::abi::Error::InvalidData);
                }
                let topic_types = <[_]>::into_vec(box [ethers_core::abi::ParamType::Address]);
                let data_types = [
                    ethers_core::abi::ParamType::String,
                    ethers_core::abi::ParamType::String,
                ];
                let flat_topics = topics
                    .iter()
                    .skip(1)
                    .flat_map(|t| t.as_ref().to_vec())
                    .collect::<Vec<u8>>();
                let topic_tokens = ethers_core::abi::decode(&topic_types, &flat_topics)?;
                if topic_tokens.len() != topics.len() - 1 {
                    return Err(ethers_core::abi::Error::InvalidData);
                }
                let data_tokens = ethers_core::abi::decode(&data_types, data)?;
                let tokens: Vec<_> = topic_tokens
                    .into_iter()
                    .chain(data_tokens.into_iter())
                    .collect();
                ethers_core::abi::Tokenizable::from_token(ethers_core::abi::Token::Tuple(tokens))
                    .map_err(|_| ethers_core::abi::Error::InvalidData)
            }
            fn is_anonymous() -> bool {
                false
            }
        }
        impl ::std::fmt::Display for ValueChangedFilter {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.author,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                self.old_value.fmt(f)?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                self.new_value.fmt(f)?;
                Ok(())
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for ValueChangedFilter {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                2u64 => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "author" => _serde::__private::Ok(__Field::__field0),
                                "old_value" => _serde::__private::Ok(__Field::__field1),
                                "new_value" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"author" => _serde::__private::Ok(__Field::__field0),
                                b"old_value" => _serde::__private::Ok(__Field::__field1),
                                b"new_value" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<ValueChangedFilter>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = ValueChangedFilter;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "struct ValueChangedFilter",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::Address,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct ValueChangedFilter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<String>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct ValueChangedFilter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match match _serde::de::SeqAccess::next_element::<String>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct ValueChangedFilter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(ValueChangedFilter {
                                author: __field0,
                                old_value: __field1,
                                new_value: __field2,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<
                                ethers_core::types::Address,
                            > = _serde::__private::None;
                            let mut __field1: _serde::__private::Option<String> =
                                _serde::__private::None;
                            let mut __field2: _serde::__private::Option<String> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "author",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::Address,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "old_value",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<String>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private::Option::is_some(&__field2) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "new_value",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<String>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("author") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("old_value") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private::Some(__field2) => __field2,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("new_value") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(ValueChangedFilter {
                                author: __field0,
                                old_value: __field1,
                                new_value: __field2,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["author", "old_value", "new_value"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "ValueChangedFilter",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<ValueChangedFilter>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for ValueChangedFilter {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "ValueChangedFilter",
                        false as usize + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "author",
                        &self.author,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "old_value",
                        &self.old_value,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "new_value",
                        &self.new_value,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
    }
    {
        match (&"ValueChanged", &ValueChangedFilter::name()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (
            &"ValueChanged(address,string,string)",
            &ValueChangedFilter::abi_signature(),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const can_gen_human_readable_multiple: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("can_gen_human_readable_multiple"),
        ignore: false,
        allow_fail: false,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(can_gen_human_readable_multiple())),
};
fn can_gen_human_readable_multiple() {
    pub mod __shared_types {}
    pub use simplecontract1_mod::*;
    #[allow(clippy::too_many_arguments)]
    mod simplecontract1_mod {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
        #![allow(clippy::type_complexity)]
        #![allow(unused_imports)]
        ///SimpleContract1 was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
        use std::sync::Arc;
        use ethers_core::{
            abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
            types::*,
        };
        use ethers_contract::{
            Contract,
            builders::{ContractCall, Event},
            Lazy,
        };
        use ethers_providers::Middleware;
        pub static SIMPLECONTRACT1_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
            ethers_contract::Lazy::new(|| {
                ethers_core :: abi :: parse_abi_str ("[\n        event ValueChanged1(address indexed author, string oldValue, string newValue)\n    ]") . expect ("invalid abi")
            });
        pub struct SimpleContract1<M>(ethers_contract::Contract<M>);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<M: ::core::clone::Clone> ::core::clone::Clone for SimpleContract1<M> {
            #[inline]
            fn clone(&self) -> SimpleContract1<M> {
                match *self {
                    SimpleContract1(ref __self_0_0) => {
                        SimpleContract1(::core::clone::Clone::clone(&(*__self_0_0)))
                    }
                }
            }
        }
        impl<M> std::ops::Deref for SimpleContract1<M> {
            type Target = ethers_contract::Contract<M>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<M: ethers_providers::Middleware> std::fmt::Debug for SimpleContract1<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple("SimpleContract1")
                    .field(&self.address())
                    .finish()
            }
        }
        impl<'a, M: ethers_providers::Middleware> SimpleContract1<M> {
            /// Creates a new contract instance with the specified `ethers`
            /// client at the given `Address`. The contract derefs to a `ethers::Contract`
            /// object
            pub fn new<T: Into<ethers_core::types::Address>>(
                address: T,
                client: ::std::sync::Arc<M>,
            ) -> Self {
                let contract = ethers_contract::Contract::new(
                    address.into(),
                    SIMPLECONTRACT1_ABI.clone(),
                    client,
                );
                Self(contract)
            }
            ///Gets the contract's `ValueChanged1` event
            pub fn value_changed_1_filter(
                &self,
            ) -> ethers_contract::builders::Event<M, ValueChanged1Filter> {
                self.0.event()
            }
            /// Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract
            pub fn events(&self) -> ethers_contract::builders::Event<M, ValueChanged1Filter> {
                self.0.event_with_filter(Default::default())
            }
        }
        #[ethevent(name = "ValueChanged1", abi = "ValueChanged1(address,string,string)")]
        pub struct ValueChanged1Filter {
            #[ethevent(indexed)]
            pub author: ethers_core::types::Address,
            pub old_value: String,
            pub new_value: String,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for ValueChanged1Filter {
            #[inline]
            fn clone(&self) -> ValueChanged1Filter {
                match *self {
                    ValueChanged1Filter {
                        author: ref __self_0_0,
                        old_value: ref __self_0_1,
                        new_value: ref __self_0_2,
                    } => ValueChanged1Filter {
                        author: ::core::clone::Clone::clone(&(*__self_0_0)),
                        old_value: ::core::clone::Clone::clone(&(*__self_0_1)),
                        new_value: ::core::clone::Clone::clone(&(*__self_0_2)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for ValueChanged1Filter {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    ValueChanged1Filter {
                        author: ref __self_0_0,
                        old_value: ref __self_0_1,
                        new_value: ref __self_0_2,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "ValueChanged1Filter");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "author",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "old_value",
                            &&(*__self_0_1),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "new_value",
                            &&(*__self_0_2),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for ValueChanged1Filter {
            #[inline]
            fn default() -> ValueChanged1Filter {
                ValueChanged1Filter {
                    author: ::core::default::Default::default(),
                    old_value: ::core::default::Default::default(),
                    new_value: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for ValueChanged1Filter {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for ValueChanged1Filter {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::Address>;
                    let _: ::core::cmp::AssertParamIsEq<String>;
                    let _: ::core::cmp::AssertParamIsEq<String>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for ValueChanged1Filter {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for ValueChanged1Filter {
            #[inline]
            fn eq(&self, other: &ValueChanged1Filter) -> bool {
                match *other {
                    ValueChanged1Filter {
                        author: ref __self_1_0,
                        old_value: ref __self_1_1,
                        new_value: ref __self_1_2,
                    } => match *self {
                        ValueChanged1Filter {
                            author: ref __self_0_0,
                            old_value: ref __self_0_1,
                            new_value: ref __self_0_2,
                        } => {
                            (*__self_0_0) == (*__self_1_0)
                                && (*__self_0_1) == (*__self_1_1)
                                && (*__self_0_2) == (*__self_1_2)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &ValueChanged1Filter) -> bool {
                match *other {
                    ValueChanged1Filter {
                        author: ref __self_1_0,
                        old_value: ref __self_1_1,
                        new_value: ref __self_1_2,
                    } => match *self {
                        ValueChanged1Filter {
                            author: ref __self_0_0,
                            old_value: ref __self_0_1,
                            new_value: ref __self_0_2,
                        } => {
                            (*__self_0_0) != (*__self_1_0)
                                || (*__self_0_1) != (*__self_1_1)
                                || (*__self_0_2) != (*__self_1_2)
                        }
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for ValueChanged1Filter
            where
                ethers_core::types::Address: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 3usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&3usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        author: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        old_value: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        new_value: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.author.into_token(),
                    self.old_value.into_token(),
                    self.new_value.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for ValueChanged1Filter
            where
                ethers_core::types::Address: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
        }
        impl ethers_contract::EthEvent for ValueChanged1Filter {
            fn name() -> ::std::borrow::Cow<'static, str> {
                "ValueChanged1".into()
            }
            fn signature() -> ethers_core::types::H256 {
                ethers_core::types::H256([
                    110, 70, 74, 245, 226, 183, 40, 84, 150, 153, 141, 198, 205, 125, 131, 110,
                    194, 199, 86, 146, 14, 1, 200, 212, 204, 84, 166, 204, 164, 134, 161, 242,
                ])
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "ValueChanged1(address,string,string)".into()
            }
            fn decode_log(log: &ethers_core::abi::RawLog) -> Result<Self, ethers_core::abi::Error>
                where
                    Self: Sized,
            {
                let ethers_core::abi::RawLog { data, topics } = log;
                let event_signature = topics.get(0).ok_or(ethers_core::abi::Error::InvalidData)?;
                if event_signature != &Self::signature() {
                    return Err(ethers_core::abi::Error::InvalidData);
                }
                let topic_types = <[_]>::into_vec(box [ethers_core::abi::ParamType::Address]);
                let data_types = [
                    ethers_core::abi::ParamType::String,
                    ethers_core::abi::ParamType::String,
                ];
                let flat_topics = topics
                    .iter()
                    .skip(1)
                    .flat_map(|t| t.as_ref().to_vec())
                    .collect::<Vec<u8>>();
                let topic_tokens = ethers_core::abi::decode(&topic_types, &flat_topics)?;
                if topic_tokens.len() != topics.len() - 1 {
                    return Err(ethers_core::abi::Error::InvalidData);
                }
                let data_tokens = ethers_core::abi::decode(&data_types, data)?;
                let tokens: Vec<_> = topic_tokens
                    .into_iter()
                    .chain(data_tokens.into_iter())
                    .collect();
                ethers_core::abi::Tokenizable::from_token(ethers_core::abi::Token::Tuple(tokens))
                    .map_err(|_| ethers_core::abi::Error::InvalidData)
            }
            fn is_anonymous() -> bool {
                false
            }
        }
        impl ::std::fmt::Display for ValueChanged1Filter {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.author,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                self.old_value.fmt(f)?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                self.new_value.fmt(f)?;
                Ok(())
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for ValueChanged1Filter {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                2u64 => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "author" => _serde::__private::Ok(__Field::__field0),
                                "old_value" => _serde::__private::Ok(__Field::__field1),
                                "new_value" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"author" => _serde::__private::Ok(__Field::__field0),
                                b"old_value" => _serde::__private::Ok(__Field::__field1),
                                b"new_value" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<ValueChanged1Filter>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = ValueChanged1Filter;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "struct ValueChanged1Filter",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::Address,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct ValueChanged1Filter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<String>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct ValueChanged1Filter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match match _serde::de::SeqAccess::next_element::<String>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct ValueChanged1Filter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(ValueChanged1Filter {
                                author: __field0,
                                old_value: __field1,
                                new_value: __field2,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<
                                ethers_core::types::Address,
                            > = _serde::__private::None;
                            let mut __field1: _serde::__private::Option<String> =
                                _serde::__private::None;
                            let mut __field2: _serde::__private::Option<String> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "author",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::Address,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "old_value",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<String>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private::Option::is_some(&__field2) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "new_value",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<String>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("author") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("old_value") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private::Some(__field2) => __field2,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("new_value") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(ValueChanged1Filter {
                                author: __field0,
                                old_value: __field1,
                                new_value: __field2,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["author", "old_value", "new_value"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "ValueChanged1Filter",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<ValueChanged1Filter>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for ValueChanged1Filter {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "ValueChanged1Filter",
                        false as usize + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "author",
                        &self.author,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "old_value",
                        &self.old_value,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "new_value",
                        &self.new_value,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
    }
    pub use simplecontract2_mod::*;
    #[allow(clippy::too_many_arguments)]
    mod simplecontract2_mod {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
        #![allow(clippy::type_complexity)]
        #![allow(unused_imports)]
        ///SimpleContract2 was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
        use std::sync::Arc;
        use ethers_core::{
            abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
            types::*,
        };
        use ethers_contract::{
            Contract,
            builders::{ContractCall, Event},
            Lazy,
        };
        use ethers_providers::Middleware;
        pub static SIMPLECONTRACT2_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
            ethers_contract::Lazy::new(|| {
                ethers_core :: abi :: parse_abi_str ("[\n        event ValueChanged2(address indexed author, string oldValue, string newValue)\n    ]") . expect ("invalid abi")
            });
        pub struct SimpleContract2<M>(ethers_contract::Contract<M>);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<M: ::core::clone::Clone> ::core::clone::Clone for SimpleContract2<M> {
            #[inline]
            fn clone(&self) -> SimpleContract2<M> {
                match *self {
                    SimpleContract2(ref __self_0_0) => {
                        SimpleContract2(::core::clone::Clone::clone(&(*__self_0_0)))
                    }
                }
            }
        }
        impl<M> std::ops::Deref for SimpleContract2<M> {
            type Target = ethers_contract::Contract<M>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<M: ethers_providers::Middleware> std::fmt::Debug for SimpleContract2<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple("SimpleContract2")
                    .field(&self.address())
                    .finish()
            }
        }
        impl<'a, M: ethers_providers::Middleware> SimpleContract2<M> {
            /// Creates a new contract instance with the specified `ethers`
            /// client at the given `Address`. The contract derefs to a `ethers::Contract`
            /// object
            pub fn new<T: Into<ethers_core::types::Address>>(
                address: T,
                client: ::std::sync::Arc<M>,
            ) -> Self {
                let contract = ethers_contract::Contract::new(
                    address.into(),
                    SIMPLECONTRACT2_ABI.clone(),
                    client,
                );
                Self(contract)
            }
            ///Gets the contract's `ValueChanged2` event
            pub fn value_changed_2_filter(
                &self,
            ) -> ethers_contract::builders::Event<M, ValueChanged2Filter> {
                self.0.event()
            }
            /// Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract
            pub fn events(&self) -> ethers_contract::builders::Event<M, ValueChanged2Filter> {
                self.0.event_with_filter(Default::default())
            }
        }
        #[ethevent(name = "ValueChanged2", abi = "ValueChanged2(address,string,string)")]
        pub struct ValueChanged2Filter {
            #[ethevent(indexed)]
            pub author: ethers_core::types::Address,
            pub old_value: String,
            pub new_value: String,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for ValueChanged2Filter {
            #[inline]
            fn clone(&self) -> ValueChanged2Filter {
                match *self {
                    ValueChanged2Filter {
                        author: ref __self_0_0,
                        old_value: ref __self_0_1,
                        new_value: ref __self_0_2,
                    } => ValueChanged2Filter {
                        author: ::core::clone::Clone::clone(&(*__self_0_0)),
                        old_value: ::core::clone::Clone::clone(&(*__self_0_1)),
                        new_value: ::core::clone::Clone::clone(&(*__self_0_2)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for ValueChanged2Filter {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    ValueChanged2Filter {
                        author: ref __self_0_0,
                        old_value: ref __self_0_1,
                        new_value: ref __self_0_2,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "ValueChanged2Filter");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "author",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "old_value",
                            &&(*__self_0_1),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "new_value",
                            &&(*__self_0_2),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for ValueChanged2Filter {
            #[inline]
            fn default() -> ValueChanged2Filter {
                ValueChanged2Filter {
                    author: ::core::default::Default::default(),
                    old_value: ::core::default::Default::default(),
                    new_value: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for ValueChanged2Filter {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for ValueChanged2Filter {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::Address>;
                    let _: ::core::cmp::AssertParamIsEq<String>;
                    let _: ::core::cmp::AssertParamIsEq<String>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for ValueChanged2Filter {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for ValueChanged2Filter {
            #[inline]
            fn eq(&self, other: &ValueChanged2Filter) -> bool {
                match *other {
                    ValueChanged2Filter {
                        author: ref __self_1_0,
                        old_value: ref __self_1_1,
                        new_value: ref __self_1_2,
                    } => match *self {
                        ValueChanged2Filter {
                            author: ref __self_0_0,
                            old_value: ref __self_0_1,
                            new_value: ref __self_0_2,
                        } => {
                            (*__self_0_0) == (*__self_1_0)
                                && (*__self_0_1) == (*__self_1_1)
                                && (*__self_0_2) == (*__self_1_2)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &ValueChanged2Filter) -> bool {
                match *other {
                    ValueChanged2Filter {
                        author: ref __self_1_0,
                        old_value: ref __self_1_1,
                        new_value: ref __self_1_2,
                    } => match *self {
                        ValueChanged2Filter {
                            author: ref __self_0_0,
                            old_value: ref __self_0_1,
                            new_value: ref __self_0_2,
                        } => {
                            (*__self_0_0) != (*__self_1_0)
                                || (*__self_0_1) != (*__self_1_1)
                                || (*__self_0_2) != (*__self_1_2)
                        }
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for ValueChanged2Filter
            where
                ethers_core::types::Address: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 3usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&3usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        author: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        old_value: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        new_value: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.author.into_token(),
                    self.old_value.into_token(),
                    self.new_value.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for ValueChanged2Filter
            where
                ethers_core::types::Address: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
        }
        impl ethers_contract::EthEvent for ValueChanged2Filter {
            fn name() -> ::std::borrow::Cow<'static, str> {
                "ValueChanged2".into()
            }
            fn signature() -> ethers_core::types::H256 {
                ethers_core::types::H256([
                    250, 202, 147, 46, 112, 49, 200, 35, 20, 11, 41, 127, 158, 74, 91, 83, 205,
                    236, 29, 121, 191, 53, 177, 172, 89, 242, 250, 221, 12, 55, 125, 88,
                ])
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "ValueChanged2(address,string,string)".into()
            }
            fn decode_log(log: &ethers_core::abi::RawLog) -> Result<Self, ethers_core::abi::Error>
                where
                    Self: Sized,
            {
                let ethers_core::abi::RawLog { data, topics } = log;
                let event_signature = topics.get(0).ok_or(ethers_core::abi::Error::InvalidData)?;
                if event_signature != &Self::signature() {
                    return Err(ethers_core::abi::Error::InvalidData);
                }
                let topic_types = <[_]>::into_vec(box [ethers_core::abi::ParamType::Address]);
                let data_types = [
                    ethers_core::abi::ParamType::String,
                    ethers_core::abi::ParamType::String,
                ];
                let flat_topics = topics
                    .iter()
                    .skip(1)
                    .flat_map(|t| t.as_ref().to_vec())
                    .collect::<Vec<u8>>();
                let topic_tokens = ethers_core::abi::decode(&topic_types, &flat_topics)?;
                if topic_tokens.len() != topics.len() - 1 {
                    return Err(ethers_core::abi::Error::InvalidData);
                }
                let data_tokens = ethers_core::abi::decode(&data_types, data)?;
                let tokens: Vec<_> = topic_tokens
                    .into_iter()
                    .chain(data_tokens.into_iter())
                    .collect();
                ethers_core::abi::Tokenizable::from_token(ethers_core::abi::Token::Tuple(tokens))
                    .map_err(|_| ethers_core::abi::Error::InvalidData)
            }
            fn is_anonymous() -> bool {
                false
            }
        }
        impl ::std::fmt::Display for ValueChanged2Filter {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.author,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                self.old_value.fmt(f)?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                self.new_value.fmt(f)?;
                Ok(())
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for ValueChanged2Filter {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                2u64 => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "author" => _serde::__private::Ok(__Field::__field0),
                                "old_value" => _serde::__private::Ok(__Field::__field1),
                                "new_value" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"author" => _serde::__private::Ok(__Field::__field0),
                                b"old_value" => _serde::__private::Ok(__Field::__field1),
                                b"new_value" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<ValueChanged2Filter>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = ValueChanged2Filter;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "struct ValueChanged2Filter",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::Address,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct ValueChanged2Filter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<String>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct ValueChanged2Filter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match match _serde::de::SeqAccess::next_element::<String>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct ValueChanged2Filter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(ValueChanged2Filter {
                                author: __field0,
                                old_value: __field1,
                                new_value: __field2,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<
                                ethers_core::types::Address,
                            > = _serde::__private::None;
                            let mut __field1: _serde::__private::Option<String> =
                                _serde::__private::None;
                            let mut __field2: _serde::__private::Option<String> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "author",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::Address,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "old_value",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<String>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private::Option::is_some(&__field2) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "new_value",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<String>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("author") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("old_value") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private::Some(__field2) => __field2,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("new_value") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(ValueChanged2Filter {
                                author: __field0,
                                old_value: __field1,
                                new_value: __field2,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["author", "old_value", "new_value"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "ValueChanged2Filter",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<ValueChanged2Filter>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for ValueChanged2Filter {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "ValueChanged2Filter",
                        false as usize + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "author",
                        &self.author,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "old_value",
                        &self.old_value,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "new_value",
                        &self.new_value,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
    }
    {
        match (&"ValueChanged1", &ValueChanged1Filter::name()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (
            &"ValueChanged1(address,string,string)",
            &ValueChanged1Filter::abi_signature(),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&"ValueChanged2", &ValueChanged2Filter::name()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (
            &"ValueChanged2(address,string,string)",
            &ValueChanged2Filter::abi_signature(),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const can_gen_structs_readable: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("can_gen_structs_readable"),
        ignore: false,
        allow_fail: false,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(can_gen_structs_readable())),
};
fn can_gen_structs_readable() {
    pub use simplecontract_mod::*;
    #[allow(clippy::too_many_arguments)]
    mod simplecontract_mod {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
        #![allow(clippy::type_complexity)]
        #![allow(unused_imports)]
        ///SimpleContract was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
        use std::sync::Arc;
        use ethers_core::{
            abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
            types::*,
        };
        use ethers_contract::{
            Contract,
            builders::{ContractCall, Event},
            Lazy,
        };
        use ethers_providers::Middleware;
        pub static SIMPLECONTRACT_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
            ethers_contract::Lazy::new(|| {
                ethers_core :: abi :: parse_abi_str ("[\n        struct Value {address addr; string value;}\n        struct Addresses {address[] addr; string s;}\n        event ValueChanged(Value indexed old, Value newValue, Addresses _a)\n    ]") . expect ("invalid abi")
            });
        pub struct SimpleContract<M>(ethers_contract::Contract<M>);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<M: ::core::clone::Clone> ::core::clone::Clone for SimpleContract<M> {
            #[inline]
            fn clone(&self) -> SimpleContract<M> {
                match *self {
                    SimpleContract(ref __self_0_0) => {
                        SimpleContract(::core::clone::Clone::clone(&(*__self_0_0)))
                    }
                }
            }
        }
        impl<M> std::ops::Deref for SimpleContract<M> {
            type Target = ethers_contract::Contract<M>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<M: ethers_providers::Middleware> std::fmt::Debug for SimpleContract<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple("SimpleContract")
                    .field(&self.address())
                    .finish()
            }
        }
        impl<'a, M: ethers_providers::Middleware> SimpleContract<M> {
            /// Creates a new contract instance with the specified `ethers`
            /// client at the given `Address`. The contract derefs to a `ethers::Contract`
            /// object
            pub fn new<T: Into<ethers_core::types::Address>>(
                address: T,
                client: ::std::sync::Arc<M>,
            ) -> Self {
                let contract = ethers_contract::Contract::new(
                    address.into(),
                    SIMPLECONTRACT_ABI.clone(),
                    client,
                );
                Self(contract)
            }
            ///Gets the contract's `ValueChanged` event
            pub fn value_changed_filter(
                &self,
            ) -> ethers_contract::builders::Event<M, ValueChangedFilter> {
                self.0.event()
            }
            /// Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract
            pub fn events(&self) -> ethers_contract::builders::Event<M, ValueChangedFilter> {
                self.0.event_with_filter(Default::default())
            }
        }
        #[ethevent(
        name = "ValueChanged",
        abi = "ValueChanged((address,string),(address,string),(address[],string))"
        )]
        pub struct ValueChangedFilter {
            #[ethevent(indexed)]
            pub old: ethers_core::types::H256,
            pub new_value: (ethers_core::types::Address, String),
            pub a: (Vec<ethers_core::types::Address>, String),
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for ValueChangedFilter {
            #[inline]
            fn clone(&self) -> ValueChangedFilter {
                match *self {
                    ValueChangedFilter {
                        old: ref __self_0_0,
                        new_value: ref __self_0_1,
                        a: ref __self_0_2,
                    } => ValueChangedFilter {
                        old: ::core::clone::Clone::clone(&(*__self_0_0)),
                        new_value: ::core::clone::Clone::clone(&(*__self_0_1)),
                        a: ::core::clone::Clone::clone(&(*__self_0_2)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for ValueChangedFilter {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    ValueChangedFilter {
                        old: ref __self_0_0,
                        new_value: ref __self_0_1,
                        a: ref __self_0_2,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "ValueChangedFilter");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "old",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "new_value",
                            &&(*__self_0_1),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "a",
                            &&(*__self_0_2),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for ValueChangedFilter {
            #[inline]
            fn default() -> ValueChangedFilter {
                ValueChangedFilter {
                    old: ::core::default::Default::default(),
                    new_value: ::core::default::Default::default(),
                    a: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for ValueChangedFilter {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for ValueChangedFilter {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::H256>;
                    let _: ::core::cmp::AssertParamIsEq<(ethers_core::types::Address, String)>;
                    let _: ::core::cmp::AssertParamIsEq<(
                        Vec<ethers_core::types::Address>,
                        String,
                    )>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for ValueChangedFilter {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for ValueChangedFilter {
            #[inline]
            fn eq(&self, other: &ValueChangedFilter) -> bool {
                match *other {
                    ValueChangedFilter {
                        old: ref __self_1_0,
                        new_value: ref __self_1_1,
                        a: ref __self_1_2,
                    } => match *self {
                        ValueChangedFilter {
                            old: ref __self_0_0,
                            new_value: ref __self_0_1,
                            a: ref __self_0_2,
                        } => {
                            (*__self_0_0) == (*__self_1_0)
                                && (*__self_0_1) == (*__self_1_1)
                                && (*__self_0_2) == (*__self_1_2)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &ValueChangedFilter) -> bool {
                match *other {
                    ValueChangedFilter {
                        old: ref __self_1_0,
                        new_value: ref __self_1_1,
                        a: ref __self_1_2,
                    } => match *self {
                        ValueChangedFilter {
                            old: ref __self_0_0,
                            new_value: ref __self_0_1,
                            a: ref __self_0_2,
                        } => {
                            (*__self_0_0) != (*__self_1_0)
                                || (*__self_0_1) != (*__self_1_1)
                                || (*__self_0_2) != (*__self_1_2)
                        }
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for ValueChangedFilter
            where
                ethers_core::types::H256: ethers_core::abi::Tokenize,
                (ethers_core::types::Address, String): ethers_core::abi::Tokenize,
                (Vec<ethers_core::types::Address>, String): ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 3usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&3usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        old: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        new_value: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        a: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.old.into_token(),
                    self.new_value.into_token(),
                    self.a.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for ValueChangedFilter
            where
                ethers_core::types::H256: ethers_core::abi::Tokenize,
                (ethers_core::types::Address, String): ethers_core::abi::Tokenize,
                (Vec<ethers_core::types::Address>, String): ethers_core::abi::Tokenize,
        {
        }
        impl ethers_contract::EthEvent for ValueChangedFilter {
            fn name() -> ::std::borrow::Cow<'static, str> {
                "ValueChanged".into()
            }
            fn signature() -> ethers_core::types::H256 {
                ethers_core::types::H256([
                    222, 42, 98, 89, 65, 10, 236, 59, 241, 53, 6, 15, 245, 101, 221, 198, 81, 206,
                    113, 13, 91, 141, 70, 63, 30, 95, 155, 55, 84, 12, 13, 235,
                ])
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "ValueChanged((address,string),(address,string),(address[],string))".into()
            }
            fn decode_log(log: &ethers_core::abi::RawLog) -> Result<Self, ethers_core::abi::Error>
                where
                    Self: Sized,
            {
                let ethers_core::abi::RawLog { data, topics } = log;
                let event_signature = topics.get(0).ok_or(ethers_core::abi::Error::InvalidData)?;
                if event_signature != &Self::signature() {
                    return Err(ethers_core::abi::Error::InvalidData);
                }
                let topic_types =
                    <[_]>::into_vec(box [ethers_core::abi::ParamType::FixedBytes(32)]);
                let data_types = [
                    ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                        ethers_core::abi::ParamType::Address,
                        ethers_core::abi::ParamType::String,
                    ])),
                    ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                        ethers_core::abi::ParamType::Array(Box::new(
                            ethers_core::abi::ParamType::Address,
                        )),
                        ethers_core::abi::ParamType::String,
                    ])),
                ];
                let flat_topics = topics
                    .iter()
                    .skip(1)
                    .flat_map(|t| t.as_ref().to_vec())
                    .collect::<Vec<u8>>();
                let topic_tokens = ethers_core::abi::decode(&topic_types, &flat_topics)?;
                if topic_tokens.len() != topics.len() - 1 {
                    return Err(ethers_core::abi::Error::InvalidData);
                }
                let data_tokens = ethers_core::abi::decode(&data_types, data)?;
                let tokens: Vec<_> = topic_tokens
                    .into_iter()
                    .chain(data_tokens.into_iter())
                    .collect();
                ethers_core::abi::Tokenizable::from_token(ethers_core::abi::Token::Tuple(tokens))
                    .map_err(|_| ethers_core::abi::Error::InvalidData)
            }
            fn is_anonymous() -> bool {
                false
            }
        }
        impl ::std::fmt::Display for ValueChangedFilter {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.old,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.new_value,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.a,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                Ok(())
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for ValueChangedFilter {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                2u64 => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "old" => _serde::__private::Ok(__Field::__field0),
                                "new_value" => _serde::__private::Ok(__Field::__field1),
                                "a" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"old" => _serde::__private::Ok(__Field::__field0),
                                b"new_value" => _serde::__private::Ok(__Field::__field1),
                                b"a" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<ValueChangedFilter>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = ValueChangedFilter;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "struct ValueChangedFilter",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::H256,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct ValueChangedFilter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<(
                                ethers_core::types::Address,
                                String,
                            )>(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct ValueChangedFilter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match match _serde::de::SeqAccess::next_element::<(
                                Vec<ethers_core::types::Address>,
                                String,
                            )>(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct ValueChangedFilter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(ValueChangedFilter {
                                old: __field0,
                                new_value: __field1,
                                a: __field2,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<ethers_core::types::H256> =
                                _serde::__private::None;
                            let mut __field1: _serde::__private::Option<(
                                ethers_core::types::Address,
                                String,
                            )> = _serde::__private::None;
                            let mut __field2: _serde::__private::Option<(
                                Vec<ethers_core::types::Address>,
                                String,
                            )> = _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "old",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::H256,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "new_value",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<(
                                                ethers_core::types::Address,
                                                String,
                                            )>(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private::Option::is_some(&__field2) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "a",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<(
                                                Vec<ethers_core::types::Address>,
                                                String,
                                            )>(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("old") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("new_value") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private::Some(__field2) => __field2,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("a") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(ValueChangedFilter {
                                old: __field0,
                                new_value: __field1,
                                a: __field2,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["old", "new_value", "a"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "ValueChangedFilter",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<ValueChangedFilter>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for ValueChangedFilter {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "ValueChangedFilter",
                        false as usize + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "old",
                        &self.old,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "new_value",
                        &self.new_value,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "a",
                        &self.a,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        ///`Addresses(address[],string)`
        pub struct Addresses {
            pub addr: Vec<ethers_core::types::Address>,
            pub s: String,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Addresses {
            #[inline]
            fn clone(&self) -> Addresses {
                match *self {
                    Addresses {
                        addr: ref __self_0_0,
                        s: ref __self_0_1,
                    } => Addresses {
                        addr: ::core::clone::Clone::clone(&(*__self_0_0)),
                        s: ::core::clone::Clone::clone(&(*__self_0_1)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Addresses {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    Addresses {
                        addr: ref __self_0_0,
                        s: ref __self_0_1,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "Addresses");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "addr",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "s",
                            &&(*__self_0_1),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for Addresses {
            #[inline]
            fn default() -> Addresses {
                Addresses {
                    addr: ::core::default::Default::default(),
                    s: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for Addresses {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for Addresses {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<Vec<ethers_core::types::Address>>;
                    let _: ::core::cmp::AssertParamIsEq<String>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for Addresses {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Addresses {
            #[inline]
            fn eq(&self, other: &Addresses) -> bool {
                match *other {
                    Addresses {
                        addr: ref __self_1_0,
                        s: ref __self_1_1,
                    } => match *self {
                        Addresses {
                            addr: ref __self_0_0,
                            s: ref __self_0_1,
                        } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &Addresses) -> bool {
                match *other {
                    Addresses {
                        addr: ref __self_1_0,
                        s: ref __self_1_1,
                    } => match *self {
                        Addresses {
                            addr: ref __self_0_0,
                            s: ref __self_0_1,
                        } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for Addresses
            where
                Vec<ethers_core::types::Address>: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 2usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&2usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        addr: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        s: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.addr.into_token(),
                    self.s.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for Addresses
            where
                Vec<ethers_core::types::Address>: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for Addresses {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "addr" => _serde::__private::Ok(__Field::__field0),
                                "s" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"addr" => _serde::__private::Ok(__Field::__field0),
                                b"s" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<Addresses>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Addresses;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "struct Addresses")
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                Vec<ethers_core::types::Address>,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct Addresses with 2 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<String>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct Addresses with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(Addresses {
                                addr: __field0,
                                s: __field1,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<
                                Vec<ethers_core::types::Address>,
                            > = _serde::__private::None;
                            let mut __field1: _serde::__private::Option<String> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "addr",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                Vec<ethers_core::types::Address>,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "s",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<String>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("addr") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("s") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(Addresses {
                                addr: __field0,
                                s: __field1,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["addr", "s"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Addresses",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<Addresses>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for Addresses {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "Addresses",
                        false as usize + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "addr",
                        &self.addr,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "s",
                        &self.s,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        ///`Value(address,string)`
        pub struct Value {
            pub addr: ethers_core::types::Address,
            pub value: String,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Value {
            #[inline]
            fn clone(&self) -> Value {
                match *self {
                    Value {
                        addr: ref __self_0_0,
                        value: ref __self_0_1,
                    } => Value {
                        addr: ::core::clone::Clone::clone(&(*__self_0_0)),
                        value: ::core::clone::Clone::clone(&(*__self_0_1)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Value {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    Value {
                        addr: ref __self_0_0,
                        value: ref __self_0_1,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "Value");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "addr",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "value",
                            &&(*__self_0_1),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for Value {
            #[inline]
            fn default() -> Value {
                Value {
                    addr: ::core::default::Default::default(),
                    value: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for Value {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for Value {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::Address>;
                    let _: ::core::cmp::AssertParamIsEq<String>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for Value {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Value {
            #[inline]
            fn eq(&self, other: &Value) -> bool {
                match *other {
                    Value {
                        addr: ref __self_1_0,
                        value: ref __self_1_1,
                    } => match *self {
                        Value {
                            addr: ref __self_0_0,
                            value: ref __self_0_1,
                        } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &Value) -> bool {
                match *other {
                    Value {
                        addr: ref __self_1_0,
                        value: ref __self_1_1,
                    } => match *self {
                        Value {
                            addr: ref __self_0_0,
                            value: ref __self_0_1,
                        } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for Value
            where
                ethers_core::types::Address: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 2usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&2usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        addr: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        value: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.addr.into_token(),
                    self.value.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for Value
            where
                ethers_core::types::Address: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for Value {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "addr" => _serde::__private::Ok(__Field::__field0),
                                "value" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"addr" => _serde::__private::Ok(__Field::__field0),
                                b"value" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<Value>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Value;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "struct Value")
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::Address,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct Value with 2 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<String>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct Value with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(Value {
                                addr: __field0,
                                value: __field1,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<
                                ethers_core::types::Address,
                            > = _serde::__private::None;
                            let mut __field1: _serde::__private::Option<String> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "addr",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::Address,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "value",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<String>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("addr") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("value") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(Value {
                                addr: __field0,
                                value: __field1,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["addr", "value"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Value",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<Value>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for Value {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "Value",
                        false as usize + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "addr",
                        &self.addr,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "value",
                        &self.value,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
    }
    let value = Addresses {
        addr: <[_]>::into_vec(box ["eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap()]),
        s: "hello".to_string(),
    };
    let token = value.clone().into_token();
    {
        match (&value, &Addresses::from_token(token).unwrap()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&"ValueChanged", &ValueChangedFilter::name()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (
            &"ValueChanged((address,string),(address,string),(address[],string))",
            &ValueChangedFilter::abi_signature(),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const can_gen_structs_with_arrays_readable: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("can_gen_structs_with_arrays_readable"),
        ignore: false,
        allow_fail: false,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(can_gen_structs_with_arrays_readable())),
};
fn can_gen_structs_with_arrays_readable() {
    pub use simplecontract_mod::*;
    #[allow(clippy::too_many_arguments)]
    mod simplecontract_mod {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
        #![allow(clippy::type_complexity)]
        #![allow(unused_imports)]
        ///SimpleContract was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
        use std::sync::Arc;
        use ethers_core::{
            abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
            types::*,
        };
        use ethers_contract::{
            Contract,
            builders::{ContractCall, Event},
            Lazy,
        };
        use ethers_providers::Middleware;
        pub static SIMPLECONTRACT_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
            ethers_contract::Lazy::new(|| {
                ethers_core :: abi :: parse_abi_str ("[\n        struct Value {address addr; string value;}\n        struct Addresses {address[] addr; string s;}\n        event ValueChanged(Value indexed old, Value newValue, Addresses[] _a)\n    ]") . expect ("invalid abi")
            });
        pub struct SimpleContract<M>(ethers_contract::Contract<M>);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<M: ::core::clone::Clone> ::core::clone::Clone for SimpleContract<M> {
            #[inline]
            fn clone(&self) -> SimpleContract<M> {
                match *self {
                    SimpleContract(ref __self_0_0) => {
                        SimpleContract(::core::clone::Clone::clone(&(*__self_0_0)))
                    }
                }
            }
        }
        impl<M> std::ops::Deref for SimpleContract<M> {
            type Target = ethers_contract::Contract<M>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<M: ethers_providers::Middleware> std::fmt::Debug for SimpleContract<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple("SimpleContract")
                    .field(&self.address())
                    .finish()
            }
        }
        impl<'a, M: ethers_providers::Middleware> SimpleContract<M> {
            /// Creates a new contract instance with the specified `ethers`
            /// client at the given `Address`. The contract derefs to a `ethers::Contract`
            /// object
            pub fn new<T: Into<ethers_core::types::Address>>(
                address: T,
                client: ::std::sync::Arc<M>,
            ) -> Self {
                let contract = ethers_contract::Contract::new(
                    address.into(),
                    SIMPLECONTRACT_ABI.clone(),
                    client,
                );
                Self(contract)
            }
            ///Gets the contract's `ValueChanged` event
            pub fn value_changed_filter(
                &self,
            ) -> ethers_contract::builders::Event<M, ValueChangedFilter> {
                self.0.event()
            }
            /// Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract
            pub fn events(&self) -> ethers_contract::builders::Event<M, ValueChangedFilter> {
                self.0.event_with_filter(Default::default())
            }
        }
        #[ethevent(
        name = "ValueChanged",
        abi = "ValueChanged((address,string),(address,string),(address[],string)[])"
        )]
        pub struct ValueChangedFilter {
            #[ethevent(indexed)]
            pub old: ethers_core::types::H256,
            pub new_value: (ethers_core::types::Address, String),
            pub a: Vec<(Vec<ethers_core::types::Address>, String)>,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for ValueChangedFilter {
            #[inline]
            fn clone(&self) -> ValueChangedFilter {
                match *self {
                    ValueChangedFilter {
                        old: ref __self_0_0,
                        new_value: ref __self_0_1,
                        a: ref __self_0_2,
                    } => ValueChangedFilter {
                        old: ::core::clone::Clone::clone(&(*__self_0_0)),
                        new_value: ::core::clone::Clone::clone(&(*__self_0_1)),
                        a: ::core::clone::Clone::clone(&(*__self_0_2)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for ValueChangedFilter {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    ValueChangedFilter {
                        old: ref __self_0_0,
                        new_value: ref __self_0_1,
                        a: ref __self_0_2,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "ValueChangedFilter");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "old",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "new_value",
                            &&(*__self_0_1),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "a",
                            &&(*__self_0_2),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for ValueChangedFilter {
            #[inline]
            fn default() -> ValueChangedFilter {
                ValueChangedFilter {
                    old: ::core::default::Default::default(),
                    new_value: ::core::default::Default::default(),
                    a: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for ValueChangedFilter {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for ValueChangedFilter {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::H256>;
                    let _: ::core::cmp::AssertParamIsEq<(ethers_core::types::Address, String)>;
                    let _: ::core::cmp::AssertParamIsEq<
                        Vec<(Vec<ethers_core::types::Address>, String)>,
                    >;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for ValueChangedFilter {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for ValueChangedFilter {
            #[inline]
            fn eq(&self, other: &ValueChangedFilter) -> bool {
                match *other {
                    ValueChangedFilter {
                        old: ref __self_1_0,
                        new_value: ref __self_1_1,
                        a: ref __self_1_2,
                    } => match *self {
                        ValueChangedFilter {
                            old: ref __self_0_0,
                            new_value: ref __self_0_1,
                            a: ref __self_0_2,
                        } => {
                            (*__self_0_0) == (*__self_1_0)
                                && (*__self_0_1) == (*__self_1_1)
                                && (*__self_0_2) == (*__self_1_2)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &ValueChangedFilter) -> bool {
                match *other {
                    ValueChangedFilter {
                        old: ref __self_1_0,
                        new_value: ref __self_1_1,
                        a: ref __self_1_2,
                    } => match *self {
                        ValueChangedFilter {
                            old: ref __self_0_0,
                            new_value: ref __self_0_1,
                            a: ref __self_0_2,
                        } => {
                            (*__self_0_0) != (*__self_1_0)
                                || (*__self_0_1) != (*__self_1_1)
                                || (*__self_0_2) != (*__self_1_2)
                        }
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for ValueChangedFilter
            where
                ethers_core::types::H256: ethers_core::abi::Tokenize,
                (ethers_core::types::Address, String): ethers_core::abi::Tokenize,
                Vec<(Vec<ethers_core::types::Address>, String)>: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 3usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&3usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        old: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        new_value: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        a: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.old.into_token(),
                    self.new_value.into_token(),
                    self.a.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for ValueChangedFilter
            where
                ethers_core::types::H256: ethers_core::abi::Tokenize,
                (ethers_core::types::Address, String): ethers_core::abi::Tokenize,
                Vec<(Vec<ethers_core::types::Address>, String)>: ethers_core::abi::Tokenize,
        {
        }
        impl ethers_contract::EthEvent for ValueChangedFilter {
            fn name() -> ::std::borrow::Cow<'static, str> {
                "ValueChanged".into()
            }
            fn signature() -> ethers_core::types::H256 {
                ethers_core::types::H256([
                    28, 115, 185, 94, 10, 221, 26, 56, 12, 215, 175, 2, 135, 252, 16, 32, 188, 207,
                    156, 137, 241, 138, 78, 227, 95, 51, 183, 232, 186, 88, 88, 64,
                ])
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "ValueChanged((address,string),(address,string),(address[],string)[])".into()
            }
            fn decode_log(log: &ethers_core::abi::RawLog) -> Result<Self, ethers_core::abi::Error>
                where
                    Self: Sized,
            {
                let ethers_core::abi::RawLog { data, topics } = log;
                let event_signature = topics.get(0).ok_or(ethers_core::abi::Error::InvalidData)?;
                if event_signature != &Self::signature() {
                    return Err(ethers_core::abi::Error::InvalidData);
                }
                let topic_types =
                    <[_]>::into_vec(box [ethers_core::abi::ParamType::FixedBytes(32)]);
                let data_types = [
                    ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                        ethers_core::abi::ParamType::Address,
                        ethers_core::abi::ParamType::String,
                    ])),
                    ethers_core::abi::ParamType::Array(Box::new(
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Array(Box::new(
                                ethers_core::abi::ParamType::Address,
                            )),
                            ethers_core::abi::ParamType::String,
                        ])),
                    )),
                ];
                let flat_topics = topics
                    .iter()
                    .skip(1)
                    .flat_map(|t| t.as_ref().to_vec())
                    .collect::<Vec<u8>>();
                let topic_tokens = ethers_core::abi::decode(&topic_types, &flat_topics)?;
                if topic_tokens.len() != topics.len() - 1 {
                    return Err(ethers_core::abi::Error::InvalidData);
                }
                let data_tokens = ethers_core::abi::decode(&data_types, data)?;
                let tokens: Vec<_> = topic_tokens
                    .into_iter()
                    .chain(data_tokens.into_iter())
                    .collect();
                ethers_core::abi::Tokenizable::from_token(ethers_core::abi::Token::Tuple(tokens))
                    .map_err(|_| ethers_core::abi::Error::InvalidData)
            }
            fn is_anonymous() -> bool {
                false
            }
        }
        impl ::std::fmt::Display for ValueChangedFilter {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.old,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.new_value,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.a,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                Ok(())
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for ValueChangedFilter {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                2u64 => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "old" => _serde::__private::Ok(__Field::__field0),
                                "new_value" => _serde::__private::Ok(__Field::__field1),
                                "a" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"old" => _serde::__private::Ok(__Field::__field0),
                                b"new_value" => _serde::__private::Ok(__Field::__field1),
                                b"a" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<ValueChangedFilter>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = ValueChangedFilter;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "struct ValueChangedFilter",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::H256,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct ValueChangedFilter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<(
                                ethers_core::types::Address,
                                String,
                            )>(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct ValueChangedFilter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match match _serde::de::SeqAccess::next_element::<
                                Vec<(Vec<ethers_core::types::Address>, String)>,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct ValueChangedFilter with 3 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(ValueChangedFilter {
                                old: __field0,
                                new_value: __field1,
                                a: __field2,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<ethers_core::types::H256> =
                                _serde::__private::None;
                            let mut __field1: _serde::__private::Option<(
                                ethers_core::types::Address,
                                String,
                            )> = _serde::__private::None;
                            let mut __field2: _serde::__private::Option<
                                Vec<(Vec<ethers_core::types::Address>, String)>,
                            > = _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "old",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::H256,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "new_value",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<(
                                                ethers_core::types::Address,
                                                String,
                                            )>(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private::Option::is_some(&__field2) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "a",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                Vec<(Vec<ethers_core::types::Address>, String)>,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("old") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("new_value") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private::Some(__field2) => __field2,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("a") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(ValueChangedFilter {
                                old: __field0,
                                new_value: __field1,
                                a: __field2,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["old", "new_value", "a"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "ValueChangedFilter",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<ValueChangedFilter>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for ValueChangedFilter {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "ValueChangedFilter",
                        false as usize + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "old",
                        &self.old,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "new_value",
                        &self.new_value,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "a",
                        &self.a,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        ///`Addresses(address[],string)`
        pub struct Addresses {
            pub addr: Vec<ethers_core::types::Address>,
            pub s: String,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Addresses {
            #[inline]
            fn clone(&self) -> Addresses {
                match *self {
                    Addresses {
                        addr: ref __self_0_0,
                        s: ref __self_0_1,
                    } => Addresses {
                        addr: ::core::clone::Clone::clone(&(*__self_0_0)),
                        s: ::core::clone::Clone::clone(&(*__self_0_1)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Addresses {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    Addresses {
                        addr: ref __self_0_0,
                        s: ref __self_0_1,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "Addresses");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "addr",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "s",
                            &&(*__self_0_1),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for Addresses {
            #[inline]
            fn default() -> Addresses {
                Addresses {
                    addr: ::core::default::Default::default(),
                    s: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for Addresses {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for Addresses {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<Vec<ethers_core::types::Address>>;
                    let _: ::core::cmp::AssertParamIsEq<String>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for Addresses {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Addresses {
            #[inline]
            fn eq(&self, other: &Addresses) -> bool {
                match *other {
                    Addresses {
                        addr: ref __self_1_0,
                        s: ref __self_1_1,
                    } => match *self {
                        Addresses {
                            addr: ref __self_0_0,
                            s: ref __self_0_1,
                        } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &Addresses) -> bool {
                match *other {
                    Addresses {
                        addr: ref __self_1_0,
                        s: ref __self_1_1,
                    } => match *self {
                        Addresses {
                            addr: ref __self_0_0,
                            s: ref __self_0_1,
                        } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for Addresses
            where
                Vec<ethers_core::types::Address>: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 2usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&2usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        addr: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        s: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.addr.into_token(),
                    self.s.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for Addresses
            where
                Vec<ethers_core::types::Address>: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for Addresses {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "addr" => _serde::__private::Ok(__Field::__field0),
                                "s" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"addr" => _serde::__private::Ok(__Field::__field0),
                                b"s" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<Addresses>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Addresses;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "struct Addresses")
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                Vec<ethers_core::types::Address>,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct Addresses with 2 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<String>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct Addresses with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(Addresses {
                                addr: __field0,
                                s: __field1,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<
                                Vec<ethers_core::types::Address>,
                            > = _serde::__private::None;
                            let mut __field1: _serde::__private::Option<String> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "addr",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                Vec<ethers_core::types::Address>,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "s",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<String>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("addr") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("s") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(Addresses {
                                addr: __field0,
                                s: __field1,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["addr", "s"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Addresses",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<Addresses>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for Addresses {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "Addresses",
                        false as usize + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "addr",
                        &self.addr,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "s",
                        &self.s,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        ///`Value(address,string)`
        pub struct Value {
            pub addr: ethers_core::types::Address,
            pub value: String,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Value {
            #[inline]
            fn clone(&self) -> Value {
                match *self {
                    Value {
                        addr: ref __self_0_0,
                        value: ref __self_0_1,
                    } => Value {
                        addr: ::core::clone::Clone::clone(&(*__self_0_0)),
                        value: ::core::clone::Clone::clone(&(*__self_0_1)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Value {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    Value {
                        addr: ref __self_0_0,
                        value: ref __self_0_1,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "Value");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "addr",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "value",
                            &&(*__self_0_1),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for Value {
            #[inline]
            fn default() -> Value {
                Value {
                    addr: ::core::default::Default::default(),
                    value: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for Value {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for Value {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::Address>;
                    let _: ::core::cmp::AssertParamIsEq<String>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for Value {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Value {
            #[inline]
            fn eq(&self, other: &Value) -> bool {
                match *other {
                    Value {
                        addr: ref __self_1_0,
                        value: ref __self_1_1,
                    } => match *self {
                        Value {
                            addr: ref __self_0_0,
                            value: ref __self_0_1,
                        } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &Value) -> bool {
                match *other {
                    Value {
                        addr: ref __self_1_0,
                        value: ref __self_1_1,
                    } => match *self {
                        Value {
                            addr: ref __self_0_0,
                            value: ref __self_0_1,
                        } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for Value
            where
                ethers_core::types::Address: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 2usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&2usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        addr: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        value: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.addr.into_token(),
                    self.value.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for Value
            where
                ethers_core::types::Address: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for Value {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "addr" => _serde::__private::Ok(__Field::__field0),
                                "value" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"addr" => _serde::__private::Ok(__Field::__field0),
                                b"value" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<Value>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Value;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "struct Value")
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::Address,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct Value with 2 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<String>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct Value with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(Value {
                                addr: __field0,
                                value: __field1,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<
                                ethers_core::types::Address,
                            > = _serde::__private::None;
                            let mut __field1: _serde::__private::Option<String> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "addr",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::Address,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "value",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<String>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("addr") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("value") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(Value {
                                addr: __field0,
                                value: __field1,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["addr", "value"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Value",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<Value>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for Value {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "Value",
                        false as usize + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "addr",
                        &self.addr,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "value",
                        &self.value,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
    }
    {
        match (
            &"ValueChanged((address,string),(address,string),(address[],string)[])",
            &ValueChangedFilter::abi_signature(),
        ) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
}
fn assert_tokenizeable<T: Tokenizable>() {}
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const can_generate_internal_structs: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("can_generate_internal_structs"),
        ignore: false,
        allow_fail: false,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(can_generate_internal_structs())),
};
fn can_generate_internal_structs() {
    pub use verifiercontract_mod::*;
    #[allow(clippy::too_many_arguments)]
    mod verifiercontract_mod {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
        #![allow(clippy::type_complexity)]
        #![allow(unused_imports)]
        ///VerifierContract was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
        use std::sync::Arc;
        use ethers_core::{
            abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
            types::*,
        };
        use ethers_contract::{
            Contract,
            builders::{ContractCall, Event},
            Lazy,
        };
        use ethers_providers::Middleware;
        pub static VERIFIERCONTRACT_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
            ethers_contract::Lazy::new(|| {
                serde_json :: from_str ("[\n  {\n    \"inputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"constructor\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"uint256[]\",\n        \"name\": \"input\",\n        \"type\": \"uint256[]\"\n      },\n      {\n        \"components\": [\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"X\",\n                \"type\": \"uint256\"\n              },\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"Y\",\n                \"type\": \"uint256\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G1Point\",\n            \"name\": \"A\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"X\",\n                \"type\": \"uint256[2]\"\n              },\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"Y\",\n                \"type\": \"uint256[2]\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G2Point\",\n            \"name\": \"B\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"X\",\n                \"type\": \"uint256\"\n              },\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"Y\",\n                \"type\": \"uint256\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G1Point\",\n            \"name\": \"C\",\n            \"type\": \"tuple\"\n          }\n        ],\n        \"internalType\": \"struct Verifier.Proof\",\n        \"name\": \"proof\",\n        \"type\": \"tuple\"\n      },\n      {\n        \"components\": [\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"X\",\n                \"type\": \"uint256\"\n              },\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"Y\",\n                \"type\": \"uint256\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G1Point\",\n            \"name\": \"alfa1\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"X\",\n                \"type\": \"uint256[2]\"\n              },\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"Y\",\n                \"type\": \"uint256[2]\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G2Point\",\n            \"name\": \"beta2\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"X\",\n                \"type\": \"uint256[2]\"\n              },\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"Y\",\n                \"type\": \"uint256[2]\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G2Point\",\n            \"name\": \"gamma2\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"X\",\n                \"type\": \"uint256[2]\"\n              },\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"Y\",\n                \"type\": \"uint256[2]\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G2Point\",\n            \"name\": \"delta2\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"X\",\n                \"type\": \"uint256\"\n              },\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"Y\",\n                \"type\": \"uint256\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G1Point[]\",\n            \"name\": \"IC\",\n            \"type\": \"tuple[]\"\n          }\n        ],\n        \"internalType\": \"struct Verifier.VerifyingKey\",\n        \"name\": \"vk\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"name\": \"verify\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  }\n]\n") . expect ("invalid abi")
            });
        pub struct VerifierContract<M>(ethers_contract::Contract<M>);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<M: ::core::clone::Clone> ::core::clone::Clone for VerifierContract<M> {
            #[inline]
            fn clone(&self) -> VerifierContract<M> {
                match *self {
                    VerifierContract(ref __self_0_0) => {
                        VerifierContract(::core::clone::Clone::clone(&(*__self_0_0)))
                    }
                }
            }
        }
        impl<M> std::ops::Deref for VerifierContract<M> {
            type Target = ethers_contract::Contract<M>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<M: ethers_providers::Middleware> std::fmt::Debug for VerifierContract<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple("VerifierContract")
                    .field(&self.address())
                    .finish()
            }
        }
        impl<'a, M: ethers_providers::Middleware> VerifierContract<M> {
            /// Creates a new contract instance with the specified `ethers`
            /// client at the given `Address`. The contract derefs to a `ethers::Contract`
            /// object
            pub fn new<T: Into<ethers_core::types::Address>>(
                address: T,
                client: ::std::sync::Arc<M>,
            ) -> Self {
                let contract = ethers_contract::Contract::new(
                    address.into(),
                    VERIFIERCONTRACT_ABI.clone(),
                    client,
                );
                Self(contract)
            }
            ///Calls the contract's `verify` (0x9416c1ee) function
            pub fn verify(
                &self,
                input: ::std::vec::Vec<ethers_core::types::U256>,
                proof: Proof,
                vk: VerifyingKey,
            ) -> ethers_contract::builders::ContractCall<M, bool> {
                self.0
                    .method_hash([148, 22, 193, 238], (input, proof, vk))
                    .expect("method not found (this should never happen)")
            }
        }
        ///Container type for all input parameters for the `verify`function with signature `verify(uint256[],((uint256,uint256),(uint256[2],uint256[2]),(uint256,uint256)),((uint256,uint256),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256,uint256)[]))` and selector `[148, 22, 193, 238]`
        #[ethcall(
        name = "verify",
        abi = "verify(uint256[],((uint256,uint256),(uint256[2],uint256[2]),(uint256,uint256)),((uint256,uint256),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256,uint256)[]))"
        )]
        pub struct VerifyCall {
            pub input: ::std::vec::Vec<ethers_core::types::U256>,
            pub proof: Proof,
            pub vk: VerifyingKey,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for VerifyCall {
            #[inline]
            fn clone(&self) -> VerifyCall {
                match *self {
                    VerifyCall {
                        input: ref __self_0_0,
                        proof: ref __self_0_1,
                        vk: ref __self_0_2,
                    } => VerifyCall {
                        input: ::core::clone::Clone::clone(&(*__self_0_0)),
                        proof: ::core::clone::Clone::clone(&(*__self_0_1)),
                        vk: ::core::clone::Clone::clone(&(*__self_0_2)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for VerifyCall {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    VerifyCall {
                        input: ref __self_0_0,
                        proof: ref __self_0_1,
                        vk: ref __self_0_2,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "VerifyCall");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "input",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "proof",
                            &&(*__self_0_1),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "vk",
                            &&(*__self_0_2),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for VerifyCall {
            #[inline]
            fn default() -> VerifyCall {
                VerifyCall {
                    input: ::core::default::Default::default(),
                    proof: ::core::default::Default::default(),
                    vk: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for VerifyCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for VerifyCall {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<::std::vec::Vec<ethers_core::types::U256>>;
                    let _: ::core::cmp::AssertParamIsEq<Proof>;
                    let _: ::core::cmp::AssertParamIsEq<VerifyingKey>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for VerifyCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for VerifyCall {
            #[inline]
            fn eq(&self, other: &VerifyCall) -> bool {
                match *other {
                    VerifyCall {
                        input: ref __self_1_0,
                        proof: ref __self_1_1,
                        vk: ref __self_1_2,
                    } => match *self {
                        VerifyCall {
                            input: ref __self_0_0,
                            proof: ref __self_0_1,
                            vk: ref __self_0_2,
                        } => {
                            (*__self_0_0) == (*__self_1_0)
                                && (*__self_0_1) == (*__self_1_1)
                                && (*__self_0_2) == (*__self_1_2)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &VerifyCall) -> bool {
                match *other {
                    VerifyCall {
                        input: ref __self_1_0,
                        proof: ref __self_1_1,
                        vk: ref __self_1_2,
                    } => match *self {
                        VerifyCall {
                            input: ref __self_0_0,
                            proof: ref __self_0_1,
                            vk: ref __self_0_2,
                        } => {
                            (*__self_0_0) != (*__self_1_0)
                                || (*__self_0_1) != (*__self_1_1)
                                || (*__self_0_2) != (*__self_1_2)
                        }
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for VerifyCall
            where
                ::std::vec::Vec<ethers_core::types::U256>: ethers_core::abi::Tokenize,
                Proof: ethers_core::abi::Tokenize,
                VerifyingKey: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 3usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&3usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        input: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        proof: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        vk: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.input.into_token(),
                    self.proof.into_token(),
                    self.vk.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for VerifyCall
            where
                ::std::vec::Vec<ethers_core::types::U256>: ethers_core::abi::Tokenize,
                Proof: ethers_core::abi::Tokenize,
                VerifyingKey: ethers_core::abi::Tokenize,
        {
        }
        impl ethers_contract::EthCall for VerifyCall {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                "verify".into()
            }
            fn selector() -> ethers_core::types::Selector {
                [181, 234, 219, 99]
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "verify(uint256[],((uint256,uint256)),((uint256[2],uint256[2])),((uint256,uint256)),((uint256,uint256)),((uint256[2],uint256[2])),((uint256[2],uint256[2])),((uint256[2],uint256[2])),((uint256,uint256)[]))" . into ()
            }
        }
        impl ethers_core::abi::AbiDecode for VerifyCall {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                let bytes = bytes.as_ref();
                if bytes.len() < 4 || bytes[..4] != <Self as ethers_contract::EthCall>::selector() {
                    return Err(ethers_contract::AbiError::WrongSelector);
                }
                let data_types = [
                    ethers_core::abi::ParamType::Array(Box::new(
                        ethers_core::abi::ParamType::Uint(256usize),
                    )),
                    ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Uint(256usize),
                            ethers_core::abi::ParamType::Uint(256usize),
                        ])),
                    ])),
                    ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::FixedArray(
                                Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                2usize,
                            ),
                            ethers_core::abi::ParamType::FixedArray(
                                Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                2usize,
                            ),
                        ])),
                    ])),
                    ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Uint(256usize),
                            ethers_core::abi::ParamType::Uint(256usize),
                        ])),
                    ])),
                    ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Uint(256usize),
                            ethers_core::abi::ParamType::Uint(256usize),
                        ])),
                    ])),
                    ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::FixedArray(
                                Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                2usize,
                            ),
                            ethers_core::abi::ParamType::FixedArray(
                                Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                2usize,
                            ),
                        ])),
                    ])),
                    ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::FixedArray(
                                Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                2usize,
                            ),
                            ethers_core::abi::ParamType::FixedArray(
                                Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                2usize,
                            ),
                        ])),
                    ])),
                    ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::FixedArray(
                                Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                2usize,
                            ),
                            ethers_core::abi::ParamType::FixedArray(
                                Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                2usize,
                            ),
                        ])),
                    ])),
                    ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                        ethers_core::abi::ParamType::Array(Box::new(
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::Uint(256usize),
                                ethers_core::abi::ParamType::Uint(256usize),
                            ])),
                        )),
                    ])),
                ];
                let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                    ethers_core::abi::Token::Tuple(data_tokens),
                )?)
            }
        }
        impl ethers_core::abi::AbiEncode for VerifyCall {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                let selector = <Self as ethers_contract::EthCall>::selector();
                let encoded = ethers_core::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }
        impl ::std::fmt::Display for VerifyCall {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.input,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.proof,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.vk,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                Ok(())
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for VerifyCall {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                2u64 => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "input" => _serde::__private::Ok(__Field::__field0),
                                "proof" => _serde::__private::Ok(__Field::__field1),
                                "vk" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"input" => _serde::__private::Ok(__Field::__field0),
                                b"proof" => _serde::__private::Ok(__Field::__field1),
                                b"vk" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<VerifyCall>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = VerifyCall;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "struct VerifyCall",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                ::std::vec::Vec<ethers_core::types::U256>,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct VerifyCall with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<Proof>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct VerifyCall with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match match _serde::de::SeqAccess::next_element::<
                                VerifyingKey,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct VerifyCall with 3 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(VerifyCall {
                                input: __field0,
                                proof: __field1,
                                vk: __field2,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<
                                ::std::vec::Vec<ethers_core::types::U256>,
                            > = _serde::__private::None;
                            let mut __field1: _serde::__private::Option<Proof> =
                                _serde::__private::None;
                            let mut __field2: _serde::__private::Option<VerifyingKey> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "input",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ::std::vec::Vec<ethers_core::types::U256>,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "proof",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<Proof>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private::Option::is_some(&__field2) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "vk",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<VerifyingKey>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("input") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("proof") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private::Some(__field2) => __field2,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("vk") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(VerifyCall {
                                input: __field0,
                                proof: __field1,
                                vk: __field2,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["input", "proof", "vk"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "VerifyCall",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<VerifyCall>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for VerifyCall {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "VerifyCall",
                        false as usize + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "input",
                        &self.input,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "proof",
                        &self.proof,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "vk",
                        &self.vk,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        ///`G1Point(uint256,uint256)`
        pub struct G1Point {
            pub x: ethers_core::types::U256,
            pub y: ethers_core::types::U256,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for G1Point {
            #[inline]
            fn clone(&self) -> G1Point {
                match *self {
                    G1Point {
                        x: ref __self_0_0,
                        y: ref __self_0_1,
                    } => G1Point {
                        x: ::core::clone::Clone::clone(&(*__self_0_0)),
                        y: ::core::clone::Clone::clone(&(*__self_0_1)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for G1Point {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    G1Point {
                        x: ref __self_0_0,
                        y: ref __self_0_1,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "G1Point");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "x",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "y",
                            &&(*__self_0_1),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for G1Point {
            #[inline]
            fn default() -> G1Point {
                G1Point {
                    x: ::core::default::Default::default(),
                    y: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for G1Point {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for G1Point {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::U256>;
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::U256>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for G1Point {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for G1Point {
            #[inline]
            fn eq(&self, other: &G1Point) -> bool {
                match *other {
                    G1Point {
                        x: ref __self_1_0,
                        y: ref __self_1_1,
                    } => match *self {
                        G1Point {
                            x: ref __self_0_0,
                            y: ref __self_0_1,
                        } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &G1Point) -> bool {
                match *other {
                    G1Point {
                        x: ref __self_1_0,
                        y: ref __self_1_1,
                    } => match *self {
                        G1Point {
                            x: ref __self_0_0,
                            y: ref __self_0_1,
                        } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for G1Point
            where
                ethers_core::types::U256: ethers_core::abi::Tokenize,
                ethers_core::types::U256: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 2usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&2usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        x: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        y: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.x.into_token(),
                    self.y.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for G1Point
            where
                ethers_core::types::U256: ethers_core::abi::Tokenize,
                ethers_core::types::U256: ethers_core::abi::Tokenize,
        {
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for G1Point {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "x" => _serde::__private::Ok(__Field::__field0),
                                "y" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"x" => _serde::__private::Ok(__Field::__field0),
                                b"y" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<G1Point>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = G1Point;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "struct G1Point")
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::U256,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct G1Point with 2 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::U256,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct G1Point with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(G1Point {
                                x: __field0,
                                y: __field1,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<ethers_core::types::U256> =
                                _serde::__private::None;
                            let mut __field1: _serde::__private::Option<ethers_core::types::U256> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "x",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::U256,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "y",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::U256,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("x") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("y") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(G1Point {
                                x: __field0,
                                y: __field1,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["x", "y"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "G1Point",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<G1Point>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for G1Point {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "G1Point",
                        false as usize + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "x",
                        &self.x,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "y",
                        &self.y,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        ///`G2Point(uint256[2],uint256[2])`
        pub struct G2Point {
            pub x: [ethers_core::types::U256; 2],
            pub y: [ethers_core::types::U256; 2],
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for G2Point {
            #[inline]
            fn clone(&self) -> G2Point {
                match *self {
                    G2Point {
                        x: ref __self_0_0,
                        y: ref __self_0_1,
                    } => G2Point {
                        x: ::core::clone::Clone::clone(&(*__self_0_0)),
                        y: ::core::clone::Clone::clone(&(*__self_0_1)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for G2Point {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    G2Point {
                        x: ref __self_0_0,
                        y: ref __self_0_1,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "G2Point");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "x",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "y",
                            &&(*__self_0_1),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for G2Point {
            #[inline]
            fn default() -> G2Point {
                G2Point {
                    x: ::core::default::Default::default(),
                    y: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for G2Point {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for G2Point {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<[ethers_core::types::U256; 2]>;
                    let _: ::core::cmp::AssertParamIsEq<[ethers_core::types::U256; 2]>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for G2Point {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for G2Point {
            #[inline]
            fn eq(&self, other: &G2Point) -> bool {
                match *other {
                    G2Point {
                        x: ref __self_1_0,
                        y: ref __self_1_1,
                    } => match *self {
                        G2Point {
                            x: ref __self_0_0,
                            y: ref __self_0_1,
                        } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &G2Point) -> bool {
                match *other {
                    G2Point {
                        x: ref __self_1_0,
                        y: ref __self_1_1,
                    } => match *self {
                        G2Point {
                            x: ref __self_0_0,
                            y: ref __self_0_1,
                        } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for G2Point
            where
                [ethers_core::types::U256; 2]: ethers_core::abi::Tokenize,
                [ethers_core::types::U256; 2]: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 2usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&2usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        x: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        y: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.x.into_token(),
                    self.y.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for G2Point
            where
                [ethers_core::types::U256; 2]: ethers_core::abi::Tokenize,
                [ethers_core::types::U256; 2]: ethers_core::abi::Tokenize,
        {
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for G2Point {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "x" => _serde::__private::Ok(__Field::__field0),
                                "y" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"x" => _serde::__private::Ok(__Field::__field0),
                                b"y" => _serde::__private::Ok(__Field::__field1),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<G2Point>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = G2Point;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "struct G2Point")
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                [ethers_core::types::U256; 2],
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct G2Point with 2 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<
                                [ethers_core::types::U256; 2],
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct G2Point with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(G2Point {
                                x: __field0,
                                y: __field1,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<
                                [ethers_core::types::U256; 2],
                            > = _serde::__private::None;
                            let mut __field1: _serde::__private::Option<
                                [ethers_core::types::U256; 2],
                            > = _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "x",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                [ethers_core::types::U256; 2],
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "y",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                [ethers_core::types::U256; 2],
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("x") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("y") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(G2Point {
                                x: __field0,
                                y: __field1,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["x", "y"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "G2Point",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<G2Point>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for G2Point {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "G2Point",
                        false as usize + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "x",
                        &self.x,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "y",
                        &self.y,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        ///`Proof((uint256,uint256),(uint256[2],uint256[2]),(uint256,uint256))`
        pub struct Proof {
            pub a: G1Point,
            pub b: G2Point,
            pub c: G1Point,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Proof {
            #[inline]
            fn clone(&self) -> Proof {
                match *self {
                    Proof {
                        a: ref __self_0_0,
                        b: ref __self_0_1,
                        c: ref __self_0_2,
                    } => Proof {
                        a: ::core::clone::Clone::clone(&(*__self_0_0)),
                        b: ::core::clone::Clone::clone(&(*__self_0_1)),
                        c: ::core::clone::Clone::clone(&(*__self_0_2)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Proof {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    Proof {
                        a: ref __self_0_0,
                        b: ref __self_0_1,
                        c: ref __self_0_2,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "Proof");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "a",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "b",
                            &&(*__self_0_1),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "c",
                            &&(*__self_0_2),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for Proof {
            #[inline]
            fn default() -> Proof {
                Proof {
                    a: ::core::default::Default::default(),
                    b: ::core::default::Default::default(),
                    c: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for Proof {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for Proof {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<G1Point>;
                    let _: ::core::cmp::AssertParamIsEq<G2Point>;
                    let _: ::core::cmp::AssertParamIsEq<G1Point>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for Proof {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Proof {
            #[inline]
            fn eq(&self, other: &Proof) -> bool {
                match *other {
                    Proof {
                        a: ref __self_1_0,
                        b: ref __self_1_1,
                        c: ref __self_1_2,
                    } => match *self {
                        Proof {
                            a: ref __self_0_0,
                            b: ref __self_0_1,
                            c: ref __self_0_2,
                        } => {
                            (*__self_0_0) == (*__self_1_0)
                                && (*__self_0_1) == (*__self_1_1)
                                && (*__self_0_2) == (*__self_1_2)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &Proof) -> bool {
                match *other {
                    Proof {
                        a: ref __self_1_0,
                        b: ref __self_1_1,
                        c: ref __self_1_2,
                    } => match *self {
                        Proof {
                            a: ref __self_0_0,
                            b: ref __self_0_1,
                            c: ref __self_0_2,
                        } => {
                            (*__self_0_0) != (*__self_1_0)
                                || (*__self_0_1) != (*__self_1_1)
                                || (*__self_0_2) != (*__self_1_2)
                        }
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for Proof
            where
                G1Point: ethers_core::abi::Tokenize,
                G2Point: ethers_core::abi::Tokenize,
                G1Point: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 3usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&3usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        a: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        b: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        c: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.a.into_token(),
                    self.b.into_token(),
                    self.c.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for Proof
            where
                G1Point: ethers_core::abi::Tokenize,
                G2Point: ethers_core::abi::Tokenize,
                G1Point: ethers_core::abi::Tokenize,
        {
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for Proof {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                2u64 => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "a" => _serde::__private::Ok(__Field::__field0),
                                "b" => _serde::__private::Ok(__Field::__field1),
                                "c" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"a" => _serde::__private::Ok(__Field::__field0),
                                b"b" => _serde::__private::Ok(__Field::__field1),
                                b"c" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<Proof>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Proof;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "struct Proof")
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<G1Point>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct Proof with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<G2Point>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct Proof with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match match _serde::de::SeqAccess::next_element::<G1Point>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct Proof with 3 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(Proof {
                                a: __field0,
                                b: __field1,
                                c: __field2,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<G1Point> =
                                _serde::__private::None;
                            let mut __field1: _serde::__private::Option<G2Point> =
                                _serde::__private::None;
                            let mut __field2: _serde::__private::Option<G1Point> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "a",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<G1Point>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "b",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<G2Point>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private::Option::is_some(&__field2) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "c",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<G1Point>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("a") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("b") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private::Some(__field2) => __field2,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("c") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(Proof {
                                a: __field0,
                                b: __field1,
                                c: __field2,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["a", "b", "c"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Proof",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<Proof>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for Proof {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "Proof",
                        false as usize + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "a",
                        &self.a,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "b",
                        &self.b,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "c",
                        &self.c,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        ///`VerifyingKey((uint256,uint256),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256,uint256)[])`
        pub struct VerifyingKey {
            pub alfa_1: G1Point,
            pub beta_2: G2Point,
            pub gamma_2: G2Point,
            pub delta_2: G2Point,
            pub ic: ::std::vec::Vec<G1Point>,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for VerifyingKey {
            #[inline]
            fn clone(&self) -> VerifyingKey {
                match *self {
                    VerifyingKey {
                        alfa_1: ref __self_0_0,
                        beta_2: ref __self_0_1,
                        gamma_2: ref __self_0_2,
                        delta_2: ref __self_0_3,
                        ic: ref __self_0_4,
                    } => VerifyingKey {
                        alfa_1: ::core::clone::Clone::clone(&(*__self_0_0)),
                        beta_2: ::core::clone::Clone::clone(&(*__self_0_1)),
                        gamma_2: ::core::clone::Clone::clone(&(*__self_0_2)),
                        delta_2: ::core::clone::Clone::clone(&(*__self_0_3)),
                        ic: ::core::clone::Clone::clone(&(*__self_0_4)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for VerifyingKey {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    VerifyingKey {
                        alfa_1: ref __self_0_0,
                        beta_2: ref __self_0_1,
                        gamma_2: ref __self_0_2,
                        delta_2: ref __self_0_3,
                        ic: ref __self_0_4,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "VerifyingKey");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "alfa_1",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "beta_2",
                            &&(*__self_0_1),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "gamma_2",
                            &&(*__self_0_2),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "delta_2",
                            &&(*__self_0_3),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "ic",
                            &&(*__self_0_4),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for VerifyingKey {
            #[inline]
            fn default() -> VerifyingKey {
                VerifyingKey {
                    alfa_1: ::core::default::Default::default(),
                    beta_2: ::core::default::Default::default(),
                    gamma_2: ::core::default::Default::default(),
                    delta_2: ::core::default::Default::default(),
                    ic: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for VerifyingKey {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for VerifyingKey {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<G1Point>;
                    let _: ::core::cmp::AssertParamIsEq<G2Point>;
                    let _: ::core::cmp::AssertParamIsEq<G2Point>;
                    let _: ::core::cmp::AssertParamIsEq<G2Point>;
                    let _: ::core::cmp::AssertParamIsEq<::std::vec::Vec<G1Point>>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for VerifyingKey {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for VerifyingKey {
            #[inline]
            fn eq(&self, other: &VerifyingKey) -> bool {
                match *other {
                    VerifyingKey {
                        alfa_1: ref __self_1_0,
                        beta_2: ref __self_1_1,
                        gamma_2: ref __self_1_2,
                        delta_2: ref __self_1_3,
                        ic: ref __self_1_4,
                    } => match *self {
                        VerifyingKey {
                            alfa_1: ref __self_0_0,
                            beta_2: ref __self_0_1,
                            gamma_2: ref __self_0_2,
                            delta_2: ref __self_0_3,
                            ic: ref __self_0_4,
                        } => {
                            (*__self_0_0) == (*__self_1_0)
                                && (*__self_0_1) == (*__self_1_1)
                                && (*__self_0_2) == (*__self_1_2)
                                && (*__self_0_3) == (*__self_1_3)
                                && (*__self_0_4) == (*__self_1_4)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &VerifyingKey) -> bool {
                match *other {
                    VerifyingKey {
                        alfa_1: ref __self_1_0,
                        beta_2: ref __self_1_1,
                        gamma_2: ref __self_1_2,
                        delta_2: ref __self_1_3,
                        ic: ref __self_1_4,
                    } => match *self {
                        VerifyingKey {
                            alfa_1: ref __self_0_0,
                            beta_2: ref __self_0_1,
                            gamma_2: ref __self_0_2,
                            delta_2: ref __self_0_3,
                            ic: ref __self_0_4,
                        } => {
                            (*__self_0_0) != (*__self_1_0)
                                || (*__self_0_1) != (*__self_1_1)
                                || (*__self_0_2) != (*__self_1_2)
                                || (*__self_0_3) != (*__self_1_3)
                                || (*__self_0_4) != (*__self_1_4)
                        }
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for VerifyingKey
            where
                G1Point: ethers_core::abi::Tokenize,
                G2Point: ethers_core::abi::Tokenize,
                G2Point: ethers_core::abi::Tokenize,
                G2Point: ethers_core::abi::Tokenize,
                ::std::vec::Vec<G1Point>: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 5usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&5usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        alfa_1: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        beta_2: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        gamma_2: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        delta_2: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        ic: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.alfa_1.into_token(),
                    self.beta_2.into_token(),
                    self.gamma_2.into_token(),
                    self.delta_2.into_token(),
                    self.ic.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for VerifyingKey
            where
                G1Point: ethers_core::abi::Tokenize,
                G2Point: ethers_core::abi::Tokenize,
                G2Point: ethers_core::abi::Tokenize,
                G2Point: ethers_core::abi::Tokenize,
                ::std::vec::Vec<G1Point>: ethers_core::abi::Tokenize,
        {
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for VerifyingKey {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __field3,
                        __field4,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                2u64 => _serde::__private::Ok(__Field::__field2),
                                3u64 => _serde::__private::Ok(__Field::__field3),
                                4u64 => _serde::__private::Ok(__Field::__field4),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "alfa_1" => _serde::__private::Ok(__Field::__field0),
                                "beta_2" => _serde::__private::Ok(__Field::__field1),
                                "gamma_2" => _serde::__private::Ok(__Field::__field2),
                                "delta_2" => _serde::__private::Ok(__Field::__field3),
                                "ic" => _serde::__private::Ok(__Field::__field4),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"alfa_1" => _serde::__private::Ok(__Field::__field0),
                                b"beta_2" => _serde::__private::Ok(__Field::__field1),
                                b"gamma_2" => _serde::__private::Ok(__Field::__field2),
                                b"delta_2" => _serde::__private::Ok(__Field::__field3),
                                b"ic" => _serde::__private::Ok(__Field::__field4),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<VerifyingKey>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = VerifyingKey;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "struct VerifyingKey",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<G1Point>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct VerifyingKey with 5 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<G2Point>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct VerifyingKey with 5 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match match _serde::de::SeqAccess::next_element::<G2Point>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct VerifyingKey with 5 elements",
                                        ),
                                    );
                                }
                            };
                            let __field3 = match match _serde::de::SeqAccess::next_element::<G2Point>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            3usize,
                                            &"struct VerifyingKey with 5 elements",
                                        ),
                                    );
                                }
                            };
                            let __field4 = match match _serde::de::SeqAccess::next_element::<
                                ::std::vec::Vec<G1Point>,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            4usize,
                                            &"struct VerifyingKey with 5 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(VerifyingKey {
                                alfa_1: __field0,
                                beta_2: __field1,
                                gamma_2: __field2,
                                delta_2: __field3,
                                ic: __field4,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<G1Point> =
                                _serde::__private::None;
                            let mut __field1: _serde::__private::Option<G2Point> =
                                _serde::__private::None;
                            let mut __field2: _serde::__private::Option<G2Point> =
                                _serde::__private::None;
                            let mut __field3: _serde::__private::Option<G2Point> =
                                _serde::__private::None;
                            let mut __field4: _serde::__private::Option<::std::vec::Vec<G1Point>> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "alfa_1",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<G1Point>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "beta_2",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<G2Point>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private::Option::is_some(&__field2) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "gamma_2",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<G2Point>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field3 => {
                                        if _serde::__private::Option::is_some(&__field3) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "delta_2",
                                                ),
                                            );
                                        }
                                        __field3 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<G2Point>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field4 => {
                                        if _serde::__private::Option::is_some(&__field4) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "ic",
                                                ),
                                            );
                                        }
                                        __field4 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ::std::vec::Vec<G1Point>,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("alfa_1") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("beta_2") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private::Some(__field2) => __field2,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("gamma_2") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field3 = match __field3 {
                                _serde::__private::Some(__field3) => __field3,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("delta_2") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field4 = match __field4 {
                                _serde::__private::Some(__field4) => __field4,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("ic") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(VerifyingKey {
                                alfa_1: __field0,
                                beta_2: __field1,
                                gamma_2: __field2,
                                delta_2: __field3,
                                ic: __field4,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] =
                        &["alfa_1", "beta_2", "gamma_2", "delta_2", "ic"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "VerifyingKey",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<VerifyingKey>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for VerifyingKey {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "VerifyingKey",
                        false as usize + 1 + 1 + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "alfa_1",
                        &self.alfa_1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "beta_2",
                        &self.beta_2,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "gamma_2",
                        &self.gamma_2,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "delta_2",
                        &self.delta_2,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "ic",
                        &self.ic,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
    }
    assert_tokenizeable::<VerifyingKey>();
    assert_tokenizeable::<G1Point>();
    assert_tokenizeable::<G2Point>();
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const can_generate_internal_structs_multiple: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("can_generate_internal_structs_multiple"),
        ignore: false,
        allow_fail: false,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| {
        test::assert_test_result(can_generate_internal_structs_multiple())
    }),
};
fn can_generate_internal_structs_multiple() {
    use contract::*;
    mod contract {
        use super::*;
        pub mod __shared_types {
            ///`G1Point(uint256,uint256)`
            pub struct G1Point {
                pub x: ethers_core::types::U256,
                pub y: ethers_core::types::U256,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::clone::Clone for G1Point {
                #[inline]
                fn clone(&self) -> G1Point {
                    match *self {
                        G1Point {
                            x: ref __self_0_0,
                            y: ref __self_0_1,
                        } => G1Point {
                            x: ::core::clone::Clone::clone(&(*__self_0_0)),
                            y: ::core::clone::Clone::clone(&(*__self_0_1)),
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::fmt::Debug for G1Point {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    match *self {
                        G1Point {
                            x: ref __self_0_0,
                            y: ref __self_0_1,
                        } => {
                            let debug_trait_builder =
                                &mut ::core::fmt::Formatter::debug_struct(f, "G1Point");
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "x",
                                &&(*__self_0_0),
                            );
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "y",
                                &&(*__self_0_1),
                            );
                            ::core::fmt::DebugStruct::finish(debug_trait_builder)
                        }
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::default::Default for G1Point {
                #[inline]
                fn default() -> G1Point {
                    G1Point {
                        x: ::core::default::Default::default(),
                        y: ::core::default::Default::default(),
                    }
                }
            }
            impl ::core::marker::StructuralEq for G1Point {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::cmp::Eq for G1Point {
                #[inline]
                #[doc(hidden)]
                #[no_coverage]
                fn assert_receiver_is_total_eq(&self) -> () {
                    {
                        let _: ::core::cmp::AssertParamIsEq<ethers_core::types::U256>;
                        let _: ::core::cmp::AssertParamIsEq<ethers_core::types::U256>;
                    }
                }
            }
            impl ::core::marker::StructuralPartialEq for G1Point {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::cmp::PartialEq for G1Point {
                #[inline]
                fn eq(&self, other: &G1Point) -> bool {
                    match *other {
                        G1Point {
                            x: ref __self_1_0,
                            y: ref __self_1_1,
                        } => match *self {
                            G1Point {
                                x: ref __self_0_0,
                                y: ref __self_0_1,
                            } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                        },
                    }
                }
                #[inline]
                fn ne(&self, other: &G1Point) -> bool {
                    match *other {
                        G1Point {
                            x: ref __self_1_0,
                            y: ref __self_1_1,
                        } => match *self {
                            G1Point {
                                x: ref __self_0_0,
                                y: ref __self_0_1,
                            } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                        },
                    }
                }
            }
            impl ethers_core::abi::Tokenizable for G1Point
                where
                    ethers_core::types::U256: ethers_core::abi::Tokenize,
                    ethers_core::types::U256: ethers_core::abi::Tokenize,
            {
                fn from_token(
                    token: ethers_core::abi::Token,
                ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                    where
                        Self: Sized,
                {
                    if let ethers_core::abi::Token::Tuple(tokens) = token {
                        if tokens.len() != 2usize {
                            return Err(ethers_core::abi::InvalidOutputType({
                                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                    &["Expected ", " tokens, got ", ": "],
                                    &match (&2usize, &tokens.len(), &tokens) {
                                        (arg0, arg1, arg2) => [
                                            ::core::fmt::ArgumentV1::new(
                                                arg0,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg1,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg2,
                                                ::core::fmt::Debug::fmt,
                                            ),
                                        ],
                                    },
                                ));
                                res
                            }));
                        }
                        let mut iter = tokens.into_iter();
                        Ok(Self {
                            x: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                            y: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                        })
                    } else {
                        Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected Tuple, got "],
                                &match (&token,) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Debug::fmt,
                                    )],
                                },
                            ));
                            res
                        }))
                    }
                }
                fn into_token(self) -> ethers_core::abi::Token {
                    ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                        self.x.into_token(),
                        self.y.into_token(),
                    ]))
                }
            }
            impl ethers_core::abi::TokenizableItem for G1Point
                where
                    ethers_core::types::U256: ethers_core::abi::Tokenize,
                    ethers_core::types::U256: ethers_core::abi::Tokenize,
            {
            }
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for G1Point {
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                    {
                        #[allow(non_camel_case_types)]
                        enum __Field {
                            __field0,
                            __field1,
                            __ignore,
                        }
                        struct __FieldVisitor;
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private::Formatter,
                            ) -> _serde::__private::fmt::Result {
                                _serde::__private::Formatter::write_str(
                                    __formatter,
                                    "field identifier",
                                )
                            }
                            fn visit_u64<__E>(
                                self,
                                __value: u64,
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    0u64 => _serde::__private::Ok(__Field::__field0),
                                    1u64 => _serde::__private::Ok(__Field::__field1),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(
                                self,
                                __value: &str,
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    "x" => _serde::__private::Ok(__Field::__field0),
                                    "y" => _serde::__private::Ok(__Field::__field1),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_bytes<__E>(
                                self,
                                __value: &[u8],
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    b"x" => _serde::__private::Ok(__Field::__field0),
                                    b"y" => _serde::__private::Ok(__Field::__field1),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                        }
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(
                                __deserializer: __D,
                            ) -> _serde::__private::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                            {
                                _serde::Deserializer::deserialize_identifier(
                                    __deserializer,
                                    __FieldVisitor,
                                )
                            }
                        }
                        struct __Visitor<'de> {
                            marker: _serde::__private::PhantomData<G1Point>,
                            lifetime: _serde::__private::PhantomData<&'de ()>,
                        }
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = G1Point;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private::Formatter,
                            ) -> _serde::__private::fmt::Result {
                                _serde::__private::Formatter::write_str(
                                    __formatter,
                                    "struct G1Point",
                                )
                            }
                            #[inline]
                            fn visit_seq<__A>(
                                self,
                                mut __seq: __A,
                            ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                            {
                                let __field0 = match match _serde::de::SeqAccess::next_element::<
                                    ethers_core::types::U256,
                                >(
                                    &mut __seq
                                ) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                } {
                                    _serde::__private::Some(__value) => __value,
                                    _serde::__private::None => {
                                        return _serde::__private::Err(
                                            _serde::de::Error::invalid_length(
                                                0usize,
                                                &"struct G1Point with 2 elements",
                                            ),
                                        );
                                    }
                                };
                                let __field1 = match match _serde::de::SeqAccess::next_element::<
                                    ethers_core::types::U256,
                                >(
                                    &mut __seq
                                ) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                } {
                                    _serde::__private::Some(__value) => __value,
                                    _serde::__private::None => {
                                        return _serde::__private::Err(
                                            _serde::de::Error::invalid_length(
                                                1usize,
                                                &"struct G1Point with 2 elements",
                                            ),
                                        );
                                    }
                                };
                                _serde::__private::Ok(G1Point {
                                    x: __field0,
                                    y: __field1,
                                })
                            }
                            #[inline]
                            fn visit_map<__A>(
                                self,
                                mut __map: __A,
                            ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                            {
                                let mut __field0: _serde::__private::Option<
                                    ethers_core::types::U256,
                                > = _serde::__private::None;
                                let mut __field1: _serde::__private::Option<
                                    ethers_core::types::U256,
                                > = _serde::__private::None;
                                while let _serde::__private::Some(__key) =
                                match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                                {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private::Option::is_some(&__field0) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("x")) ;
                                            }
                                            __field0 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<
                                                    ethers_core::types::U256,
                                                >(
                                                    &mut __map
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private::Option::is_some(&__field1) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("y")) ;
                                            }
                                            __field1 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<
                                                    ethers_core::types::U256,
                                                >(
                                                    &mut __map
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        _ => {
                                            let _ = match _serde::de::MapAccess::next_value::<
                                                _serde::de::IgnoredAny,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            };
                                        }
                                    }
                                }
                                let __field0 = match __field0 {
                                    _serde::__private::Some(__field0) => __field0,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("x") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                let __field1 = match __field1 {
                                    _serde::__private::Some(__field1) => __field1,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("y") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                _serde::__private::Ok(G1Point {
                                    x: __field0,
                                    y: __field1,
                                })
                            }
                        }
                        const FIELDS: &'static [&'static str] = &["x", "y"];
                        _serde::Deserializer::deserialize_struct(
                            __deserializer,
                            "G1Point",
                            FIELDS,
                            __Visitor {
                                marker: _serde::__private::PhantomData::<G1Point>,
                                lifetime: _serde::__private::PhantomData,
                            },
                        )
                    }
                }
            };
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl _serde::Serialize for G1Point {
                    fn serialize<__S>(
                        &self,
                        __serializer: __S,
                    ) -> _serde::__private::Result<__S::Ok, __S::Error>
                        where
                            __S: _serde::Serializer,
                    {
                        let mut __serde_state = match _serde::Serializer::serialize_struct(
                            __serializer,
                            "G1Point",
                            false as usize + 1 + 1,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "x",
                            &self.x,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "y",
                            &self.y,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
            ///`VerifyingKey((uint256,uint256),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256,uint256)[])`
            pub struct VerifyingKey {
                pub alfa_1: G1Point,
                pub beta_2: G2Point,
                pub gamma_2: G2Point,
                pub delta_2: G2Point,
                pub ic: ::std::vec::Vec<G1Point>,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::clone::Clone for VerifyingKey {
                #[inline]
                fn clone(&self) -> VerifyingKey {
                    match *self {
                        VerifyingKey {
                            alfa_1: ref __self_0_0,
                            beta_2: ref __self_0_1,
                            gamma_2: ref __self_0_2,
                            delta_2: ref __self_0_3,
                            ic: ref __self_0_4,
                        } => VerifyingKey {
                            alfa_1: ::core::clone::Clone::clone(&(*__self_0_0)),
                            beta_2: ::core::clone::Clone::clone(&(*__self_0_1)),
                            gamma_2: ::core::clone::Clone::clone(&(*__self_0_2)),
                            delta_2: ::core::clone::Clone::clone(&(*__self_0_3)),
                            ic: ::core::clone::Clone::clone(&(*__self_0_4)),
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::fmt::Debug for VerifyingKey {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    match *self {
                        VerifyingKey {
                            alfa_1: ref __self_0_0,
                            beta_2: ref __self_0_1,
                            gamma_2: ref __self_0_2,
                            delta_2: ref __self_0_3,
                            ic: ref __self_0_4,
                        } => {
                            let debug_trait_builder =
                                &mut ::core::fmt::Formatter::debug_struct(f, "VerifyingKey");
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "alfa_1",
                                &&(*__self_0_0),
                            );
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "beta_2",
                                &&(*__self_0_1),
                            );
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "gamma_2",
                                &&(*__self_0_2),
                            );
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "delta_2",
                                &&(*__self_0_3),
                            );
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "ic",
                                &&(*__self_0_4),
                            );
                            ::core::fmt::DebugStruct::finish(debug_trait_builder)
                        }
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::default::Default for VerifyingKey {
                #[inline]
                fn default() -> VerifyingKey {
                    VerifyingKey {
                        alfa_1: ::core::default::Default::default(),
                        beta_2: ::core::default::Default::default(),
                        gamma_2: ::core::default::Default::default(),
                        delta_2: ::core::default::Default::default(),
                        ic: ::core::default::Default::default(),
                    }
                }
            }
            impl ::core::marker::StructuralEq for VerifyingKey {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::cmp::Eq for VerifyingKey {
                #[inline]
                #[doc(hidden)]
                #[no_coverage]
                fn assert_receiver_is_total_eq(&self) -> () {
                    {
                        let _: ::core::cmp::AssertParamIsEq<G1Point>;
                        let _: ::core::cmp::AssertParamIsEq<G2Point>;
                        let _: ::core::cmp::AssertParamIsEq<G2Point>;
                        let _: ::core::cmp::AssertParamIsEq<G2Point>;
                        let _: ::core::cmp::AssertParamIsEq<::std::vec::Vec<G1Point>>;
                    }
                }
            }
            impl ::core::marker::StructuralPartialEq for VerifyingKey {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::cmp::PartialEq for VerifyingKey {
                #[inline]
                fn eq(&self, other: &VerifyingKey) -> bool {
                    match *other {
                        VerifyingKey {
                            alfa_1: ref __self_1_0,
                            beta_2: ref __self_1_1,
                            gamma_2: ref __self_1_2,
                            delta_2: ref __self_1_3,
                            ic: ref __self_1_4,
                        } => match *self {
                            VerifyingKey {
                                alfa_1: ref __self_0_0,
                                beta_2: ref __self_0_1,
                                gamma_2: ref __self_0_2,
                                delta_2: ref __self_0_3,
                                ic: ref __self_0_4,
                            } => {
                                (*__self_0_0) == (*__self_1_0)
                                    && (*__self_0_1) == (*__self_1_1)
                                    && (*__self_0_2) == (*__self_1_2)
                                    && (*__self_0_3) == (*__self_1_3)
                                    && (*__self_0_4) == (*__self_1_4)
                            }
                        },
                    }
                }
                #[inline]
                fn ne(&self, other: &VerifyingKey) -> bool {
                    match *other {
                        VerifyingKey {
                            alfa_1: ref __self_1_0,
                            beta_2: ref __self_1_1,
                            gamma_2: ref __self_1_2,
                            delta_2: ref __self_1_3,
                            ic: ref __self_1_4,
                        } => match *self {
                            VerifyingKey {
                                alfa_1: ref __self_0_0,
                                beta_2: ref __self_0_1,
                                gamma_2: ref __self_0_2,
                                delta_2: ref __self_0_3,
                                ic: ref __self_0_4,
                            } => {
                                (*__self_0_0) != (*__self_1_0)
                                    || (*__self_0_1) != (*__self_1_1)
                                    || (*__self_0_2) != (*__self_1_2)
                                    || (*__self_0_3) != (*__self_1_3)
                                    || (*__self_0_4) != (*__self_1_4)
                            }
                        },
                    }
                }
            }
            impl ethers_core::abi::Tokenizable for VerifyingKey
                where
                    G1Point: ethers_core::abi::Tokenize,
                    G2Point: ethers_core::abi::Tokenize,
                    G2Point: ethers_core::abi::Tokenize,
                    G2Point: ethers_core::abi::Tokenize,
                    ::std::vec::Vec<G1Point>: ethers_core::abi::Tokenize,
            {
                fn from_token(
                    token: ethers_core::abi::Token,
                ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                    where
                        Self: Sized,
                {
                    if let ethers_core::abi::Token::Tuple(tokens) = token {
                        if tokens.len() != 5usize {
                            return Err(ethers_core::abi::InvalidOutputType({
                                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                    &["Expected ", " tokens, got ", ": "],
                                    &match (&5usize, &tokens.len(), &tokens) {
                                        (arg0, arg1, arg2) => [
                                            ::core::fmt::ArgumentV1::new(
                                                arg0,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg1,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg2,
                                                ::core::fmt::Debug::fmt,
                                            ),
                                        ],
                                    },
                                ));
                                res
                            }));
                        }
                        let mut iter = tokens.into_iter();
                        Ok(Self {
                            alfa_1: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                            beta_2: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                            gamma_2: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                            delta_2: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                            ic: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                        })
                    } else {
                        Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected Tuple, got "],
                                &match (&token,) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Debug::fmt,
                                    )],
                                },
                            ));
                            res
                        }))
                    }
                }
                fn into_token(self) -> ethers_core::abi::Token {
                    ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                        self.alfa_1.into_token(),
                        self.beta_2.into_token(),
                        self.gamma_2.into_token(),
                        self.delta_2.into_token(),
                        self.ic.into_token(),
                    ]))
                }
            }
            impl ethers_core::abi::TokenizableItem for VerifyingKey
                where
                    G1Point: ethers_core::abi::Tokenize,
                    G2Point: ethers_core::abi::Tokenize,
                    G2Point: ethers_core::abi::Tokenize,
                    G2Point: ethers_core::abi::Tokenize,
                    ::std::vec::Vec<G1Point>: ethers_core::abi::Tokenize,
            {
            }
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for VerifyingKey {
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                    {
                        #[allow(non_camel_case_types)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __field3,
                            __field4,
                            __ignore,
                        }
                        struct __FieldVisitor;
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private::Formatter,
                            ) -> _serde::__private::fmt::Result {
                                _serde::__private::Formatter::write_str(
                                    __formatter,
                                    "field identifier",
                                )
                            }
                            fn visit_u64<__E>(
                                self,
                                __value: u64,
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    0u64 => _serde::__private::Ok(__Field::__field0),
                                    1u64 => _serde::__private::Ok(__Field::__field1),
                                    2u64 => _serde::__private::Ok(__Field::__field2),
                                    3u64 => _serde::__private::Ok(__Field::__field3),
                                    4u64 => _serde::__private::Ok(__Field::__field4),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(
                                self,
                                __value: &str,
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    "alfa_1" => _serde::__private::Ok(__Field::__field0),
                                    "beta_2" => _serde::__private::Ok(__Field::__field1),
                                    "gamma_2" => _serde::__private::Ok(__Field::__field2),
                                    "delta_2" => _serde::__private::Ok(__Field::__field3),
                                    "ic" => _serde::__private::Ok(__Field::__field4),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_bytes<__E>(
                                self,
                                __value: &[u8],
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    b"alfa_1" => _serde::__private::Ok(__Field::__field0),
                                    b"beta_2" => _serde::__private::Ok(__Field::__field1),
                                    b"gamma_2" => _serde::__private::Ok(__Field::__field2),
                                    b"delta_2" => _serde::__private::Ok(__Field::__field3),
                                    b"ic" => _serde::__private::Ok(__Field::__field4),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                        }
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(
                                __deserializer: __D,
                            ) -> _serde::__private::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                            {
                                _serde::Deserializer::deserialize_identifier(
                                    __deserializer,
                                    __FieldVisitor,
                                )
                            }
                        }
                        struct __Visitor<'de> {
                            marker: _serde::__private::PhantomData<VerifyingKey>,
                            lifetime: _serde::__private::PhantomData<&'de ()>,
                        }
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = VerifyingKey;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private::Formatter,
                            ) -> _serde::__private::fmt::Result {
                                _serde::__private::Formatter::write_str(
                                    __formatter,
                                    "struct VerifyingKey",
                                )
                            }
                            #[inline]
                            fn visit_seq<__A>(
                                self,
                                mut __seq: __A,
                            ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                            {
                                let __field0 =
                                    match match _serde::de::SeqAccess::next_element::<G1Point>(
                                        &mut __seq,
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    0usize,
                                                    &"struct VerifyingKey with 5 elements",
                                                ),
                                            );
                                        }
                                    };
                                let __field1 =
                                    match match _serde::de::SeqAccess::next_element::<G2Point>(
                                        &mut __seq,
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    1usize,
                                                    &"struct VerifyingKey with 5 elements",
                                                ),
                                            );
                                        }
                                    };
                                let __field2 =
                                    match match _serde::de::SeqAccess::next_element::<G2Point>(
                                        &mut __seq,
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    2usize,
                                                    &"struct VerifyingKey with 5 elements",
                                                ),
                                            );
                                        }
                                    };
                                let __field3 =
                                    match match _serde::de::SeqAccess::next_element::<G2Point>(
                                        &mut __seq,
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    3usize,
                                                    &"struct VerifyingKey with 5 elements",
                                                ),
                                            );
                                        }
                                    };
                                let __field4 = match match _serde::de::SeqAccess::next_element::<
                                    ::std::vec::Vec<G1Point>,
                                >(
                                    &mut __seq
                                ) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                } {
                                    _serde::__private::Some(__value) => __value,
                                    _serde::__private::None => {
                                        return _serde::__private::Err(
                                            _serde::de::Error::invalid_length(
                                                4usize,
                                                &"struct VerifyingKey with 5 elements",
                                            ),
                                        );
                                    }
                                };
                                _serde::__private::Ok(VerifyingKey {
                                    alfa_1: __field0,
                                    beta_2: __field1,
                                    gamma_2: __field2,
                                    delta_2: __field3,
                                    ic: __field4,
                                })
                            }
                            #[inline]
                            fn visit_map<__A>(
                                self,
                                mut __map: __A,
                            ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                            {
                                let mut __field0: _serde::__private::Option<G1Point> =
                                    _serde::__private::None;
                                let mut __field1: _serde::__private::Option<G2Point> =
                                    _serde::__private::None;
                                let mut __field2: _serde::__private::Option<G2Point> =
                                    _serde::__private::None;
                                let mut __field3: _serde::__private::Option<G2Point> =
                                    _serde::__private::None;
                                let mut __field4: _serde::__private::Option<
                                    ::std::vec::Vec<G1Point>,
                                > = _serde::__private::None;
                                while let _serde::__private::Some(__key) =
                                match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                                {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private::Option::is_some(&__field0) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("alfa_1")) ;
                                            }
                                            __field0 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<G1Point>(
                                                    &mut __map,
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private::Option::is_some(&__field1) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("beta_2")) ;
                                            }
                                            __field1 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<G2Point>(
                                                    &mut __map,
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private::Option::is_some(&__field2) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("gamma_2")) ;
                                            }
                                            __field2 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<G2Point>(
                                                    &mut __map,
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        __Field::__field3 => {
                                            if _serde::__private::Option::is_some(&__field3) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("delta_2")) ;
                                            }
                                            __field3 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<G2Point>(
                                                    &mut __map,
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        __Field::__field4 => {
                                            if _serde::__private::Option::is_some(&__field4) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("ic")) ;
                                            }
                                            __field4 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<
                                                    ::std::vec::Vec<G1Point>,
                                                >(
                                                    &mut __map
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        _ => {
                                            let _ = match _serde::de::MapAccess::next_value::<
                                                _serde::de::IgnoredAny,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            };
                                        }
                                    }
                                }
                                let __field0 = match __field0 {
                                    _serde::__private::Some(__field0) => __field0,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("alfa_1") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                let __field1 = match __field1 {
                                    _serde::__private::Some(__field1) => __field1,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("beta_2") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                let __field2 = match __field2 {
                                    _serde::__private::Some(__field2) => __field2,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("gamma_2") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                let __field3 = match __field3 {
                                    _serde::__private::Some(__field3) => __field3,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("delta_2") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                let __field4 = match __field4 {
                                    _serde::__private::Some(__field4) => __field4,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("ic") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                _serde::__private::Ok(VerifyingKey {
                                    alfa_1: __field0,
                                    beta_2: __field1,
                                    gamma_2: __field2,
                                    delta_2: __field3,
                                    ic: __field4,
                                })
                            }
                        }
                        const FIELDS: &'static [&'static str] =
                            &["alfa_1", "beta_2", "gamma_2", "delta_2", "ic"];
                        _serde::Deserializer::deserialize_struct(
                            __deserializer,
                            "VerifyingKey",
                            FIELDS,
                            __Visitor {
                                marker: _serde::__private::PhantomData::<VerifyingKey>,
                                lifetime: _serde::__private::PhantomData,
                            },
                        )
                    }
                }
            };
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl _serde::Serialize for VerifyingKey {
                    fn serialize<__S>(
                        &self,
                        __serializer: __S,
                    ) -> _serde::__private::Result<__S::Ok, __S::Error>
                        where
                            __S: _serde::Serializer,
                    {
                        let mut __serde_state = match _serde::Serializer::serialize_struct(
                            __serializer,
                            "VerifyingKey",
                            false as usize + 1 + 1 + 1 + 1 + 1,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "alfa_1",
                            &self.alfa_1,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "beta_2",
                            &self.beta_2,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "gamma_2",
                            &self.gamma_2,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "delta_2",
                            &self.delta_2,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "ic",
                            &self.ic,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
            ///`Proof((uint256,uint256),(uint256[2],uint256[2]),(uint256,uint256))`
            pub struct Proof {
                pub a: G1Point,
                pub b: G2Point,
                pub c: G1Point,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::clone::Clone for Proof {
                #[inline]
                fn clone(&self) -> Proof {
                    match *self {
                        Proof {
                            a: ref __self_0_0,
                            b: ref __self_0_1,
                            c: ref __self_0_2,
                        } => Proof {
                            a: ::core::clone::Clone::clone(&(*__self_0_0)),
                            b: ::core::clone::Clone::clone(&(*__self_0_1)),
                            c: ::core::clone::Clone::clone(&(*__self_0_2)),
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::fmt::Debug for Proof {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    match *self {
                        Proof {
                            a: ref __self_0_0,
                            b: ref __self_0_1,
                            c: ref __self_0_2,
                        } => {
                            let debug_trait_builder =
                                &mut ::core::fmt::Formatter::debug_struct(f, "Proof");
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "a",
                                &&(*__self_0_0),
                            );
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "b",
                                &&(*__self_0_1),
                            );
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "c",
                                &&(*__self_0_2),
                            );
                            ::core::fmt::DebugStruct::finish(debug_trait_builder)
                        }
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::default::Default for Proof {
                #[inline]
                fn default() -> Proof {
                    Proof {
                        a: ::core::default::Default::default(),
                        b: ::core::default::Default::default(),
                        c: ::core::default::Default::default(),
                    }
                }
            }
            impl ::core::marker::StructuralEq for Proof {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::cmp::Eq for Proof {
                #[inline]
                #[doc(hidden)]
                #[no_coverage]
                fn assert_receiver_is_total_eq(&self) -> () {
                    {
                        let _: ::core::cmp::AssertParamIsEq<G1Point>;
                        let _: ::core::cmp::AssertParamIsEq<G2Point>;
                        let _: ::core::cmp::AssertParamIsEq<G1Point>;
                    }
                }
            }
            impl ::core::marker::StructuralPartialEq for Proof {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::cmp::PartialEq for Proof {
                #[inline]
                fn eq(&self, other: &Proof) -> bool {
                    match *other {
                        Proof {
                            a: ref __self_1_0,
                            b: ref __self_1_1,
                            c: ref __self_1_2,
                        } => match *self {
                            Proof {
                                a: ref __self_0_0,
                                b: ref __self_0_1,
                                c: ref __self_0_2,
                            } => {
                                (*__self_0_0) == (*__self_1_0)
                                    && (*__self_0_1) == (*__self_1_1)
                                    && (*__self_0_2) == (*__self_1_2)
                            }
                        },
                    }
                }
                #[inline]
                fn ne(&self, other: &Proof) -> bool {
                    match *other {
                        Proof {
                            a: ref __self_1_0,
                            b: ref __self_1_1,
                            c: ref __self_1_2,
                        } => match *self {
                            Proof {
                                a: ref __self_0_0,
                                b: ref __self_0_1,
                                c: ref __self_0_2,
                            } => {
                                (*__self_0_0) != (*__self_1_0)
                                    || (*__self_0_1) != (*__self_1_1)
                                    || (*__self_0_2) != (*__self_1_2)
                            }
                        },
                    }
                }
            }
            impl ethers_core::abi::Tokenizable for Proof
                where
                    G1Point: ethers_core::abi::Tokenize,
                    G2Point: ethers_core::abi::Tokenize,
                    G1Point: ethers_core::abi::Tokenize,
            {
                fn from_token(
                    token: ethers_core::abi::Token,
                ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                    where
                        Self: Sized,
                {
                    if let ethers_core::abi::Token::Tuple(tokens) = token {
                        if tokens.len() != 3usize {
                            return Err(ethers_core::abi::InvalidOutputType({
                                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                    &["Expected ", " tokens, got ", ": "],
                                    &match (&3usize, &tokens.len(), &tokens) {
                                        (arg0, arg1, arg2) => [
                                            ::core::fmt::ArgumentV1::new(
                                                arg0,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg1,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg2,
                                                ::core::fmt::Debug::fmt,
                                            ),
                                        ],
                                    },
                                ));
                                res
                            }));
                        }
                        let mut iter = tokens.into_iter();
                        Ok(Self {
                            a: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                            b: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                            c: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                        })
                    } else {
                        Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected Tuple, got "],
                                &match (&token,) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Debug::fmt,
                                    )],
                                },
                            ));
                            res
                        }))
                    }
                }
                fn into_token(self) -> ethers_core::abi::Token {
                    ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                        self.a.into_token(),
                        self.b.into_token(),
                        self.c.into_token(),
                    ]))
                }
            }
            impl ethers_core::abi::TokenizableItem for Proof
                where
                    G1Point: ethers_core::abi::Tokenize,
                    G2Point: ethers_core::abi::Tokenize,
                    G1Point: ethers_core::abi::Tokenize,
            {
            }
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for Proof {
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                    {
                        #[allow(non_camel_case_types)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __ignore,
                        }
                        struct __FieldVisitor;
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private::Formatter,
                            ) -> _serde::__private::fmt::Result {
                                _serde::__private::Formatter::write_str(
                                    __formatter,
                                    "field identifier",
                                )
                            }
                            fn visit_u64<__E>(
                                self,
                                __value: u64,
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    0u64 => _serde::__private::Ok(__Field::__field0),
                                    1u64 => _serde::__private::Ok(__Field::__field1),
                                    2u64 => _serde::__private::Ok(__Field::__field2),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(
                                self,
                                __value: &str,
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    "a" => _serde::__private::Ok(__Field::__field0),
                                    "b" => _serde::__private::Ok(__Field::__field1),
                                    "c" => _serde::__private::Ok(__Field::__field2),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_bytes<__E>(
                                self,
                                __value: &[u8],
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    b"a" => _serde::__private::Ok(__Field::__field0),
                                    b"b" => _serde::__private::Ok(__Field::__field1),
                                    b"c" => _serde::__private::Ok(__Field::__field2),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                        }
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(
                                __deserializer: __D,
                            ) -> _serde::__private::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                            {
                                _serde::Deserializer::deserialize_identifier(
                                    __deserializer,
                                    __FieldVisitor,
                                )
                            }
                        }
                        struct __Visitor<'de> {
                            marker: _serde::__private::PhantomData<Proof>,
                            lifetime: _serde::__private::PhantomData<&'de ()>,
                        }
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = Proof;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private::Formatter,
                            ) -> _serde::__private::fmt::Result {
                                _serde::__private::Formatter::write_str(__formatter, "struct Proof")
                            }
                            #[inline]
                            fn visit_seq<__A>(
                                self,
                                mut __seq: __A,
                            ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                            {
                                let __field0 =
                                    match match _serde::de::SeqAccess::next_element::<G1Point>(
                                        &mut __seq,
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    0usize,
                                                    &"struct Proof with 3 elements",
                                                ),
                                            );
                                        }
                                    };
                                let __field1 =
                                    match match _serde::de::SeqAccess::next_element::<G2Point>(
                                        &mut __seq,
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    1usize,
                                                    &"struct Proof with 3 elements",
                                                ),
                                            );
                                        }
                                    };
                                let __field2 =
                                    match match _serde::de::SeqAccess::next_element::<G1Point>(
                                        &mut __seq,
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    2usize,
                                                    &"struct Proof with 3 elements",
                                                ),
                                            );
                                        }
                                    };
                                _serde::__private::Ok(Proof {
                                    a: __field0,
                                    b: __field1,
                                    c: __field2,
                                })
                            }
                            #[inline]
                            fn visit_map<__A>(
                                self,
                                mut __map: __A,
                            ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                            {
                                let mut __field0: _serde::__private::Option<G1Point> =
                                    _serde::__private::None;
                                let mut __field1: _serde::__private::Option<G2Point> =
                                    _serde::__private::None;
                                let mut __field2: _serde::__private::Option<G1Point> =
                                    _serde::__private::None;
                                while let _serde::__private::Some(__key) =
                                match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                                {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private::Option::is_some(&__field0) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("a")) ;
                                            }
                                            __field0 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<G1Point>(
                                                    &mut __map,
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private::Option::is_some(&__field1) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("b")) ;
                                            }
                                            __field1 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<G2Point>(
                                                    &mut __map,
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private::Option::is_some(&__field2) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("c")) ;
                                            }
                                            __field2 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<G1Point>(
                                                    &mut __map,
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        _ => {
                                            let _ = match _serde::de::MapAccess::next_value::<
                                                _serde::de::IgnoredAny,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            };
                                        }
                                    }
                                }
                                let __field0 = match __field0 {
                                    _serde::__private::Some(__field0) => __field0,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("a") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                let __field1 = match __field1 {
                                    _serde::__private::Some(__field1) => __field1,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("b") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                let __field2 = match __field2 {
                                    _serde::__private::Some(__field2) => __field2,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("c") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                _serde::__private::Ok(Proof {
                                    a: __field0,
                                    b: __field1,
                                    c: __field2,
                                })
                            }
                        }
                        const FIELDS: &'static [&'static str] = &["a", "b", "c"];
                        _serde::Deserializer::deserialize_struct(
                            __deserializer,
                            "Proof",
                            FIELDS,
                            __Visitor {
                                marker: _serde::__private::PhantomData::<Proof>,
                                lifetime: _serde::__private::PhantomData,
                            },
                        )
                    }
                }
            };
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl _serde::Serialize for Proof {
                    fn serialize<__S>(
                        &self,
                        __serializer: __S,
                    ) -> _serde::__private::Result<__S::Ok, __S::Error>
                        where
                            __S: _serde::Serializer,
                    {
                        let mut __serde_state = match _serde::Serializer::serialize_struct(
                            __serializer,
                            "Proof",
                            false as usize + 1 + 1 + 1,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "a",
                            &self.a,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "b",
                            &self.b,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "c",
                            &self.c,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
            ///`G2Point(uint256[2],uint256[2])`
            pub struct G2Point {
                pub x: [ethers_core::types::U256; 2],
                pub y: [ethers_core::types::U256; 2],
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::clone::Clone for G2Point {
                #[inline]
                fn clone(&self) -> G2Point {
                    match *self {
                        G2Point {
                            x: ref __self_0_0,
                            y: ref __self_0_1,
                        } => G2Point {
                            x: ::core::clone::Clone::clone(&(*__self_0_0)),
                            y: ::core::clone::Clone::clone(&(*__self_0_1)),
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::fmt::Debug for G2Point {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    match *self {
                        G2Point {
                            x: ref __self_0_0,
                            y: ref __self_0_1,
                        } => {
                            let debug_trait_builder =
                                &mut ::core::fmt::Formatter::debug_struct(f, "G2Point");
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "x",
                                &&(*__self_0_0),
                            );
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "y",
                                &&(*__self_0_1),
                            );
                            ::core::fmt::DebugStruct::finish(debug_trait_builder)
                        }
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::default::Default for G2Point {
                #[inline]
                fn default() -> G2Point {
                    G2Point {
                        x: ::core::default::Default::default(),
                        y: ::core::default::Default::default(),
                    }
                }
            }
            impl ::core::marker::StructuralEq for G2Point {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::cmp::Eq for G2Point {
                #[inline]
                #[doc(hidden)]
                #[no_coverage]
                fn assert_receiver_is_total_eq(&self) -> () {
                    {
                        let _: ::core::cmp::AssertParamIsEq<[ethers_core::types::U256; 2]>;
                        let _: ::core::cmp::AssertParamIsEq<[ethers_core::types::U256; 2]>;
                    }
                }
            }
            impl ::core::marker::StructuralPartialEq for G2Point {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::cmp::PartialEq for G2Point {
                #[inline]
                fn eq(&self, other: &G2Point) -> bool {
                    match *other {
                        G2Point {
                            x: ref __self_1_0,
                            y: ref __self_1_1,
                        } => match *self {
                            G2Point {
                                x: ref __self_0_0,
                                y: ref __self_0_1,
                            } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                        },
                    }
                }
                #[inline]
                fn ne(&self, other: &G2Point) -> bool {
                    match *other {
                        G2Point {
                            x: ref __self_1_0,
                            y: ref __self_1_1,
                        } => match *self {
                            G2Point {
                                x: ref __self_0_0,
                                y: ref __self_0_1,
                            } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                        },
                    }
                }
            }
            impl ethers_core::abi::Tokenizable for G2Point
                where
                    [ethers_core::types::U256; 2]: ethers_core::abi::Tokenize,
                    [ethers_core::types::U256; 2]: ethers_core::abi::Tokenize,
            {
                fn from_token(
                    token: ethers_core::abi::Token,
                ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                    where
                        Self: Sized,
                {
                    if let ethers_core::abi::Token::Tuple(tokens) = token {
                        if tokens.len() != 2usize {
                            return Err(ethers_core::abi::InvalidOutputType({
                                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                    &["Expected ", " tokens, got ", ": "],
                                    &match (&2usize, &tokens.len(), &tokens) {
                                        (arg0, arg1, arg2) => [
                                            ::core::fmt::ArgumentV1::new(
                                                arg0,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg1,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg2,
                                                ::core::fmt::Debug::fmt,
                                            ),
                                        ],
                                    },
                                ));
                                res
                            }));
                        }
                        let mut iter = tokens.into_iter();
                        Ok(Self {
                            x: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                            y: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                        })
                    } else {
                        Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected Tuple, got "],
                                &match (&token,) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Debug::fmt,
                                    )],
                                },
                            ));
                            res
                        }))
                    }
                }
                fn into_token(self) -> ethers_core::abi::Token {
                    ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                        self.x.into_token(),
                        self.y.into_token(),
                    ]))
                }
            }
            impl ethers_core::abi::TokenizableItem for G2Point
                where
                    [ethers_core::types::U256; 2]: ethers_core::abi::Tokenize,
                    [ethers_core::types::U256; 2]: ethers_core::abi::Tokenize,
            {
            }
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for G2Point {
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                    {
                        #[allow(non_camel_case_types)]
                        enum __Field {
                            __field0,
                            __field1,
                            __ignore,
                        }
                        struct __FieldVisitor;
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private::Formatter,
                            ) -> _serde::__private::fmt::Result {
                                _serde::__private::Formatter::write_str(
                                    __formatter,
                                    "field identifier",
                                )
                            }
                            fn visit_u64<__E>(
                                self,
                                __value: u64,
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    0u64 => _serde::__private::Ok(__Field::__field0),
                                    1u64 => _serde::__private::Ok(__Field::__field1),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(
                                self,
                                __value: &str,
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    "x" => _serde::__private::Ok(__Field::__field0),
                                    "y" => _serde::__private::Ok(__Field::__field1),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_bytes<__E>(
                                self,
                                __value: &[u8],
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    b"x" => _serde::__private::Ok(__Field::__field0),
                                    b"y" => _serde::__private::Ok(__Field::__field1),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                        }
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(
                                __deserializer: __D,
                            ) -> _serde::__private::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                            {
                                _serde::Deserializer::deserialize_identifier(
                                    __deserializer,
                                    __FieldVisitor,
                                )
                            }
                        }
                        struct __Visitor<'de> {
                            marker: _serde::__private::PhantomData<G2Point>,
                            lifetime: _serde::__private::PhantomData<&'de ()>,
                        }
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = G2Point;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private::Formatter,
                            ) -> _serde::__private::fmt::Result {
                                _serde::__private::Formatter::write_str(
                                    __formatter,
                                    "struct G2Point",
                                )
                            }
                            #[inline]
                            fn visit_seq<__A>(
                                self,
                                mut __seq: __A,
                            ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                            {
                                let __field0 = match match _serde::de::SeqAccess::next_element::<
                                    [ethers_core::types::U256; 2],
                                >(
                                    &mut __seq
                                ) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                } {
                                    _serde::__private::Some(__value) => __value,
                                    _serde::__private::None => {
                                        return _serde::__private::Err(
                                            _serde::de::Error::invalid_length(
                                                0usize,
                                                &"struct G2Point with 2 elements",
                                            ),
                                        );
                                    }
                                };
                                let __field1 = match match _serde::de::SeqAccess::next_element::<
                                    [ethers_core::types::U256; 2],
                                >(
                                    &mut __seq
                                ) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                } {
                                    _serde::__private::Some(__value) => __value,
                                    _serde::__private::None => {
                                        return _serde::__private::Err(
                                            _serde::de::Error::invalid_length(
                                                1usize,
                                                &"struct G2Point with 2 elements",
                                            ),
                                        );
                                    }
                                };
                                _serde::__private::Ok(G2Point {
                                    x: __field0,
                                    y: __field1,
                                })
                            }
                            #[inline]
                            fn visit_map<__A>(
                                self,
                                mut __map: __A,
                            ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                            {
                                let mut __field0: _serde::__private::Option<
                                    [ethers_core::types::U256; 2],
                                > = _serde::__private::None;
                                let mut __field1: _serde::__private::Option<
                                    [ethers_core::types::U256; 2],
                                > = _serde::__private::None;
                                while let _serde::__private::Some(__key) =
                                match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                                {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private::Option::is_some(&__field0) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("x")) ;
                                            }
                                            __field0 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<
                                                    [ethers_core::types::U256; 2],
                                                >(
                                                    &mut __map
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private::Option::is_some(&__field1) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("y")) ;
                                            }
                                            __field1 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<
                                                    [ethers_core::types::U256; 2],
                                                >(
                                                    &mut __map
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        _ => {
                                            let _ = match _serde::de::MapAccess::next_value::<
                                                _serde::de::IgnoredAny,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            };
                                        }
                                    }
                                }
                                let __field0 = match __field0 {
                                    _serde::__private::Some(__field0) => __field0,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("x") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                let __field1 = match __field1 {
                                    _serde::__private::Some(__field1) => __field1,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("y") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                _serde::__private::Ok(G2Point {
                                    x: __field0,
                                    y: __field1,
                                })
                            }
                        }
                        const FIELDS: &'static [&'static str] = &["x", "y"];
                        _serde::Deserializer::deserialize_struct(
                            __deserializer,
                            "G2Point",
                            FIELDS,
                            __Visitor {
                                marker: _serde::__private::PhantomData::<G2Point>,
                                lifetime: _serde::__private::PhantomData,
                            },
                        )
                    }
                }
            };
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl _serde::Serialize for G2Point {
                    fn serialize<__S>(
                        &self,
                        __serializer: __S,
                    ) -> _serde::__private::Result<__S::Ok, __S::Error>
                        where
                            __S: _serde::Serializer,
                    {
                        let mut __serde_state = match _serde::Serializer::serialize_struct(
                            __serializer,
                            "G2Point",
                            false as usize + 1 + 1,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "x",
                            &self.x,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "y",
                            &self.y,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        }
        pub use verifiercontract_mod::*;
        #[allow(clippy::too_many_arguments)]
        mod verifiercontract_mod {
            #![allow(clippy::enum_variant_names)]
            #![allow(dead_code)]
            #![allow(clippy::type_complexity)]
            #![allow(unused_imports)]
            ///VerifierContract was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
            use std::sync::Arc;
            use ethers_core::{
                abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
                types::*,
            };
            use ethers_contract::{
                Contract,
                builders::{ContractCall, Event},
                Lazy,
            };
            use ethers_providers::Middleware;
            pub use super::__shared_types::*;
            pub static VERIFIERCONTRACT_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
                ethers_contract::Lazy::new(|| {
                    serde_json :: from_str ("[\n  {\n    \"inputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"constructor\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"uint256[]\",\n        \"name\": \"input\",\n        \"type\": \"uint256[]\"\n      },\n      {\n        \"components\": [\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"X\",\n                \"type\": \"uint256\"\n              },\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"Y\",\n                \"type\": \"uint256\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G1Point\",\n            \"name\": \"A\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"X\",\n                \"type\": \"uint256[2]\"\n              },\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"Y\",\n                \"type\": \"uint256[2]\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G2Point\",\n            \"name\": \"B\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"X\",\n                \"type\": \"uint256\"\n              },\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"Y\",\n                \"type\": \"uint256\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G1Point\",\n            \"name\": \"C\",\n            \"type\": \"tuple\"\n          }\n        ],\n        \"internalType\": \"struct Verifier.Proof\",\n        \"name\": \"proof\",\n        \"type\": \"tuple\"\n      },\n      {\n        \"components\": [\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"X\",\n                \"type\": \"uint256\"\n              },\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"Y\",\n                \"type\": \"uint256\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G1Point\",\n            \"name\": \"alfa1\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"X\",\n                \"type\": \"uint256[2]\"\n              },\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"Y\",\n                \"type\": \"uint256[2]\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G2Point\",\n            \"name\": \"beta2\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"X\",\n                \"type\": \"uint256[2]\"\n              },\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"Y\",\n                \"type\": \"uint256[2]\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G2Point\",\n            \"name\": \"gamma2\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"X\",\n                \"type\": \"uint256[2]\"\n              },\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"Y\",\n                \"type\": \"uint256[2]\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G2Point\",\n            \"name\": \"delta2\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"X\",\n                \"type\": \"uint256\"\n              },\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"Y\",\n                \"type\": \"uint256\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G1Point[]\",\n            \"name\": \"IC\",\n            \"type\": \"tuple[]\"\n          }\n        ],\n        \"internalType\": \"struct Verifier.VerifyingKey\",\n        \"name\": \"vk\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"name\": \"verify\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  }\n]\n") . expect ("invalid abi")
                });
            pub struct VerifierContract<M>(ethers_contract::Contract<M>);
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl<M: ::core::clone::Clone> ::core::clone::Clone for VerifierContract<M> {
                #[inline]
                fn clone(&self) -> VerifierContract<M> {
                    match *self {
                        VerifierContract(ref __self_0_0) => {
                            VerifierContract(::core::clone::Clone::clone(&(*__self_0_0)))
                        }
                    }
                }
            }
            impl<M> std::ops::Deref for VerifierContract<M> {
                type Target = ethers_contract::Contract<M>;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
            impl<M: ethers_providers::Middleware> std::fmt::Debug for VerifierContract<M> {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.debug_tuple("VerifierContract")
                        .field(&self.address())
                        .finish()
                }
            }
            impl<'a, M: ethers_providers::Middleware> VerifierContract<M> {
                /// Creates a new contract instance with the specified `ethers`
                /// client at the given `Address`. The contract derefs to a `ethers::Contract`
                /// object
                pub fn new<T: Into<ethers_core::types::Address>>(
                    address: T,
                    client: ::std::sync::Arc<M>,
                ) -> Self {
                    let contract = ethers_contract::Contract::new(
                        address.into(),
                        VERIFIERCONTRACT_ABI.clone(),
                        client,
                    );
                    Self(contract)
                }
                ///Calls the contract's `verify` (0x9416c1ee) function
                pub fn verify(
                    &self,
                    input: ::std::vec::Vec<ethers_core::types::U256>,
                    proof: Proof,
                    vk: VerifyingKey,
                ) -> ethers_contract::builders::ContractCall<M, bool> {
                    self.0
                        .method_hash([148, 22, 193, 238], (input, proof, vk))
                        .expect("method not found (this should never happen)")
                }
            }
            ///Container type for all input parameters for the `verify`function with signature `verify(uint256[],((uint256,uint256),(uint256[2],uint256[2]),(uint256,uint256)),((uint256,uint256),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256,uint256)[]))` and selector `[148, 22, 193, 238]`
            #[ethcall(
            name = "verify",
            abi = "verify(uint256[],((uint256,uint256),(uint256[2],uint256[2]),(uint256,uint256)),((uint256,uint256),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256,uint256)[]))"
            )]
            pub struct VerifyCall {
                pub input: ::std::vec::Vec<ethers_core::types::U256>,
                pub proof: Proof,
                pub vk: VerifyingKey,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::clone::Clone for VerifyCall {
                #[inline]
                fn clone(&self) -> VerifyCall {
                    match *self {
                        VerifyCall {
                            input: ref __self_0_0,
                            proof: ref __self_0_1,
                            vk: ref __self_0_2,
                        } => VerifyCall {
                            input: ::core::clone::Clone::clone(&(*__self_0_0)),
                            proof: ::core::clone::Clone::clone(&(*__self_0_1)),
                            vk: ::core::clone::Clone::clone(&(*__self_0_2)),
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::fmt::Debug for VerifyCall {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    match *self {
                        VerifyCall {
                            input: ref __self_0_0,
                            proof: ref __self_0_1,
                            vk: ref __self_0_2,
                        } => {
                            let debug_trait_builder =
                                &mut ::core::fmt::Formatter::debug_struct(f, "VerifyCall");
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "input",
                                &&(*__self_0_0),
                            );
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "proof",
                                &&(*__self_0_1),
                            );
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "vk",
                                &&(*__self_0_2),
                            );
                            ::core::fmt::DebugStruct::finish(debug_trait_builder)
                        }
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::default::Default for VerifyCall {
                #[inline]
                fn default() -> VerifyCall {
                    VerifyCall {
                        input: ::core::default::Default::default(),
                        proof: ::core::default::Default::default(),
                        vk: ::core::default::Default::default(),
                    }
                }
            }
            impl ::core::marker::StructuralEq for VerifyCall {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::cmp::Eq for VerifyCall {
                #[inline]
                #[doc(hidden)]
                #[no_coverage]
                fn assert_receiver_is_total_eq(&self) -> () {
                    {
                        let _: ::core::cmp::AssertParamIsEq<
                            ::std::vec::Vec<ethers_core::types::U256>,
                        >;
                        let _: ::core::cmp::AssertParamIsEq<Proof>;
                        let _: ::core::cmp::AssertParamIsEq<VerifyingKey>;
                    }
                }
            }
            impl ::core::marker::StructuralPartialEq for VerifyCall {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::cmp::PartialEq for VerifyCall {
                #[inline]
                fn eq(&self, other: &VerifyCall) -> bool {
                    match *other {
                        VerifyCall {
                            input: ref __self_1_0,
                            proof: ref __self_1_1,
                            vk: ref __self_1_2,
                        } => match *self {
                            VerifyCall {
                                input: ref __self_0_0,
                                proof: ref __self_0_1,
                                vk: ref __self_0_2,
                            } => {
                                (*__self_0_0) == (*__self_1_0)
                                    && (*__self_0_1) == (*__self_1_1)
                                    && (*__self_0_2) == (*__self_1_2)
                            }
                        },
                    }
                }
                #[inline]
                fn ne(&self, other: &VerifyCall) -> bool {
                    match *other {
                        VerifyCall {
                            input: ref __self_1_0,
                            proof: ref __self_1_1,
                            vk: ref __self_1_2,
                        } => match *self {
                            VerifyCall {
                                input: ref __self_0_0,
                                proof: ref __self_0_1,
                                vk: ref __self_0_2,
                            } => {
                                (*__self_0_0) != (*__self_1_0)
                                    || (*__self_0_1) != (*__self_1_1)
                                    || (*__self_0_2) != (*__self_1_2)
                            }
                        },
                    }
                }
            }
            impl ethers_core::abi::Tokenizable for VerifyCall
                where
                    ::std::vec::Vec<ethers_core::types::U256>: ethers_core::abi::Tokenize,
                    Proof: ethers_core::abi::Tokenize,
                    VerifyingKey: ethers_core::abi::Tokenize,
            {
                fn from_token(
                    token: ethers_core::abi::Token,
                ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                    where
                        Self: Sized,
                {
                    if let ethers_core::abi::Token::Tuple(tokens) = token {
                        if tokens.len() != 3usize {
                            return Err(ethers_core::abi::InvalidOutputType({
                                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                    &["Expected ", " tokens, got ", ": "],
                                    &match (&3usize, &tokens.len(), &tokens) {
                                        (arg0, arg1, arg2) => [
                                            ::core::fmt::ArgumentV1::new(
                                                arg0,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg1,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg2,
                                                ::core::fmt::Debug::fmt,
                                            ),
                                        ],
                                    },
                                ));
                                res
                            }));
                        }
                        let mut iter = tokens.into_iter();
                        Ok(Self {
                            input: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                            proof: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                            vk: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                        })
                    } else {
                        Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected Tuple, got "],
                                &match (&token,) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Debug::fmt,
                                    )],
                                },
                            ));
                            res
                        }))
                    }
                }
                fn into_token(self) -> ethers_core::abi::Token {
                    ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                        self.input.into_token(),
                        self.proof.into_token(),
                        self.vk.into_token(),
                    ]))
                }
            }
            impl ethers_core::abi::TokenizableItem for VerifyCall
                where
                    ::std::vec::Vec<ethers_core::types::U256>: ethers_core::abi::Tokenize,
                    Proof: ethers_core::abi::Tokenize,
                    VerifyingKey: ethers_core::abi::Tokenize,
            {
            }
            impl ethers_contract::EthCall for VerifyCall {
                fn function_name() -> ::std::borrow::Cow<'static, str> {
                    "verify".into()
                }
                fn selector() -> ethers_core::types::Selector {
                    [181, 234, 219, 99]
                }
                fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                    "verify(uint256[],((uint256,uint256)),((uint256[2],uint256[2])),((uint256,uint256)),((uint256,uint256)),((uint256[2],uint256[2])),((uint256[2],uint256[2])),((uint256[2],uint256[2])),((uint256,uint256)[]))" . into ()
                }
            }
            impl ethers_core::abi::AbiDecode for VerifyCall {
                fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                    let bytes = bytes.as_ref();
                    if bytes.len() < 4
                        || bytes[..4] != <Self as ethers_contract::EthCall>::selector()
                    {
                        return Err(ethers_contract::AbiError::WrongSelector);
                    }
                    let data_types = [
                        ethers_core::abi::ParamType::Array(Box::new(
                            ethers_core::abi::ParamType::Uint(256usize),
                        )),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::Uint(256usize),
                                ethers_core::abi::ParamType::Uint(256usize),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::Uint(256usize),
                                ethers_core::abi::ParamType::Uint(256usize),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::Uint(256usize),
                                ethers_core::abi::ParamType::Uint(256usize),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Array(Box::new(
                                ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                    ethers_core::abi::ParamType::Uint(256usize),
                                    ethers_core::abi::ParamType::Uint(256usize),
                                ])),
                            )),
                        ])),
                    ];
                    let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                    Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                        ethers_core::abi::Token::Tuple(data_tokens),
                    )?)
                }
            }
            impl ethers_core::abi::AbiEncode for VerifyCall {
                fn encode(self) -> ::std::vec::Vec<u8> {
                    let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                    let selector = <Self as ethers_contract::EthCall>::selector();
                    let encoded = ethers_core::abi::encode(&tokens);
                    selector
                        .iter()
                        .copied()
                        .chain(encoded.into_iter())
                        .collect()
                }
            }
            impl ::std::fmt::Display for VerifyCall {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[""],
                        &match (&&self.input,) {
                            (arg0,) => {
                                [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                            }
                        },
                    ))?;
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[", "],
                        &match () {
                            () => [],
                        },
                    ))?;
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[""],
                        &match (&&self.proof,) {
                            (arg0,) => {
                                [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                            }
                        },
                    ))?;
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[", "],
                        &match () {
                            () => [],
                        },
                    ))?;
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[""],
                        &match (&&self.vk,) {
                            (arg0,) => {
                                [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                            }
                        },
                    ))?;
                    Ok(())
                }
            }
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for VerifyCall {
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                    {
                        #[allow(non_camel_case_types)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __ignore,
                        }
                        struct __FieldVisitor;
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private::Formatter,
                            ) -> _serde::__private::fmt::Result {
                                _serde::__private::Formatter::write_str(
                                    __formatter,
                                    "field identifier",
                                )
                            }
                            fn visit_u64<__E>(
                                self,
                                __value: u64,
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    0u64 => _serde::__private::Ok(__Field::__field0),
                                    1u64 => _serde::__private::Ok(__Field::__field1),
                                    2u64 => _serde::__private::Ok(__Field::__field2),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(
                                self,
                                __value: &str,
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    "input" => _serde::__private::Ok(__Field::__field0),
                                    "proof" => _serde::__private::Ok(__Field::__field1),
                                    "vk" => _serde::__private::Ok(__Field::__field2),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_bytes<__E>(
                                self,
                                __value: &[u8],
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    b"input" => _serde::__private::Ok(__Field::__field0),
                                    b"proof" => _serde::__private::Ok(__Field::__field1),
                                    b"vk" => _serde::__private::Ok(__Field::__field2),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                        }
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(
                                __deserializer: __D,
                            ) -> _serde::__private::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                            {
                                _serde::Deserializer::deserialize_identifier(
                                    __deserializer,
                                    __FieldVisitor,
                                )
                            }
                        }
                        struct __Visitor<'de> {
                            marker: _serde::__private::PhantomData<VerifyCall>,
                            lifetime: _serde::__private::PhantomData<&'de ()>,
                        }
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = VerifyCall;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private::Formatter,
                            ) -> _serde::__private::fmt::Result {
                                _serde::__private::Formatter::write_str(
                                    __formatter,
                                    "struct VerifyCall",
                                )
                            }
                            #[inline]
                            fn visit_seq<__A>(
                                self,
                                mut __seq: __A,
                            ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                            {
                                let __field0 = match match _serde::de::SeqAccess::next_element::<
                                    ::std::vec::Vec<ethers_core::types::U256>,
                                >(
                                    &mut __seq
                                ) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                } {
                                    _serde::__private::Some(__value) => __value,
                                    _serde::__private::None => {
                                        return _serde::__private::Err(
                                            _serde::de::Error::invalid_length(
                                                0usize,
                                                &"struct VerifyCall with 3 elements",
                                            ),
                                        );
                                    }
                                };
                                let __field1 =
                                    match match _serde::de::SeqAccess::next_element::<Proof>(
                                        &mut __seq,
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    1usize,
                                                    &"struct VerifyCall with 3 elements",
                                                ),
                                            );
                                        }
                                    };
                                let __field2 = match match _serde::de::SeqAccess::next_element::<
                                    VerifyingKey,
                                >(
                                    &mut __seq
                                ) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                } {
                                    _serde::__private::Some(__value) => __value,
                                    _serde::__private::None => {
                                        return _serde::__private::Err(
                                            _serde::de::Error::invalid_length(
                                                2usize,
                                                &"struct VerifyCall with 3 elements",
                                            ),
                                        );
                                    }
                                };
                                _serde::__private::Ok(VerifyCall {
                                    input: __field0,
                                    proof: __field1,
                                    vk: __field2,
                                })
                            }
                            #[inline]
                            fn visit_map<__A>(
                                self,
                                mut __map: __A,
                            ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                            {
                                let mut __field0: _serde::__private::Option<
                                    ::std::vec::Vec<ethers_core::types::U256>,
                                > = _serde::__private::None;
                                let mut __field1: _serde::__private::Option<Proof> =
                                    _serde::__private::None;
                                let mut __field2: _serde::__private::Option<VerifyingKey> =
                                    _serde::__private::None;
                                while let _serde::__private::Some(__key) =
                                match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                                {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private::Option::is_some(&__field0) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("input")) ;
                                            }
                                            __field0 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<
                                                    ::std::vec::Vec<ethers_core::types::U256>,
                                                >(
                                                    &mut __map
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private::Option::is_some(&__field1) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("proof")) ;
                                            }
                                            __field1 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<Proof>(
                                                    &mut __map,
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private::Option::is_some(&__field2) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("vk")) ;
                                            }
                                            __field2 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<
                                                    VerifyingKey,
                                                >(
                                                    &mut __map
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        _ => {
                                            let _ = match _serde::de::MapAccess::next_value::<
                                                _serde::de::IgnoredAny,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            };
                                        }
                                    }
                                }
                                let __field0 = match __field0 {
                                    _serde::__private::Some(__field0) => __field0,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("input") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                let __field1 = match __field1 {
                                    _serde::__private::Some(__field1) => __field1,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("proof") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                let __field2 = match __field2 {
                                    _serde::__private::Some(__field2) => __field2,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("vk") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                _serde::__private::Ok(VerifyCall {
                                    input: __field0,
                                    proof: __field1,
                                    vk: __field2,
                                })
                            }
                        }
                        const FIELDS: &'static [&'static str] = &["input", "proof", "vk"];
                        _serde::Deserializer::deserialize_struct(
                            __deserializer,
                            "VerifyCall",
                            FIELDS,
                            __Visitor {
                                marker: _serde::__private::PhantomData::<VerifyCall>,
                                lifetime: _serde::__private::PhantomData,
                            },
                        )
                    }
                }
            };
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl _serde::Serialize for VerifyCall {
                    fn serialize<__S>(
                        &self,
                        __serializer: __S,
                    ) -> _serde::__private::Result<__S::Ok, __S::Error>
                        where
                            __S: _serde::Serializer,
                    {
                        let mut __serde_state = match _serde::Serializer::serialize_struct(
                            __serializer,
                            "VerifyCall",
                            false as usize + 1 + 1 + 1,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "input",
                            &self.input,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "proof",
                            &self.proof,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "vk",
                            &self.vk,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        }
        pub use myotherverifiercontract_mod::*;
        #[allow(clippy::too_many_arguments)]
        mod myotherverifiercontract_mod {
            #![allow(clippy::enum_variant_names)]
            #![allow(dead_code)]
            #![allow(clippy::type_complexity)]
            #![allow(unused_imports)]
            ///MyOtherVerifierContract was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
            use std::sync::Arc;
            use ethers_core::{
                abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
                types::*,
            };
            use ethers_contract::{
                Contract,
                builders::{ContractCall, Event},
                Lazy,
            };
            use ethers_providers::Middleware;
            pub use super::__shared_types::*;
            pub static MYOTHERVERIFIERCONTRACT_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
                ethers_contract::Lazy::new(|| {
                    serde_json :: from_str ("[\n  {\n    \"inputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"constructor\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"uint256[]\",\n        \"name\": \"input\",\n        \"type\": \"uint256[]\"\n      },\n      {\n        \"components\": [\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"X\",\n                \"type\": \"uint256\"\n              },\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"Y\",\n                \"type\": \"uint256\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G1Point\",\n            \"name\": \"A\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"X\",\n                \"type\": \"uint256[2]\"\n              },\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"Y\",\n                \"type\": \"uint256[2]\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G2Point\",\n            \"name\": \"B\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"X\",\n                \"type\": \"uint256\"\n              },\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"Y\",\n                \"type\": \"uint256\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G1Point\",\n            \"name\": \"C\",\n            \"type\": \"tuple\"\n          }\n        ],\n        \"internalType\": \"struct Verifier.Proof\",\n        \"name\": \"proof\",\n        \"type\": \"tuple\"\n      },\n      {\n        \"components\": [\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"X\",\n                \"type\": \"uint256\"\n              },\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"Y\",\n                \"type\": \"uint256\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G1Point\",\n            \"name\": \"alfa1\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"X\",\n                \"type\": \"uint256[2]\"\n              },\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"Y\",\n                \"type\": \"uint256[2]\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G2Point\",\n            \"name\": \"beta2\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"X\",\n                \"type\": \"uint256[2]\"\n              },\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"Y\",\n                \"type\": \"uint256[2]\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G2Point\",\n            \"name\": \"gamma2\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"X\",\n                \"type\": \"uint256[2]\"\n              },\n              {\n                \"internalType\": \"uint256[2]\",\n                \"name\": \"Y\",\n                \"type\": \"uint256[2]\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G2Point\",\n            \"name\": \"delta2\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"X\",\n                \"type\": \"uint256\"\n              },\n              {\n                \"internalType\": \"uint256\",\n                \"name\": \"Y\",\n                \"type\": \"uint256\"\n              }\n            ],\n            \"internalType\": \"struct Pairing.G1Point[]\",\n            \"name\": \"IC\",\n            \"type\": \"tuple[]\"\n          }\n        ],\n        \"internalType\": \"struct Verifier.VerifyingKey\",\n        \"name\": \"vk\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"name\": \"verify\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  }\n]\n") . expect ("invalid abi")
                });
            pub struct MyOtherVerifierContract<M>(ethers_contract::Contract<M>);
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl<M: ::core::clone::Clone> ::core::clone::Clone for MyOtherVerifierContract<M> {
                #[inline]
                fn clone(&self) -> MyOtherVerifierContract<M> {
                    match *self {
                        MyOtherVerifierContract(ref __self_0_0) => {
                            MyOtherVerifierContract(::core::clone::Clone::clone(&(*__self_0_0)))
                        }
                    }
                }
            }
            impl<M> std::ops::Deref for MyOtherVerifierContract<M> {
                type Target = ethers_contract::Contract<M>;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
            impl<M: ethers_providers::Middleware> std::fmt::Debug for MyOtherVerifierContract<M> {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.debug_tuple("MyOtherVerifierContract")
                        .field(&self.address())
                        .finish()
                }
            }
            impl<'a, M: ethers_providers::Middleware> MyOtherVerifierContract<M> {
                /// Creates a new contract instance with the specified `ethers`
                /// client at the given `Address`. The contract derefs to a `ethers::Contract`
                /// object
                pub fn new<T: Into<ethers_core::types::Address>>(
                    address: T,
                    client: ::std::sync::Arc<M>,
                ) -> Self {
                    let contract = ethers_contract::Contract::new(
                        address.into(),
                        MYOTHERVERIFIERCONTRACT_ABI.clone(),
                        client,
                    );
                    Self(contract)
                }
                ///Calls the contract's `verify` (0x9416c1ee) function
                pub fn verify(
                    &self,
                    input: ::std::vec::Vec<ethers_core::types::U256>,
                    proof: Proof,
                    vk: VerifyingKey,
                ) -> ethers_contract::builders::ContractCall<M, bool> {
                    self.0
                        .method_hash([148, 22, 193, 238], (input, proof, vk))
                        .expect("method not found (this should never happen)")
                }
            }
            ///Container type for all input parameters for the `verify`function with signature `verify(uint256[],((uint256,uint256),(uint256[2],uint256[2]),(uint256,uint256)),((uint256,uint256),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256,uint256)[]))` and selector `[148, 22, 193, 238]`
            #[ethcall(
            name = "verify",
            abi = "verify(uint256[],((uint256,uint256),(uint256[2],uint256[2]),(uint256,uint256)),((uint256,uint256),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256[2],uint256[2]),(uint256,uint256)[]))"
            )]
            pub struct VerifyCall {
                pub input: ::std::vec::Vec<ethers_core::types::U256>,
                pub proof: Proof,
                pub vk: VerifyingKey,
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::clone::Clone for VerifyCall {
                #[inline]
                fn clone(&self) -> VerifyCall {
                    match *self {
                        VerifyCall {
                            input: ref __self_0_0,
                            proof: ref __self_0_1,
                            vk: ref __self_0_2,
                        } => VerifyCall {
                            input: ::core::clone::Clone::clone(&(*__self_0_0)),
                            proof: ::core::clone::Clone::clone(&(*__self_0_1)),
                            vk: ::core::clone::Clone::clone(&(*__self_0_2)),
                        },
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::fmt::Debug for VerifyCall {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    match *self {
                        VerifyCall {
                            input: ref __self_0_0,
                            proof: ref __self_0_1,
                            vk: ref __self_0_2,
                        } => {
                            let debug_trait_builder =
                                &mut ::core::fmt::Formatter::debug_struct(f, "VerifyCall");
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "input",
                                &&(*__self_0_0),
                            );
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "proof",
                                &&(*__self_0_1),
                            );
                            let _ = ::core::fmt::DebugStruct::field(
                                debug_trait_builder,
                                "vk",
                                &&(*__self_0_2),
                            );
                            ::core::fmt::DebugStruct::finish(debug_trait_builder)
                        }
                    }
                }
            }
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::default::Default for VerifyCall {
                #[inline]
                fn default() -> VerifyCall {
                    VerifyCall {
                        input: ::core::default::Default::default(),
                        proof: ::core::default::Default::default(),
                        vk: ::core::default::Default::default(),
                    }
                }
            }
            impl ::core::marker::StructuralEq for VerifyCall {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::cmp::Eq for VerifyCall {
                #[inline]
                #[doc(hidden)]
                #[no_coverage]
                fn assert_receiver_is_total_eq(&self) -> () {
                    {
                        let _: ::core::cmp::AssertParamIsEq<
                            ::std::vec::Vec<ethers_core::types::U256>,
                        >;
                        let _: ::core::cmp::AssertParamIsEq<Proof>;
                        let _: ::core::cmp::AssertParamIsEq<VerifyingKey>;
                    }
                }
            }
            impl ::core::marker::StructuralPartialEq for VerifyCall {}
            #[automatically_derived]
            #[allow(unused_qualifications)]
            impl ::core::cmp::PartialEq for VerifyCall {
                #[inline]
                fn eq(&self, other: &VerifyCall) -> bool {
                    match *other {
                        VerifyCall {
                            input: ref __self_1_0,
                            proof: ref __self_1_1,
                            vk: ref __self_1_2,
                        } => match *self {
                            VerifyCall {
                                input: ref __self_0_0,
                                proof: ref __self_0_1,
                                vk: ref __self_0_2,
                            } => {
                                (*__self_0_0) == (*__self_1_0)
                                    && (*__self_0_1) == (*__self_1_1)
                                    && (*__self_0_2) == (*__self_1_2)
                            }
                        },
                    }
                }
                #[inline]
                fn ne(&self, other: &VerifyCall) -> bool {
                    match *other {
                        VerifyCall {
                            input: ref __self_1_0,
                            proof: ref __self_1_1,
                            vk: ref __self_1_2,
                        } => match *self {
                            VerifyCall {
                                input: ref __self_0_0,
                                proof: ref __self_0_1,
                                vk: ref __self_0_2,
                            } => {
                                (*__self_0_0) != (*__self_1_0)
                                    || (*__self_0_1) != (*__self_1_1)
                                    || (*__self_0_2) != (*__self_1_2)
                            }
                        },
                    }
                }
            }
            impl ethers_core::abi::Tokenizable for VerifyCall
                where
                    ::std::vec::Vec<ethers_core::types::U256>: ethers_core::abi::Tokenize,
                    Proof: ethers_core::abi::Tokenize,
                    VerifyingKey: ethers_core::abi::Tokenize,
            {
                fn from_token(
                    token: ethers_core::abi::Token,
                ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                    where
                        Self: Sized,
                {
                    if let ethers_core::abi::Token::Tuple(tokens) = token {
                        if tokens.len() != 3usize {
                            return Err(ethers_core::abi::InvalidOutputType({
                                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                    &["Expected ", " tokens, got ", ": "],
                                    &match (&3usize, &tokens.len(), &tokens) {
                                        (arg0, arg1, arg2) => [
                                            ::core::fmt::ArgumentV1::new(
                                                arg0,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg1,
                                                ::core::fmt::Display::fmt,
                                            ),
                                            ::core::fmt::ArgumentV1::new(
                                                arg2,
                                                ::core::fmt::Debug::fmt,
                                            ),
                                        ],
                                    },
                                ));
                                res
                            }));
                        }
                        let mut iter = tokens.into_iter();
                        Ok(Self {
                            input: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                            proof: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                            vk: ethers_core::abi::Tokenizable::from_token(
                                iter.next()
                                    .expect("tokens size is sufficient qed")
                                    .into_token(),
                            )?,
                        })
                    } else {
                        Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected Tuple, got "],
                                &match (&token,) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Debug::fmt,
                                    )],
                                },
                            ));
                            res
                        }))
                    }
                }
                fn into_token(self) -> ethers_core::abi::Token {
                    ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                        self.input.into_token(),
                        self.proof.into_token(),
                        self.vk.into_token(),
                    ]))
                }
            }
            impl ethers_core::abi::TokenizableItem for VerifyCall
                where
                    ::std::vec::Vec<ethers_core::types::U256>: ethers_core::abi::Tokenize,
                    Proof: ethers_core::abi::Tokenize,
                    VerifyingKey: ethers_core::abi::Tokenize,
            {
            }
            impl ethers_contract::EthCall for VerifyCall {
                fn function_name() -> ::std::borrow::Cow<'static, str> {
                    "verify".into()
                }
                fn selector() -> ethers_core::types::Selector {
                    [181, 234, 219, 99]
                }
                fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                    "verify(uint256[],((uint256,uint256)),((uint256[2],uint256[2])),((uint256,uint256)),((uint256,uint256)),((uint256[2],uint256[2])),((uint256[2],uint256[2])),((uint256[2],uint256[2])),((uint256,uint256)[]))" . into ()
                }
            }
            impl ethers_core::abi::AbiDecode for VerifyCall {
                fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                    let bytes = bytes.as_ref();
                    if bytes.len() < 4
                        || bytes[..4] != <Self as ethers_contract::EthCall>::selector()
                    {
                        return Err(ethers_contract::AbiError::WrongSelector);
                    }
                    let data_types = [
                        ethers_core::abi::ParamType::Array(Box::new(
                            ethers_core::abi::ParamType::Uint(256usize),
                        )),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::Uint(256usize),
                                ethers_core::abi::ParamType::Uint(256usize),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::Uint(256usize),
                                ethers_core::abi::ParamType::Uint(256usize),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::Uint(256usize),
                                ethers_core::abi::ParamType::Uint(256usize),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                                ethers_core::abi::ParamType::FixedArray(
                                    Box::new(ethers_core::abi::ParamType::Uint(256usize)),
                                    2usize,
                                ),
                            ])),
                        ])),
                        ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                            ethers_core::abi::ParamType::Array(Box::new(
                                ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                                    ethers_core::abi::ParamType::Uint(256usize),
                                    ethers_core::abi::ParamType::Uint(256usize),
                                ])),
                            )),
                        ])),
                    ];
                    let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                    Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                        ethers_core::abi::Token::Tuple(data_tokens),
                    )?)
                }
            }
            impl ethers_core::abi::AbiEncode for VerifyCall {
                fn encode(self) -> ::std::vec::Vec<u8> {
                    let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                    let selector = <Self as ethers_contract::EthCall>::selector();
                    let encoded = ethers_core::abi::encode(&tokens);
                    selector
                        .iter()
                        .copied()
                        .chain(encoded.into_iter())
                        .collect()
                }
            }
            impl ::std::fmt::Display for VerifyCall {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[""],
                        &match (&&self.input,) {
                            (arg0,) => {
                                [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                            }
                        },
                    ))?;
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[", "],
                        &match () {
                            () => [],
                        },
                    ))?;
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[""],
                        &match (&&self.proof,) {
                            (arg0,) => {
                                [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                            }
                        },
                    ))?;
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[", "],
                        &match () {
                            () => [],
                        },
                    ))?;
                    f.write_fmt(::core::fmt::Arguments::new_v1(
                        &[""],
                        &match (&&self.vk,) {
                            (arg0,) => {
                                [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                            }
                        },
                    ))?;
                    Ok(())
                }
            }
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for VerifyCall {
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                    {
                        #[allow(non_camel_case_types)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __ignore,
                        }
                        struct __FieldVisitor;
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private::Formatter,
                            ) -> _serde::__private::fmt::Result {
                                _serde::__private::Formatter::write_str(
                                    __formatter,
                                    "field identifier",
                                )
                            }
                            fn visit_u64<__E>(
                                self,
                                __value: u64,
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    0u64 => _serde::__private::Ok(__Field::__field0),
                                    1u64 => _serde::__private::Ok(__Field::__field1),
                                    2u64 => _serde::__private::Ok(__Field::__field2),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(
                                self,
                                __value: &str,
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    "input" => _serde::__private::Ok(__Field::__field0),
                                    "proof" => _serde::__private::Ok(__Field::__field1),
                                    "vk" => _serde::__private::Ok(__Field::__field2),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_bytes<__E>(
                                self,
                                __value: &[u8],
                            ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                            {
                                match __value {
                                    b"input" => _serde::__private::Ok(__Field::__field0),
                                    b"proof" => _serde::__private::Ok(__Field::__field1),
                                    b"vk" => _serde::__private::Ok(__Field::__field2),
                                    _ => _serde::__private::Ok(__Field::__ignore),
                                }
                            }
                        }
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(
                                __deserializer: __D,
                            ) -> _serde::__private::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                            {
                                _serde::Deserializer::deserialize_identifier(
                                    __deserializer,
                                    __FieldVisitor,
                                )
                            }
                        }
                        struct __Visitor<'de> {
                            marker: _serde::__private::PhantomData<VerifyCall>,
                            lifetime: _serde::__private::PhantomData<&'de ()>,
                        }
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = VerifyCall;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private::Formatter,
                            ) -> _serde::__private::fmt::Result {
                                _serde::__private::Formatter::write_str(
                                    __formatter,
                                    "struct VerifyCall",
                                )
                            }
                            #[inline]
                            fn visit_seq<__A>(
                                self,
                                mut __seq: __A,
                            ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                            {
                                let __field0 = match match _serde::de::SeqAccess::next_element::<
                                    ::std::vec::Vec<ethers_core::types::U256>,
                                >(
                                    &mut __seq
                                ) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                } {
                                    _serde::__private::Some(__value) => __value,
                                    _serde::__private::None => {
                                        return _serde::__private::Err(
                                            _serde::de::Error::invalid_length(
                                                0usize,
                                                &"struct VerifyCall with 3 elements",
                                            ),
                                        );
                                    }
                                };
                                let __field1 =
                                    match match _serde::de::SeqAccess::next_element::<Proof>(
                                        &mut __seq,
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    1usize,
                                                    &"struct VerifyCall with 3 elements",
                                                ),
                                            );
                                        }
                                    };
                                let __field2 = match match _serde::de::SeqAccess::next_element::<
                                    VerifyingKey,
                                >(
                                    &mut __seq
                                ) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                } {
                                    _serde::__private::Some(__value) => __value,
                                    _serde::__private::None => {
                                        return _serde::__private::Err(
                                            _serde::de::Error::invalid_length(
                                                2usize,
                                                &"struct VerifyCall with 3 elements",
                                            ),
                                        );
                                    }
                                };
                                _serde::__private::Ok(VerifyCall {
                                    input: __field0,
                                    proof: __field1,
                                    vk: __field2,
                                })
                            }
                            #[inline]
                            fn visit_map<__A>(
                                self,
                                mut __map: __A,
                            ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                            {
                                let mut __field0: _serde::__private::Option<
                                    ::std::vec::Vec<ethers_core::types::U256>,
                                > = _serde::__private::None;
                                let mut __field1: _serde::__private::Option<Proof> =
                                    _serde::__private::None;
                                let mut __field2: _serde::__private::Option<VerifyingKey> =
                                    _serde::__private::None;
                                while let _serde::__private::Some(__key) =
                                match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                }
                                {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private::Option::is_some(&__field0) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("input")) ;
                                            }
                                            __field0 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<
                                                    ::std::vec::Vec<ethers_core::types::U256>,
                                                >(
                                                    &mut __map
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private::Option::is_some(&__field1) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("proof")) ;
                                            }
                                            __field1 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<Proof>(
                                                    &mut __map,
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private::Option::is_some(&__field2) {
                                                return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("vk")) ;
                                            }
                                            __field2 = _serde::__private::Some(
                                                match _serde::de::MapAccess::next_value::<
                                                    VerifyingKey,
                                                >(
                                                    &mut __map
                                                ) {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                },
                                            );
                                        }
                                        _ => {
                                            let _ = match _serde::de::MapAccess::next_value::<
                                                _serde::de::IgnoredAny,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            };
                                        }
                                    }
                                }
                                let __field0 = match __field0 {
                                    _serde::__private::Some(__field0) => __field0,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("input") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                let __field1 = match __field1 {
                                    _serde::__private::Some(__field1) => __field1,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("proof") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                let __field2 = match __field2 {
                                    _serde::__private::Some(__field2) => __field2,
                                    _serde::__private::None => {
                                        match _serde::__private::de::missing_field("vk") {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    }
                                };
                                _serde::__private::Ok(VerifyCall {
                                    input: __field0,
                                    proof: __field1,
                                    vk: __field2,
                                })
                            }
                        }
                        const FIELDS: &'static [&'static str] = &["input", "proof", "vk"];
                        _serde::Deserializer::deserialize_struct(
                            __deserializer,
                            "VerifyCall",
                            FIELDS,
                            __Visitor {
                                marker: _serde::__private::PhantomData::<VerifyCall>,
                                lifetime: _serde::__private::PhantomData,
                            },
                        )
                    }
                }
            };
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl _serde::Serialize for VerifyCall {
                    fn serialize<__S>(
                        &self,
                        __serializer: __S,
                    ) -> _serde::__private::Result<__S::Ok, __S::Error>
                        where
                            __S: _serde::Serializer,
                    {
                        let mut __serde_state = match _serde::Serializer::serialize_struct(
                            __serializer,
                            "VerifyCall",
                            false as usize + 1 + 1 + 1,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "input",
                            &self.input,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "proof",
                            &self.proof,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "vk",
                            &self.vk,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
        }
    }
    assert_tokenizeable::<VerifyingKey>();
    assert_tokenizeable::<G1Point>();
    assert_tokenizeable::<G2Point>();
    let (provider, _) = Provider::mocked();
    let client = Arc::new(provider);
    let g1 = G1Point {
        x: U256::zero(),
        y: U256::zero(),
    };
    let g2 = G2Point {
        x: [U256::zero(), U256::zero()],
        y: [U256::zero(), U256::zero()],
    };
    let vk = VerifyingKey {
        alfa_1: g1.clone(),
        beta_2: g2.clone(),
        gamma_2: g2.clone(),
        delta_2: g2.clone(),
        ic: <[_]>::into_vec(box [g1.clone()]),
    };
    let proof = Proof {
        a: g1.clone(),
        b: g2,
        c: g1,
    };
    let contract = VerifierContract::new(Address::zero(), client.clone());
    let _ = contract.verify(::alloc::vec::Vec::new(), proof.clone(), vk.clone());
    let contract = MyOtherVerifierContract::new(Address::zero(), client);
    let _ = contract.verify(::alloc::vec::Vec::new(), proof, vk);
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const can_gen_human_readable_with_structs: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("can_gen_human_readable_with_structs"),
        ignore: false,
        allow_fail: false,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(can_gen_human_readable_with_structs())),
};
fn can_gen_human_readable_with_structs() {
    pub use simplecontract_mod::*;
    #[allow(clippy::too_many_arguments)]
    mod simplecontract_mod {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
        #![allow(clippy::type_complexity)]
        #![allow(unused_imports)]
        ///SimpleContract was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
        use std::sync::Arc;
        use ethers_core::{
            abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
            types::*,
        };
        use ethers_contract::{
            Contract,
            builders::{ContractCall, Event},
            Lazy,
        };
        use ethers_providers::Middleware;
        pub static SIMPLECONTRACT_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
            ethers_contract::Lazy::new(|| {
                ethers_core :: abi :: parse_abi_str ("[\n        struct Foo { uint256 x; }\n        function foo(Foo memory x)\n        function bar(uint256 x, uint256 y, address addr)\n        yeet(uint256,uint256,address)\n    ]") . expect ("invalid abi")
            });
        pub struct SimpleContract<M>(ethers_contract::Contract<M>);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<M: ::core::clone::Clone> ::core::clone::Clone for SimpleContract<M> {
            #[inline]
            fn clone(&self) -> SimpleContract<M> {
                match *self {
                    SimpleContract(ref __self_0_0) => {
                        SimpleContract(::core::clone::Clone::clone(&(*__self_0_0)))
                    }
                }
            }
        }
        impl<M> std::ops::Deref for SimpleContract<M> {
            type Target = ethers_contract::Contract<M>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<M: ethers_providers::Middleware> std::fmt::Debug for SimpleContract<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple("SimpleContract")
                    .field(&self.address())
                    .finish()
            }
        }
        impl<'a, M: ethers_providers::Middleware> SimpleContract<M> {
            /// Creates a new contract instance with the specified `ethers`
            /// client at the given `Address`. The contract derefs to a `ethers::Contract`
            /// object
            pub fn new<T: Into<ethers_core::types::Address>>(
                address: T,
                client: ::std::sync::Arc<M>,
            ) -> Self {
                let contract = ethers_contract::Contract::new(
                    address.into(),
                    SIMPLECONTRACT_ABI.clone(),
                    client,
                );
                Self(contract)
            }
            ///Calls the contract's `bar` (0xd7a53568) function
            pub fn bar(
                &self,
                x: ethers_core::types::U256,
                y: ethers_core::types::U256,
                addr: ethers_core::types::Address,
            ) -> ethers_contract::builders::ContractCall<M, ()> {
                self.0
                    .method_hash([215, 165, 53, 104], (x, y, addr))
                    .expect("method not found (this should never happen)")
            }
            ///Calls the contract's `foo` (0xec8a819f) function
            pub fn foo(&self, x: Foo) -> ethers_contract::builders::ContractCall<M, ()> {
                self.0
                    .method_hash([236, 138, 129, 159], (x,))
                    .expect("method not found (this should never happen)")
            }
            ///Calls the contract's `yeet` (0x6e95d74b) function
            pub fn yeet(
                &self,
                p0: ethers_core::types::U256,
                p1: ethers_core::types::U256,
                p2: ethers_core::types::Address,
            ) -> ethers_contract::builders::ContractCall<M, ()> {
                self.0
                    .method_hash([110, 149, 215, 75], (p0, p1, p2))
                    .expect("method not found (this should never happen)")
            }
        }
        ///Container type for all input parameters for the `bar`function with signature `bar(uint256,uint256,address)` and selector `[215, 165, 53, 104]`
        #[ethcall(name = "bar", abi = "bar(uint256,uint256,address)")]
        pub struct BarCall {
            pub x: ethers_core::types::U256,
            pub y: ethers_core::types::U256,
            pub addr: ethers_core::types::Address,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for BarCall {
            #[inline]
            fn clone(&self) -> BarCall {
                match *self {
                    BarCall {
                        x: ref __self_0_0,
                        y: ref __self_0_1,
                        addr: ref __self_0_2,
                    } => BarCall {
                        x: ::core::clone::Clone::clone(&(*__self_0_0)),
                        y: ::core::clone::Clone::clone(&(*__self_0_1)),
                        addr: ::core::clone::Clone::clone(&(*__self_0_2)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for BarCall {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    BarCall {
                        x: ref __self_0_0,
                        y: ref __self_0_1,
                        addr: ref __self_0_2,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "BarCall");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "x",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "y",
                            &&(*__self_0_1),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "addr",
                            &&(*__self_0_2),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for BarCall {
            #[inline]
            fn default() -> BarCall {
                BarCall {
                    x: ::core::default::Default::default(),
                    y: ::core::default::Default::default(),
                    addr: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for BarCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for BarCall {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::U256>;
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::U256>;
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::Address>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for BarCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for BarCall {
            #[inline]
            fn eq(&self, other: &BarCall) -> bool {
                match *other {
                    BarCall {
                        x: ref __self_1_0,
                        y: ref __self_1_1,
                        addr: ref __self_1_2,
                    } => match *self {
                        BarCall {
                            x: ref __self_0_0,
                            y: ref __self_0_1,
                            addr: ref __self_0_2,
                        } => {
                            (*__self_0_0) == (*__self_1_0)
                                && (*__self_0_1) == (*__self_1_1)
                                && (*__self_0_2) == (*__self_1_2)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &BarCall) -> bool {
                match *other {
                    BarCall {
                        x: ref __self_1_0,
                        y: ref __self_1_1,
                        addr: ref __self_1_2,
                    } => match *self {
                        BarCall {
                            x: ref __self_0_0,
                            y: ref __self_0_1,
                            addr: ref __self_0_2,
                        } => {
                            (*__self_0_0) != (*__self_1_0)
                                || (*__self_0_1) != (*__self_1_1)
                                || (*__self_0_2) != (*__self_1_2)
                        }
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for BarCall
            where
                ethers_core::types::U256: ethers_core::abi::Tokenize,
                ethers_core::types::U256: ethers_core::abi::Tokenize,
                ethers_core::types::Address: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 3usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&3usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        x: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        y: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        addr: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.x.into_token(),
                    self.y.into_token(),
                    self.addr.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for BarCall
            where
                ethers_core::types::U256: ethers_core::abi::Tokenize,
                ethers_core::types::U256: ethers_core::abi::Tokenize,
                ethers_core::types::Address: ethers_core::abi::Tokenize,
        {
        }
        impl ethers_contract::EthCall for BarCall {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                "bar".into()
            }
            fn selector() -> ethers_core::types::Selector {
                [215, 165, 53, 104]
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "bar(uint256,uint256,address)".into()
            }
        }
        impl ethers_core::abi::AbiDecode for BarCall {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                let bytes = bytes.as_ref();
                if bytes.len() < 4 || bytes[..4] != <Self as ethers_contract::EthCall>::selector() {
                    return Err(ethers_contract::AbiError::WrongSelector);
                }
                let data_types = [
                    ethers_core::abi::ParamType::Uint(256usize),
                    ethers_core::abi::ParamType::Uint(256usize),
                    ethers_core::abi::ParamType::Address,
                ];
                let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                    ethers_core::abi::Token::Tuple(data_tokens),
                )?)
            }
        }
        impl ethers_core::abi::AbiEncode for BarCall {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                let selector = <Self as ethers_contract::EthCall>::selector();
                let encoded = ethers_core::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }
        impl ::std::fmt::Display for BarCall {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.x,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.y,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.addr,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                Ok(())
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for BarCall {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                1u64 => _serde::__private::Ok(__Field::__field1),
                                2u64 => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "x" => _serde::__private::Ok(__Field::__field0),
                                "y" => _serde::__private::Ok(__Field::__field1),
                                "addr" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"x" => _serde::__private::Ok(__Field::__field0),
                                b"y" => _serde::__private::Ok(__Field::__field1),
                                b"addr" => _serde::__private::Ok(__Field::__field2),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<BarCall>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = BarCall;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "struct BarCall")
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::U256,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct BarCall with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::U256,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct BarCall with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::Address,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct BarCall with 3 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(BarCall {
                                x: __field0,
                                y: __field1,
                                addr: __field2,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<ethers_core::types::U256> =
                                _serde::__private::None;
                            let mut __field1: _serde::__private::Option<ethers_core::types::U256> =
                                _serde::__private::None;
                            let mut __field2: _serde::__private::Option<
                                ethers_core::types::Address,
                            > = _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "x",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::U256,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private::Option::is_some(&__field1) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "y",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::U256,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private::Option::is_some(&__field2) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "addr",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::Address,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("x") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private::Some(__field1) => __field1,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("y") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private::Some(__field2) => __field2,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("addr") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(BarCall {
                                x: __field0,
                                y: __field1,
                                addr: __field2,
                            })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["x", "y", "addr"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "BarCall",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<BarCall>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for BarCall {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "BarCall",
                        false as usize + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "x",
                        &self.x,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "y",
                        &self.y,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "addr",
                        &self.addr,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        ///Container type for all input parameters for the `foo`function with signature `foo((uint256))` and selector `[236, 138, 129, 159]`
        #[ethcall(name = "foo", abi = "foo((uint256))")]
        pub struct FooCall {
            pub x: Foo,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for FooCall {
            #[inline]
            fn clone(&self) -> FooCall {
                match *self {
                    FooCall { x: ref __self_0_0 } => FooCall {
                        x: ::core::clone::Clone::clone(&(*__self_0_0)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for FooCall {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    FooCall { x: ref __self_0_0 } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "FooCall");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "x",
                            &&(*__self_0_0),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for FooCall {
            #[inline]
            fn default() -> FooCall {
                FooCall {
                    x: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for FooCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for FooCall {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<Foo>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for FooCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for FooCall {
            #[inline]
            fn eq(&self, other: &FooCall) -> bool {
                match *other {
                    FooCall { x: ref __self_1_0 } => match *self {
                        FooCall { x: ref __self_0_0 } => (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &FooCall) -> bool {
                match *other {
                    FooCall { x: ref __self_1_0 } => match *self {
                        FooCall { x: ref __self_0_0 } => (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for FooCall
            where
                Foo: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 1usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&1usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        x: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [self.x.into_token()]))
            }
        }
        impl ethers_core::abi::TokenizableItem for FooCall where Foo: ethers_core::abi::Tokenize {}
        impl ethers_contract::EthCall for FooCall {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                "foo".into()
            }
            fn selector() -> ethers_core::types::Selector {
                [236, 138, 129, 159]
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "foo((uint256))".into()
            }
        }
        impl ethers_core::abi::AbiDecode for FooCall {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                let bytes = bytes.as_ref();
                if bytes.len() < 4 || bytes[..4] != <Self as ethers_contract::EthCall>::selector() {
                    return Err(ethers_contract::AbiError::WrongSelector);
                }
                let data_types = [ethers_core::abi::ParamType::Tuple(<[_]>::into_vec(box [
                    ethers_core::abi::ParamType::Uint(256usize),
                ]))];
                let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                    ethers_core::abi::Token::Tuple(data_tokens),
                )?)
            }
        }
        impl ethers_core::abi::AbiEncode for FooCall {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                let selector = <Self as ethers_contract::EthCall>::selector();
                let encoded = ethers_core::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }
        impl ::std::fmt::Display for FooCall {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.x,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                Ok(())
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for FooCall {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "x" => _serde::__private::Ok(__Field::__field0),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"x" => _serde::__private::Ok(__Field::__field0),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<FooCall>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = FooCall;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "struct FooCall")
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<Foo>(
                                &mut __seq,
                            ) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct FooCall with 1 element",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(FooCall { x: __field0 })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<Foo> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "x",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<Foo>(
                                                &mut __map,
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("x") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(FooCall { x: __field0 })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["x"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "FooCall",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<FooCall>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for FooCall {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "FooCall",
                        false as usize + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "x",
                        &self.x,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        ///Container type for all input parameters for the `yeet`function with signature `yeet(uint256,uint256,address)` and selector `[110, 149, 215, 75]`
        #[ethcall(name = "yeet", abi = "yeet(uint256,uint256,address)")]
        pub struct YeetCall(
            pub ethers_core::types::U256,
            pub ethers_core::types::U256,
            pub ethers_core::types::Address,
        );
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for YeetCall {
            #[inline]
            fn clone(&self) -> YeetCall {
                match *self {
                    YeetCall(ref __self_0_0, ref __self_0_1, ref __self_0_2) => YeetCall(
                        ::core::clone::Clone::clone(&(*__self_0_0)),
                        ::core::clone::Clone::clone(&(*__self_0_1)),
                        ::core::clone::Clone::clone(&(*__self_0_2)),
                    ),
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for YeetCall {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    YeetCall(ref __self_0_0, ref __self_0_1, ref __self_0_2) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "YeetCall");
                        let _ =
                            ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                        let _ =
                            ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_1));
                        let _ =
                            ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_2));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for YeetCall {
            #[inline]
            fn default() -> YeetCall {
                YeetCall(
                    ::core::default::Default::default(),
                    ::core::default::Default::default(),
                    ::core::default::Default::default(),
                )
            }
        }
        impl ::core::marker::StructuralEq for YeetCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for YeetCall {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::U256>;
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::U256>;
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::Address>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for YeetCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for YeetCall {
            #[inline]
            fn eq(&self, other: &YeetCall) -> bool {
                match *other {
                    YeetCall(ref __self_1_0, ref __self_1_1, ref __self_1_2) => match *self {
                        YeetCall(ref __self_0_0, ref __self_0_1, ref __self_0_2) => {
                            (*__self_0_0) == (*__self_1_0)
                                && (*__self_0_1) == (*__self_1_1)
                                && (*__self_0_2) == (*__self_1_2)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &YeetCall) -> bool {
                match *other {
                    YeetCall(ref __self_1_0, ref __self_1_1, ref __self_1_2) => match *self {
                        YeetCall(ref __self_0_0, ref __self_0_1, ref __self_0_2) => {
                            (*__self_0_0) != (*__self_1_0)
                                || (*__self_0_1) != (*__self_1_1)
                                || (*__self_0_2) != (*__self_1_2)
                        }
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for YeetCall
            where
                ethers_core::types::U256: ethers_core::abi::Tokenize,
                ethers_core::types::U256: ethers_core::abi::Tokenize,
                ethers_core::types::Address: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 3usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&3usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self(
                        ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    ))
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.0.into_token(),
                    self.1.into_token(),
                    self.2.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for YeetCall
            where
                ethers_core::types::U256: ethers_core::abi::Tokenize,
                ethers_core::types::U256: ethers_core::abi::Tokenize,
                ethers_core::types::Address: ethers_core::abi::Tokenize,
        {
        }
        impl ethers_contract::EthCall for YeetCall {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                "yeet".into()
            }
            fn selector() -> ethers_core::types::Selector {
                [110, 149, 215, 75]
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "yeet(uint256,uint256,address)".into()
            }
        }
        impl ethers_core::abi::AbiDecode for YeetCall {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                let bytes = bytes.as_ref();
                if bytes.len() < 4 || bytes[..4] != <Self as ethers_contract::EthCall>::selector() {
                    return Err(ethers_contract::AbiError::WrongSelector);
                }
                let data_types = [
                    ethers_core::abi::ParamType::Uint(256usize),
                    ethers_core::abi::ParamType::Uint(256usize),
                    ethers_core::abi::ParamType::Address,
                ];
                let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                    ethers_core::abi::Token::Tuple(data_tokens),
                )?)
            }
        }
        impl ethers_core::abi::AbiEncode for YeetCall {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                let selector = <Self as ethers_contract::EthCall>::selector();
                let encoded = ethers_core::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }
        impl ::std::fmt::Display for YeetCall {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.0,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.1,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.2,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                Ok(())
            }
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for YeetCall {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<YeetCall>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = YeetCall;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "tuple struct YeetCall",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::U256,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"tuple struct YeetCall with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::U256,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"tuple struct YeetCall with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::Address,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"tuple struct YeetCall with 3 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(YeetCall(__field0, __field1, __field2))
                        }
                    }
                    _serde::Deserializer::deserialize_tuple_struct(
                        __deserializer,
                        "YeetCall",
                        3usize,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<YeetCall>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for YeetCall {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_tuple_struct(
                        __serializer,
                        "YeetCall",
                        0 + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleStruct::serialize_field(
                        &mut __serde_state,
                        &self.0,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleStruct::serialize_field(
                        &mut __serde_state,
                        &self.1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleStruct::serialize_field(
                        &mut __serde_state,
                        &self.2,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeTupleStruct::end(__serde_state)
                }
            }
        };
        pub enum SimpleContractCalls {
            Bar(BarCall),
            Foo(FooCall),
            Yeet(YeetCall),
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for SimpleContractCalls {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&SimpleContractCalls::Bar(ref __self_0),) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "Bar");
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                    (&SimpleContractCalls::Foo(ref __self_0),) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "Foo");
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                    (&SimpleContractCalls::Yeet(ref __self_0),) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "Yeet");
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for SimpleContractCalls {
            #[inline]
            fn clone(&self) -> SimpleContractCalls {
                match (&*self,) {
                    (&SimpleContractCalls::Bar(ref __self_0),) => {
                        SimpleContractCalls::Bar(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&SimpleContractCalls::Foo(ref __self_0),) => {
                        SimpleContractCalls::Foo(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&SimpleContractCalls::Yeet(ref __self_0),) => {
                        SimpleContractCalls::Yeet(::core::clone::Clone::clone(&(*__self_0)))
                    }
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for SimpleContractCalls {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for SimpleContractCalls {
            #[inline]
            fn eq(&self, other: &SimpleContractCalls) -> bool {
                {
                    let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                    let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            (
                                &SimpleContractCalls::Bar(ref __self_0),
                                &SimpleContractCalls::Bar(ref __arg_1_0),
                            ) => (*__self_0) == (*__arg_1_0),
                            (
                                &SimpleContractCalls::Foo(ref __self_0),
                                &SimpleContractCalls::Foo(ref __arg_1_0),
                            ) => (*__self_0) == (*__arg_1_0),
                            (
                                &SimpleContractCalls::Yeet(ref __self_0),
                                &SimpleContractCalls::Yeet(ref __arg_1_0),
                            ) => (*__self_0) == (*__arg_1_0),
                            _ => unsafe { ::core::intrinsics::unreachable() },
                        }
                    } else {
                        false
                    }
                }
            }
            #[inline]
            fn ne(&self, other: &SimpleContractCalls) -> bool {
                {
                    let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                    let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            (
                                &SimpleContractCalls::Bar(ref __self_0),
                                &SimpleContractCalls::Bar(ref __arg_1_0),
                            ) => (*__self_0) != (*__arg_1_0),
                            (
                                &SimpleContractCalls::Foo(ref __self_0),
                                &SimpleContractCalls::Foo(ref __arg_1_0),
                            ) => (*__self_0) != (*__arg_1_0),
                            (
                                &SimpleContractCalls::Yeet(ref __self_0),
                                &SimpleContractCalls::Yeet(ref __arg_1_0),
                            ) => (*__self_0) != (*__arg_1_0),
                            _ => unsafe { ::core::intrinsics::unreachable() },
                        }
                    } else {
                        true
                    }
                }
            }
        }
        impl ::core::marker::StructuralEq for SimpleContractCalls {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for SimpleContractCalls {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<BarCall>;
                    let _: ::core::cmp::AssertParamIsEq<FooCall>;
                    let _: ::core::cmp::AssertParamIsEq<YeetCall>;
                }
            }
        }
        impl ethers_core::abi::Tokenizable for SimpleContractCalls {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let Ok(decoded) = BarCall::from_token(token.clone()) {
                    return Ok(SimpleContractCalls::Bar(decoded));
                }
                if let Ok(decoded) = FooCall::from_token(token.clone()) {
                    return Ok(SimpleContractCalls::Foo(decoded));
                }
                if let Ok(decoded) = YeetCall::from_token(token.clone()) {
                    return Ok(SimpleContractCalls::Yeet(decoded));
                }
                Err(ethers_core::abi::InvalidOutputType(
                    "Failed to decode all type variants".to_string(),
                ))
            }
            fn into_token(self) -> ethers_core::abi::Token {
                match self {
                    SimpleContractCalls::Bar(element) => element.into_token(),
                    SimpleContractCalls::Foo(element) => element.into_token(),
                    SimpleContractCalls::Yeet(element) => element.into_token(),
                }
            }
        }
        impl ethers_core::abi::TokenizableItem for SimpleContractCalls {}
        impl ethers_core::abi::AbiDecode for SimpleContractCalls {
            fn decode(data: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                if let Ok(decoded) = <BarCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
                {
                    return Ok(SimpleContractCalls::Bar(decoded));
                }
                if let Ok(decoded) = <FooCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
                {
                    return Ok(SimpleContractCalls::Foo(decoded));
                }
                if let Ok(decoded) =
                <YeetCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
                {
                    return Ok(SimpleContractCalls::Yeet(decoded));
                }
                Err(ethers_core::abi::Error::InvalidData.into())
            }
        }
        impl ethers_core::abi::AbiEncode for SimpleContractCalls {
            fn encode(self) -> Vec<u8> {
                match self {
                    SimpleContractCalls::Bar(element) => element.encode(),
                    SimpleContractCalls::Foo(element) => element.encode(),
                    SimpleContractCalls::Yeet(element) => element.encode(),
                }
            }
        }
        impl ::std::fmt::Display for SimpleContractCalls {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    SimpleContractCalls::Bar(element) => element.fmt(f),
                    SimpleContractCalls::Foo(element) => element.fmt(f),
                    SimpleContractCalls::Yeet(element) => element.fmt(f),
                }
            }
        }
        impl ::std::convert::From<BarCall> for SimpleContractCalls {
            fn from(var: BarCall) -> Self {
                SimpleContractCalls::Bar(var)
            }
        }
        impl ::std::convert::From<FooCall> for SimpleContractCalls {
            fn from(var: FooCall) -> Self {
                SimpleContractCalls::Foo(var)
            }
        }
        impl ::std::convert::From<YeetCall> for SimpleContractCalls {
            fn from(var: YeetCall) -> Self {
                SimpleContractCalls::Yeet(var)
            }
        }
        ///`Foo(uint256)`
        pub struct Foo {
            pub x: ethers_core::types::U256,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Foo {
            #[inline]
            fn clone(&self) -> Foo {
                match *self {
                    Foo { x: ref __self_0_0 } => Foo {
                        x: ::core::clone::Clone::clone(&(*__self_0_0)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Foo {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    Foo { x: ref __self_0_0 } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "Foo");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "x",
                            &&(*__self_0_0),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for Foo {
            #[inline]
            fn default() -> Foo {
                Foo {
                    x: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for Foo {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for Foo {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::U256>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for Foo {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Foo {
            #[inline]
            fn eq(&self, other: &Foo) -> bool {
                match *other {
                    Foo { x: ref __self_1_0 } => match *self {
                        Foo { x: ref __self_0_0 } => (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &Foo) -> bool {
                match *other {
                    Foo { x: ref __self_1_0 } => match *self {
                        Foo { x: ref __self_0_0 } => (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for Foo
            where
                ethers_core::types::U256: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 1usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&1usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        x: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [self.x.into_token()]))
            }
        }
        impl ethers_core::abi::TokenizableItem for Foo where
            ethers_core::types::U256: ethers_core::abi::Tokenize
        {
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for Foo {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __ignore,
                    }
                    struct __FieldVisitor;
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "field identifier")
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private::Ok(__Field::__field0),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                "x" => _serde::__private::Ok(__Field::__field0),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                        {
                            match __value {
                                b"x" => _serde::__private::Ok(__Field::__field0),
                                _ => _serde::__private::Ok(__Field::__ignore),
                            }
                        }
                    }
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    struct __Visitor<'de> {
                        marker: _serde::__private::PhantomData<Foo>,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Foo;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(__formatter, "struct Foo")
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match match _serde::de::SeqAccess::next_element::<
                                ethers_core::types::U256,
                            >(&mut __seq)
                            {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            } {
                                _serde::__private::Some(__value) => __value,
                                _serde::__private::None => {
                                    return _serde::__private::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct Foo with 1 element",
                                        ),
                                    );
                                }
                            };
                            _serde::__private::Ok(Foo { x: __field0 })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private::Option<ethers_core::types::U256> =
                                _serde::__private::None;
                            while let _serde::__private::Some(__key) =
                            match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                            {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private::Option::is_some(&__field0) {
                                            return _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "x",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private::Some(
                                            match _serde::de::MapAccess::next_value::<
                                                ethers_core::types::U256,
                                            >(
                                                &mut __map
                                            ) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            },
                                        );
                                    }
                                    _ => {
                                        let _ = match _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(
                                            &mut __map
                                        ) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        };
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private::Some(__field0) => __field0,
                                _serde::__private::None => {
                                    match _serde::__private::de::missing_field("x") {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    }
                                }
                            };
                            _serde::__private::Ok(Foo { x: __field0 })
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["x"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Foo",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<Foo>,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for Foo {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                {
                    let mut __serde_state = match _serde::Serializer::serialize_struct(
                        __serializer,
                        "Foo",
                        false as usize + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "x",
                        &self.x,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
    }
    assert_tokenizeable::<Foo>();
    let (client, _mock) = Provider::mocked();
    let contract = SimpleContract::new(Address::default(), Arc::new(client));
    let f = Foo { x: 100u64.into() };
    let _ = contract.foo(f);
    let call = BarCall {
        x: 1u64.into(),
        y: 0u64.into(),
        addr: Address::random(),
    };
    let encoded_call = contract.encode("bar", (call.x, call.y, call.addr)).unwrap();
    {
        match (&encoded_call, &call.clone().encode().into()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let decoded_call = BarCall::decode(encoded_call.as_ref()).unwrap();
    {
        match (&call, &decoded_call) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let contract_call = SimpleContractCalls::Bar(call);
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    {
        match (&contract_call, &decoded_enum) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&encoded_call, &contract_call.encode().into()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let call = YeetCall(1u64.into(), 0u64.into(), Address::zero());
    let encoded_call = contract.encode("yeet", (call.0, call.1, call.2)).unwrap();
    {
        match (&encoded_call, &call.clone().encode().into()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let decoded_call = YeetCall::decode(encoded_call.as_ref()).unwrap();
    {
        match (&call, &decoded_call) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let contract_call = SimpleContractCalls::Yeet(call.clone());
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    {
        match (&contract_call, &decoded_enum) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&contract_call, &call.into()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&encoded_call, &contract_call.encode().into()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const can_handle_overloaded_functions: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("can_handle_overloaded_functions"),
        ignore: false,
        allow_fail: false,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(can_handle_overloaded_functions())),
};
fn can_handle_overloaded_functions() {
    pub use simplecontract_mod::*;
    #[allow(clippy::too_many_arguments)]
    mod simplecontract_mod {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
        #![allow(clippy::type_complexity)]
        #![allow(unused_imports)]
        ///SimpleContract was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
        use std::sync::Arc;
        use ethers_core::{
            abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
            types::*,
        };
        use ethers_contract::{
            Contract,
            builders::{ContractCall, Event},
            Lazy,
        };
        use ethers_providers::Middleware;
        pub static SIMPLECONTRACT_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
            ethers_contract::Lazy::new(|| {
                ethers_core :: abi :: parse_abi_str ("[\n        getValue() (uint256)\n        getValue(uint256 otherValue) (uint256)\n        getValue(uint256 otherValue, address addr) (uint256)\n        log(string, string)\n        log(string)\n    ]") . expect ("invalid abi")
            });
        pub struct SimpleContract<M>(ethers_contract::Contract<M>);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<M: ::core::clone::Clone> ::core::clone::Clone for SimpleContract<M> {
            #[inline]
            fn clone(&self) -> SimpleContract<M> {
                match *self {
                    SimpleContract(ref __self_0_0) => {
                        SimpleContract(::core::clone::Clone::clone(&(*__self_0_0)))
                    }
                }
            }
        }
        impl<M> std::ops::Deref for SimpleContract<M> {
            type Target = ethers_contract::Contract<M>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<M: ethers_providers::Middleware> std::fmt::Debug for SimpleContract<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple("SimpleContract")
                    .field(&self.address())
                    .finish()
            }
        }
        impl<'a, M: ethers_providers::Middleware> SimpleContract<M> {
            /// Creates a new contract instance with the specified `ethers`
            /// client at the given `Address`. The contract derefs to a `ethers::Contract`
            /// object
            pub fn new<T: Into<ethers_core::types::Address>>(
                address: T,
                client: ::std::sync::Arc<M>,
            ) -> Self {
                let contract = ethers_contract::Contract::new(
                    address.into(),
                    SIMPLECONTRACT_ABI.clone(),
                    client,
                );
                Self(contract)
            }
            ///Calls the contract's `getValue` (0x20965255) function
            pub fn get_value(
                &self,
            ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
                self.0
                    .method_hash([32, 150, 82, 85], ())
                    .expect("method not found (this should never happen)")
            }
            ///Calls the contract's `getValue` (0x0ff4c916) function
            pub fn get_value_with_other_value(
                &self,
                other_value: ethers_core::types::U256,
            ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
                self.0
                    .method_hash([15, 244, 201, 22], other_value)
                    .expect("method not found (this should never happen)")
            }
            ///Calls the contract's `getValue` (0x0e611d38) function
            pub fn get_value_with_other_value_and_addr(
                &self,
                other_value: ethers_core::types::U256,
                addr: ethers_core::types::Address,
            ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
                self.0
                    .method_hash([14, 97, 29, 56], (other_value, addr))
                    .expect("method not found (this should never happen)")
            }
            ///Calls the contract's `log` (0x4b5c4277) function
            pub fn log_with__and_(
                &self,
                p0: String,
                p1: String,
            ) -> ethers_contract::builders::ContractCall<M, ()> {
                self.0
                    .method_hash([75, 92, 66, 119], (p0, p1))
                    .expect("method not found (this should never happen)")
            }
            ///Calls the contract's `log` (0x41304fac) function
            pub fn log(&self, p0: String) -> ethers_contract::builders::ContractCall<M, ()> {
                self.0
                    .method_hash([65, 48, 79, 172], p0)
                    .expect("method not found (this should never happen)")
            }
        }
        ///Container type for all input parameters for the `getValue`function with signature `getValue()` and selector `[32, 150, 82, 85]`
        #[ethcall(name = "getValue", abi = "getValue()")]
        pub struct GetValueCall;
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for GetValueCall {
            #[inline]
            fn clone(&self) -> GetValueCall {
                match *self {
                    GetValueCall => GetValueCall,
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for GetValueCall {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    GetValueCall => ::core::fmt::Formatter::write_str(f, "GetValueCall"),
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for GetValueCall {
            #[inline]
            fn default() -> GetValueCall {
                GetValueCall {}
            }
        }
        impl ::core::marker::StructuralEq for GetValueCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for GetValueCall {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {}
            }
        }
        impl ::core::marker::StructuralPartialEq for GetValueCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for GetValueCall {
            #[inline]
            fn eq(&self, other: &GetValueCall) -> bool {
                match *other {
                    GetValueCall => match *self {
                        GetValueCall => true,
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for GetValueCall {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if !tokens.is_empty() {
                        Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected empty tuple, got "],
                                &match (&tokens,) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Debug::fmt,
                                    )],
                                },
                            ));
                            res
                        }))
                    } else {
                        Ok(GetValueCall {})
                    }
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(::std::vec::Vec::new())
            }
        }
        impl ethers_core::abi::TokenizableItem for GetValueCall {}
        impl ethers_contract::EthCall for GetValueCall {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                "getValue".into()
            }
            fn selector() -> ethers_core::types::Selector {
                [32, 150, 82, 85]
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "getValue()".into()
            }
        }
        impl ethers_core::abi::AbiDecode for GetValueCall {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                let bytes = bytes.as_ref();
                if bytes.len() < 4 || bytes[..4] != <Self as ethers_contract::EthCall>::selector() {
                    return Err(ethers_contract::AbiError::WrongSelector);
                }
                let data_types = [];
                let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                    ethers_core::abi::Token::Tuple(data_tokens),
                )?)
            }
        }
        impl ethers_core::abi::AbiEncode for GetValueCall {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                let selector = <Self as ethers_contract::EthCall>::selector();
                let encoded = ethers_core::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }
        impl ::std::fmt::Display for GetValueCall {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                Ok(())
            }
        }
        ///Container type for all input parameters for the `getValue`function with signature `getValue(uint256)` and selector `[15, 244, 201, 22]`
        #[ethcall(name = "getValue", abi = "getValue(uint256)")]
        pub struct GetValueWithOtherValueCall {
            pub other_value: ethers_core::types::U256,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for GetValueWithOtherValueCall {
            #[inline]
            fn clone(&self) -> GetValueWithOtherValueCall {
                match *self {
                    GetValueWithOtherValueCall {
                        other_value: ref __self_0_0,
                    } => GetValueWithOtherValueCall {
                        other_value: ::core::clone::Clone::clone(&(*__self_0_0)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for GetValueWithOtherValueCall {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    GetValueWithOtherValueCall {
                        other_value: ref __self_0_0,
                    } => {
                        let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(
                            f,
                            "GetValueWithOtherValueCall",
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "other_value",
                            &&(*__self_0_0),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for GetValueWithOtherValueCall {
            #[inline]
            fn default() -> GetValueWithOtherValueCall {
                GetValueWithOtherValueCall {
                    other_value: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for GetValueWithOtherValueCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for GetValueWithOtherValueCall {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::U256>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for GetValueWithOtherValueCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for GetValueWithOtherValueCall {
            #[inline]
            fn eq(&self, other: &GetValueWithOtherValueCall) -> bool {
                match *other {
                    GetValueWithOtherValueCall {
                        other_value: ref __self_1_0,
                    } => match *self {
                        GetValueWithOtherValueCall {
                            other_value: ref __self_0_0,
                        } => (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &GetValueWithOtherValueCall) -> bool {
                match *other {
                    GetValueWithOtherValueCall {
                        other_value: ref __self_1_0,
                    } => match *self {
                        GetValueWithOtherValueCall {
                            other_value: ref __self_0_0,
                        } => (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for GetValueWithOtherValueCall
            where
                ethers_core::types::U256: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 1usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&1usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        other_value: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [self.other_value.into_token()]))
            }
        }
        impl ethers_core::abi::TokenizableItem for GetValueWithOtherValueCall where
            ethers_core::types::U256: ethers_core::abi::Tokenize
        {
        }
        impl ethers_contract::EthCall for GetValueWithOtherValueCall {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                "getValue".into()
            }
            fn selector() -> ethers_core::types::Selector {
                [15, 244, 201, 22]
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "getValue(uint256)".into()
            }
        }
        impl ethers_core::abi::AbiDecode for GetValueWithOtherValueCall {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                let bytes = bytes.as_ref();
                if bytes.len() < 4 || bytes[..4] != <Self as ethers_contract::EthCall>::selector() {
                    return Err(ethers_contract::AbiError::WrongSelector);
                }
                let data_types = [ethers_core::abi::ParamType::Uint(256usize)];
                let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                    ethers_core::abi::Token::Tuple(data_tokens),
                )?)
            }
        }
        impl ethers_core::abi::AbiEncode for GetValueWithOtherValueCall {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                let selector = <Self as ethers_contract::EthCall>::selector();
                let encoded = ethers_core::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }
        impl ::std::fmt::Display for GetValueWithOtherValueCall {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.other_value,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                Ok(())
            }
        }
        ///Container type for all input parameters for the `getValue`function with signature `getValue(uint256,address)` and selector `[14, 97, 29, 56]`
        #[ethcall(name = "getValue", abi = "getValue(uint256,address)")]
        pub struct GetValueWithOtherValueAndAddrCall {
            pub other_value: ethers_core::types::U256,
            pub addr: ethers_core::types::Address,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for GetValueWithOtherValueAndAddrCall {
            #[inline]
            fn clone(&self) -> GetValueWithOtherValueAndAddrCall {
                match *self {
                    GetValueWithOtherValueAndAddrCall {
                        other_value: ref __self_0_0,
                        addr: ref __self_0_1,
                    } => GetValueWithOtherValueAndAddrCall {
                        other_value: ::core::clone::Clone::clone(&(*__self_0_0)),
                        addr: ::core::clone::Clone::clone(&(*__self_0_1)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for GetValueWithOtherValueAndAddrCall {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    GetValueWithOtherValueAndAddrCall {
                        other_value: ref __self_0_0,
                        addr: ref __self_0_1,
                    } => {
                        let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(
                            f,
                            "GetValueWithOtherValueAndAddrCall",
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "other_value",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "addr",
                            &&(*__self_0_1),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for GetValueWithOtherValueAndAddrCall {
            #[inline]
            fn default() -> GetValueWithOtherValueAndAddrCall {
                GetValueWithOtherValueAndAddrCall {
                    other_value: ::core::default::Default::default(),
                    addr: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for GetValueWithOtherValueAndAddrCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for GetValueWithOtherValueAndAddrCall {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::U256>;
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::Address>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for GetValueWithOtherValueAndAddrCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for GetValueWithOtherValueAndAddrCall {
            #[inline]
            fn eq(&self, other: &GetValueWithOtherValueAndAddrCall) -> bool {
                match *other {
                    GetValueWithOtherValueAndAddrCall {
                        other_value: ref __self_1_0,
                        addr: ref __self_1_1,
                    } => match *self {
                        GetValueWithOtherValueAndAddrCall {
                            other_value: ref __self_0_0,
                            addr: ref __self_0_1,
                        } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &GetValueWithOtherValueAndAddrCall) -> bool {
                match *other {
                    GetValueWithOtherValueAndAddrCall {
                        other_value: ref __self_1_0,
                        addr: ref __self_1_1,
                    } => match *self {
                        GetValueWithOtherValueAndAddrCall {
                            other_value: ref __self_0_0,
                            addr: ref __self_0_1,
                        } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for GetValueWithOtherValueAndAddrCall
            where
                ethers_core::types::U256: ethers_core::abi::Tokenize,
                ethers_core::types::Address: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 2usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&2usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self {
                        other_value: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        addr: ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.other_value.into_token(),
                    self.addr.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for GetValueWithOtherValueAndAddrCall
            where
                ethers_core::types::U256: ethers_core::abi::Tokenize,
                ethers_core::types::Address: ethers_core::abi::Tokenize,
        {
        }
        impl ethers_contract::EthCall for GetValueWithOtherValueAndAddrCall {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                "getValue".into()
            }
            fn selector() -> ethers_core::types::Selector {
                [14, 97, 29, 56]
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "getValue(uint256,address)".into()
            }
        }
        impl ethers_core::abi::AbiDecode for GetValueWithOtherValueAndAddrCall {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                let bytes = bytes.as_ref();
                if bytes.len() < 4 || bytes[..4] != <Self as ethers_contract::EthCall>::selector() {
                    return Err(ethers_contract::AbiError::WrongSelector);
                }
                let data_types = [
                    ethers_core::abi::ParamType::Uint(256usize),
                    ethers_core::abi::ParamType::Address,
                ];
                let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                    ethers_core::abi::Token::Tuple(data_tokens),
                )?)
            }
        }
        impl ethers_core::abi::AbiEncode for GetValueWithOtherValueAndAddrCall {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                let selector = <Self as ethers_contract::EthCall>::selector();
                let encoded = ethers_core::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }
        impl ::std::fmt::Display for GetValueWithOtherValueAndAddrCall {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.other_value,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.addr,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                ))?;
                Ok(())
            }
        }
        ///Container type for all input parameters for the `log`function with signature `log(string,string)` and selector `[75, 92, 66, 119]`
        #[ethcall(name = "log", abi = "log(string,string)")]
        pub struct LogWithAndCall(pub String, pub String);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for LogWithAndCall {
            #[inline]
            fn clone(&self) -> LogWithAndCall {
                match *self {
                    LogWithAndCall(ref __self_0_0, ref __self_0_1) => LogWithAndCall(
                        ::core::clone::Clone::clone(&(*__self_0_0)),
                        ::core::clone::Clone::clone(&(*__self_0_1)),
                    ),
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for LogWithAndCall {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    LogWithAndCall(ref __self_0_0, ref __self_0_1) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "LogWithAndCall");
                        let _ =
                            ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                        let _ =
                            ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_1));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for LogWithAndCall {
            #[inline]
            fn default() -> LogWithAndCall {
                LogWithAndCall(
                    ::core::default::Default::default(),
                    ::core::default::Default::default(),
                )
            }
        }
        impl ::core::marker::StructuralEq for LogWithAndCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for LogWithAndCall {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<String>;
                    let _: ::core::cmp::AssertParamIsEq<String>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for LogWithAndCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for LogWithAndCall {
            #[inline]
            fn eq(&self, other: &LogWithAndCall) -> bool {
                match *other {
                    LogWithAndCall(ref __self_1_0, ref __self_1_1) => match *self {
                        LogWithAndCall(ref __self_0_0, ref __self_0_1) => {
                            (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &LogWithAndCall) -> bool {
                match *other {
                    LogWithAndCall(ref __self_1_0, ref __self_1_1) => match *self {
                        LogWithAndCall(ref __self_0_0, ref __self_0_1) => {
                            (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1)
                        }
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for LogWithAndCall
            where
                String: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 2usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&2usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self(
                        ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    ))
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.0.into_token(),
                    self.1.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for LogWithAndCall
            where
                String: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
        }
        impl ethers_contract::EthCall for LogWithAndCall {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                "log".into()
            }
            fn selector() -> ethers_core::types::Selector {
                [75, 92, 66, 119]
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "log(string,string)".into()
            }
        }
        impl ethers_core::abi::AbiDecode for LogWithAndCall {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                let bytes = bytes.as_ref();
                if bytes.len() < 4 || bytes[..4] != <Self as ethers_contract::EthCall>::selector() {
                    return Err(ethers_contract::AbiError::WrongSelector);
                }
                let data_types = [
                    ethers_core::abi::ParamType::String,
                    ethers_core::abi::ParamType::String,
                ];
                let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                    ethers_core::abi::Token::Tuple(data_tokens),
                )?)
            }
        }
        impl ethers_core::abi::AbiEncode for LogWithAndCall {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                let selector = <Self as ethers_contract::EthCall>::selector();
                let encoded = ethers_core::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }
        impl ::std::fmt::Display for LogWithAndCall {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                self.1.fmt(f)?;
                Ok(())
            }
        }
        ///Container type for all input parameters for the `log`function with signature `log(string)` and selector `[65, 48, 79, 172]`
        #[ethcall(name = "log", abi = "log(string)")]
        pub struct LogCall(pub String);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for LogCall {
            #[inline]
            fn clone(&self) -> LogCall {
                match *self {
                    LogCall(ref __self_0_0) => LogCall(::core::clone::Clone::clone(&(*__self_0_0))),
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for LogCall {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    LogCall(ref __self_0_0) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "LogCall");
                        let _ =
                            ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for LogCall {
            #[inline]
            fn default() -> LogCall {
                LogCall(::core::default::Default::default())
            }
        }
        impl ::core::marker::StructuralEq for LogCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for LogCall {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<String>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for LogCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for LogCall {
            #[inline]
            fn eq(&self, other: &LogCall) -> bool {
                match *other {
                    LogCall(ref __self_1_0) => match *self {
                        LogCall(ref __self_0_0) => (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &LogCall) -> bool {
                match *other {
                    LogCall(ref __self_1_0) => match *self {
                        LogCall(ref __self_0_0) => (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for LogCall
            where
                String: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 1usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&1usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self(ethers_core::abi::Tokenizable::from_token(
                        iter.next()
                            .expect("tokens size is sufficient qed")
                            .into_token(),
                    )?))
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [self.0.into_token()]))
            }
        }
        impl ethers_core::abi::TokenizableItem for LogCall where String: ethers_core::abi::Tokenize {}
        impl ethers_contract::EthCall for LogCall {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                "log".into()
            }
            fn selector() -> ethers_core::types::Selector {
                [65, 48, 79, 172]
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "log(string)".into()
            }
        }
        impl ethers_core::abi::AbiDecode for LogCall {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                let bytes = bytes.as_ref();
                if bytes.len() < 4 || bytes[..4] != <Self as ethers_contract::EthCall>::selector() {
                    return Err(ethers_contract::AbiError::WrongSelector);
                }
                let data_types = [ethers_core::abi::ParamType::String];
                let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                    ethers_core::abi::Token::Tuple(data_tokens),
                )?)
            }
        }
        impl ethers_core::abi::AbiEncode for LogCall {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                let selector = <Self as ethers_contract::EthCall>::selector();
                let encoded = ethers_core::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }
        impl ::std::fmt::Display for LogCall {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)?;
                Ok(())
            }
        }
        pub enum SimpleContractCalls {
            GetValue(GetValueCall),
            GetValueWithOtherValue(GetValueWithOtherValueCall),
            GetValueWithOtherValueAndAddr(GetValueWithOtherValueAndAddrCall),
            LogWithAnd(LogWithAndCall),
            Log(LogCall),
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for SimpleContractCalls {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&SimpleContractCalls::GetValue(ref __self_0),) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "GetValue");
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                    (&SimpleContractCalls::GetValueWithOtherValue(ref __self_0),) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "GetValueWithOtherValue");
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                    (&SimpleContractCalls::GetValueWithOtherValueAndAddr(ref __self_0),) => {
                        let debug_trait_builder = &mut ::core::fmt::Formatter::debug_tuple(
                            f,
                            "GetValueWithOtherValueAndAddr",
                        );
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                    (&SimpleContractCalls::LogWithAnd(ref __self_0),) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "LogWithAnd");
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                    (&SimpleContractCalls::Log(ref __self_0),) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "Log");
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for SimpleContractCalls {
            #[inline]
            fn clone(&self) -> SimpleContractCalls {
                match (&*self,) {
                    (&SimpleContractCalls::GetValue(ref __self_0),) => {
                        SimpleContractCalls::GetValue(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&SimpleContractCalls::GetValueWithOtherValue(ref __self_0),) => {
                        SimpleContractCalls::GetValueWithOtherValue(::core::clone::Clone::clone(
                            &(*__self_0),
                        ))
                    }
                    (&SimpleContractCalls::GetValueWithOtherValueAndAddr(ref __self_0),) => {
                        SimpleContractCalls::GetValueWithOtherValueAndAddr(
                            ::core::clone::Clone::clone(&(*__self_0)),
                        )
                    }
                    (&SimpleContractCalls::LogWithAnd(ref __self_0),) => {
                        SimpleContractCalls::LogWithAnd(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&SimpleContractCalls::Log(ref __self_0),) => {
                        SimpleContractCalls::Log(::core::clone::Clone::clone(&(*__self_0)))
                    }
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for SimpleContractCalls {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for SimpleContractCalls {
            #[inline]
            fn eq(&self, other: &SimpleContractCalls) -> bool {
                {
                    let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                    let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            (
                                &SimpleContractCalls::GetValue(ref __self_0),
                                &SimpleContractCalls::GetValue(ref __arg_1_0),
                            ) => (*__self_0) == (*__arg_1_0),
                            (
                                &SimpleContractCalls::GetValueWithOtherValue(ref __self_0),
                                &SimpleContractCalls::GetValueWithOtherValue(ref __arg_1_0),
                            ) => (*__self_0) == (*__arg_1_0),
                            (
                                &SimpleContractCalls::GetValueWithOtherValueAndAddr(ref __self_0),
                                &SimpleContractCalls::GetValueWithOtherValueAndAddr(ref __arg_1_0),
                            ) => (*__self_0) == (*__arg_1_0),
                            (
                                &SimpleContractCalls::LogWithAnd(ref __self_0),
                                &SimpleContractCalls::LogWithAnd(ref __arg_1_0),
                            ) => (*__self_0) == (*__arg_1_0),
                            (
                                &SimpleContractCalls::Log(ref __self_0),
                                &SimpleContractCalls::Log(ref __arg_1_0),
                            ) => (*__self_0) == (*__arg_1_0),
                            _ => unsafe { ::core::intrinsics::unreachable() },
                        }
                    } else {
                        false
                    }
                }
            }
            #[inline]
            fn ne(&self, other: &SimpleContractCalls) -> bool {
                {
                    let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                    let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            (
                                &SimpleContractCalls::GetValue(ref __self_0),
                                &SimpleContractCalls::GetValue(ref __arg_1_0),
                            ) => (*__self_0) != (*__arg_1_0),
                            (
                                &SimpleContractCalls::GetValueWithOtherValue(ref __self_0),
                                &SimpleContractCalls::GetValueWithOtherValue(ref __arg_1_0),
                            ) => (*__self_0) != (*__arg_1_0),
                            (
                                &SimpleContractCalls::GetValueWithOtherValueAndAddr(ref __self_0),
                                &SimpleContractCalls::GetValueWithOtherValueAndAddr(ref __arg_1_0),
                            ) => (*__self_0) != (*__arg_1_0),
                            (
                                &SimpleContractCalls::LogWithAnd(ref __self_0),
                                &SimpleContractCalls::LogWithAnd(ref __arg_1_0),
                            ) => (*__self_0) != (*__arg_1_0),
                            (
                                &SimpleContractCalls::Log(ref __self_0),
                                &SimpleContractCalls::Log(ref __arg_1_0),
                            ) => (*__self_0) != (*__arg_1_0),
                            _ => unsafe { ::core::intrinsics::unreachable() },
                        }
                    } else {
                        true
                    }
                }
            }
        }
        impl ::core::marker::StructuralEq for SimpleContractCalls {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for SimpleContractCalls {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<GetValueCall>;
                    let _: ::core::cmp::AssertParamIsEq<GetValueWithOtherValueCall>;
                    let _: ::core::cmp::AssertParamIsEq<GetValueWithOtherValueAndAddrCall>;
                    let _: ::core::cmp::AssertParamIsEq<LogWithAndCall>;
                    let _: ::core::cmp::AssertParamIsEq<LogCall>;
                }
            }
        }
        impl ethers_core::abi::Tokenizable for SimpleContractCalls {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let Ok(decoded) = GetValueCall::from_token(token.clone()) {
                    return Ok(SimpleContractCalls::GetValue(decoded));
                }
                if let Ok(decoded) = GetValueWithOtherValueCall::from_token(token.clone()) {
                    return Ok(SimpleContractCalls::GetValueWithOtherValue(decoded));
                }
                if let Ok(decoded) = GetValueWithOtherValueAndAddrCall::from_token(token.clone()) {
                    return Ok(SimpleContractCalls::GetValueWithOtherValueAndAddr(decoded));
                }
                if let Ok(decoded) = LogWithAndCall::from_token(token.clone()) {
                    return Ok(SimpleContractCalls::LogWithAnd(decoded));
                }
                if let Ok(decoded) = LogCall::from_token(token.clone()) {
                    return Ok(SimpleContractCalls::Log(decoded));
                }
                Err(ethers_core::abi::InvalidOutputType(
                    "Failed to decode all type variants".to_string(),
                ))
            }
            fn into_token(self) -> ethers_core::abi::Token {
                match self {
                    SimpleContractCalls::GetValue(element) => element.into_token(),
                    SimpleContractCalls::GetValueWithOtherValue(element) => element.into_token(),
                    SimpleContractCalls::GetValueWithOtherValueAndAddr(element) => {
                        element.into_token()
                    }
                    SimpleContractCalls::LogWithAnd(element) => element.into_token(),
                    SimpleContractCalls::Log(element) => element.into_token(),
                }
            }
        }
        impl ethers_core::abi::TokenizableItem for SimpleContractCalls {}
        impl ethers_core::abi::AbiDecode for SimpleContractCalls {
            fn decode(data: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                if let Ok(decoded) =
                <GetValueCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
                {
                    return Ok(SimpleContractCalls::GetValue(decoded));
                }
                if let Ok(decoded) =
                <GetValueWithOtherValueCall as ethers_core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
                {
                    return Ok(SimpleContractCalls::GetValueWithOtherValue(decoded));
                }
                if let Ok(decoded) =
                <GetValueWithOtherValueAndAddrCall as ethers_core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
                {
                    return Ok(SimpleContractCalls::GetValueWithOtherValueAndAddr(decoded));
                }
                if let Ok(decoded) =
                <LogWithAndCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
                {
                    return Ok(SimpleContractCalls::LogWithAnd(decoded));
                }
                if let Ok(decoded) = <LogCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
                {
                    return Ok(SimpleContractCalls::Log(decoded));
                }
                Err(ethers_core::abi::Error::InvalidData.into())
            }
        }
        impl ethers_core::abi::AbiEncode for SimpleContractCalls {
            fn encode(self) -> Vec<u8> {
                match self {
                    SimpleContractCalls::GetValue(element) => element.encode(),
                    SimpleContractCalls::GetValueWithOtherValue(element) => element.encode(),
                    SimpleContractCalls::GetValueWithOtherValueAndAddr(element) => element.encode(),
                    SimpleContractCalls::LogWithAnd(element) => element.encode(),
                    SimpleContractCalls::Log(element) => element.encode(),
                }
            }
        }
        impl ::std::fmt::Display for SimpleContractCalls {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    SimpleContractCalls::GetValue(element) => element.fmt(f),
                    SimpleContractCalls::GetValueWithOtherValue(element) => element.fmt(f),
                    SimpleContractCalls::GetValueWithOtherValueAndAddr(element) => element.fmt(f),
                    SimpleContractCalls::LogWithAnd(element) => element.fmt(f),
                    SimpleContractCalls::Log(element) => element.fmt(f),
                }
            }
        }
        impl ::std::convert::From<GetValueCall> for SimpleContractCalls {
            fn from(var: GetValueCall) -> Self {
                SimpleContractCalls::GetValue(var)
            }
        }
        impl ::std::convert::From<GetValueWithOtherValueCall> for SimpleContractCalls {
            fn from(var: GetValueWithOtherValueCall) -> Self {
                SimpleContractCalls::GetValueWithOtherValue(var)
            }
        }
        impl ::std::convert::From<GetValueWithOtherValueAndAddrCall> for SimpleContractCalls {
            fn from(var: GetValueWithOtherValueAndAddrCall) -> Self {
                SimpleContractCalls::GetValueWithOtherValueAndAddr(var)
            }
        }
        impl ::std::convert::From<LogWithAndCall> for SimpleContractCalls {
            fn from(var: LogWithAndCall) -> Self {
                SimpleContractCalls::LogWithAnd(var)
            }
        }
        impl ::std::convert::From<LogCall> for SimpleContractCalls {
            fn from(var: LogCall) -> Self {
                SimpleContractCalls::Log(var)
            }
        }
    }
    let (provider, _) = Provider::mocked();
    let client = Arc::new(provider);
    let contract = SimpleContract::new(Address::zero(), client);
    let _ = contract.get_value();
    let _ = contract.get_value_with_other_value(1337u64.into());
    let _ = contract.get_value_with_other_value_and_addr(1337u64.into(), Address::zero());
    let call = GetValueCall;
    let encoded_call = contract.encode("getValue", ()).unwrap();
    {
        match (&encoded_call, &call.clone().encode().into()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let decoded_call = GetValueCall::decode(encoded_call.as_ref()).unwrap();
    {
        match (&call, &decoded_call) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let contract_call = SimpleContractCalls::GetValue(call);
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    {
        match (&contract_call, &decoded_enum) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&encoded_call, &contract_call.encode().into()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let call = GetValueWithOtherValueCall {
        other_value: 420u64.into(),
    };
    let encoded_call = contract
        .encode_with_selector([15, 244, 201, 22], call.other_value)
        .unwrap();
    {
        match (&encoded_call, &call.clone().encode().into()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let decoded_call = GetValueWithOtherValueCall::decode(encoded_call.as_ref()).unwrap();
    {
        match (&call, &decoded_call) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let contract_call = SimpleContractCalls::GetValueWithOtherValue(call);
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    {
        match (&contract_call, &decoded_enum) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&encoded_call, &contract_call.encode().into()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let call = GetValueWithOtherValueAndAddrCall {
        other_value: 420u64.into(),
        addr: Address::random(),
    };
    let encoded_call = contract
        .encode_with_selector([14, 97, 29, 56], (call.other_value, call.addr))
        .unwrap();
    let decoded_call = GetValueWithOtherValueAndAddrCall::decode(encoded_call.as_ref()).unwrap();
    {
        match (&call, &decoded_call) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let contract_call = SimpleContractCalls::GetValueWithOtherValueAndAddr(call);
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    {
        match (&contract_call, &decoded_enum) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&encoded_call, &contract_call.encode().into()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let call = LogCall("message".to_string());
    let contract_call = SimpleContractCalls::Log(call);
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const can_handle_even_more_overloaded_functions: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("can_handle_even_more_overloaded_functions"),
        ignore: false,
        allow_fail: false,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| {
        test::assert_test_result(can_handle_even_more_overloaded_functions())
    }),
};
fn can_handle_even_more_overloaded_functions() {
    pub use consolelog_mod::*;
    #[allow(clippy::too_many_arguments)]
    mod consolelog_mod {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
        #![allow(clippy::type_complexity)]
        #![allow(unused_imports)]
        ///ConsoleLog was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
        use std::sync::Arc;
        use ethers_core::{
            abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
            types::*,
        };
        use ethers_contract::{
            Contract,
            builders::{ContractCall, Event},
            Lazy,
        };
        use ethers_providers::Middleware;
        pub static CONSOLELOG_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
            ethers_contract::Lazy::new(|| {
                ethers_core::abi::parse_abi_str(
                    "[\n            log(string, string)\n            log(string)\n    ]",
                )
                    .expect("invalid abi")
            });
        pub struct ConsoleLog<M>(ethers_contract::Contract<M>);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<M: ::core::clone::Clone> ::core::clone::Clone for ConsoleLog<M> {
            #[inline]
            fn clone(&self) -> ConsoleLog<M> {
                match *self {
                    ConsoleLog(ref __self_0_0) => {
                        ConsoleLog(::core::clone::Clone::clone(&(*__self_0_0)))
                    }
                }
            }
        }
        impl<M> std::ops::Deref for ConsoleLog<M> {
            type Target = ethers_contract::Contract<M>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<M: ethers_providers::Middleware> std::fmt::Debug for ConsoleLog<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple("ConsoleLog").field(&self.address()).finish()
            }
        }
        impl<'a, M: ethers_providers::Middleware> ConsoleLog<M> {
            /// Creates a new contract instance with the specified `ethers`
            /// client at the given `Address`. The contract derefs to a `ethers::Contract`
            /// object
            pub fn new<T: Into<ethers_core::types::Address>>(
                address: T,
                client: ::std::sync::Arc<M>,
            ) -> Self {
                let contract =
                    ethers_contract::Contract::new(address.into(), CONSOLELOG_ABI.clone(), client);
                Self(contract)
            }
            ///Calls the contract's `log` (0x4b5c4277) function
            pub fn log_with__and_(
                &self,
                p0: String,
                p1: String,
            ) -> ethers_contract::builders::ContractCall<M, ()> {
                self.0
                    .method_hash([75, 92, 66, 119], (p0, p1))
                    .expect("method not found (this should never happen)")
            }
            ///Calls the contract's `log` (0x41304fac) function
            pub fn log(&self, p0: String) -> ethers_contract::builders::ContractCall<M, ()> {
                self.0
                    .method_hash([65, 48, 79, 172], p0)
                    .expect("method not found (this should never happen)")
            }
        }
        ///Container type for all input parameters for the `log`function with signature `log(string,string)` and selector `[75, 92, 66, 119]`
        #[ethcall(name = "log", abi = "log(string,string)")]
        pub struct LogWithAndCall(pub String, pub String);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for LogWithAndCall {
            #[inline]
            fn clone(&self) -> LogWithAndCall {
                match *self {
                    LogWithAndCall(ref __self_0_0, ref __self_0_1) => LogWithAndCall(
                        ::core::clone::Clone::clone(&(*__self_0_0)),
                        ::core::clone::Clone::clone(&(*__self_0_1)),
                    ),
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for LogWithAndCall {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    LogWithAndCall(ref __self_0_0, ref __self_0_1) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "LogWithAndCall");
                        let _ =
                            ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                        let _ =
                            ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_1));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for LogWithAndCall {
            #[inline]
            fn default() -> LogWithAndCall {
                LogWithAndCall(
                    ::core::default::Default::default(),
                    ::core::default::Default::default(),
                )
            }
        }
        impl ::core::marker::StructuralEq for LogWithAndCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for LogWithAndCall {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<String>;
                    let _: ::core::cmp::AssertParamIsEq<String>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for LogWithAndCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for LogWithAndCall {
            #[inline]
            fn eq(&self, other: &LogWithAndCall) -> bool {
                match *other {
                    LogWithAndCall(ref __self_1_0, ref __self_1_1) => match *self {
                        LogWithAndCall(ref __self_0_0, ref __self_0_1) => {
                            (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &LogWithAndCall) -> bool {
                match *other {
                    LogWithAndCall(ref __self_1_0, ref __self_1_1) => match *self {
                        LogWithAndCall(ref __self_0_0, ref __self_0_1) => {
                            (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1)
                        }
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for LogWithAndCall
            where
                String: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 2usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&2usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self(
                        ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                        ethers_core::abi::Tokenizable::from_token(
                            iter.next()
                                .expect("tokens size is sufficient qed")
                                .into_token(),
                        )?,
                    ))
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [
                    self.0.into_token(),
                    self.1.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for LogWithAndCall
            where
                String: ethers_core::abi::Tokenize,
                String: ethers_core::abi::Tokenize,
        {
        }
        impl ethers_contract::EthCall for LogWithAndCall {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                "log".into()
            }
            fn selector() -> ethers_core::types::Selector {
                [75, 92, 66, 119]
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "log(string,string)".into()
            }
        }
        impl ethers_core::abi::AbiDecode for LogWithAndCall {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                let bytes = bytes.as_ref();
                if bytes.len() < 4 || bytes[..4] != <Self as ethers_contract::EthCall>::selector() {
                    return Err(ethers_contract::AbiError::WrongSelector);
                }
                let data_types = [
                    ethers_core::abi::ParamType::String,
                    ethers_core::abi::ParamType::String,
                ];
                let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                    ethers_core::abi::Token::Tuple(data_tokens),
                )?)
            }
        }
        impl ethers_core::abi::AbiEncode for LogWithAndCall {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                let selector = <Self as ethers_contract::EthCall>::selector();
                let encoded = ethers_core::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }
        impl ::std::fmt::Display for LogWithAndCall {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)?;
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[", "],
                    &match () {
                        () => [],
                    },
                ))?;
                self.1.fmt(f)?;
                Ok(())
            }
        }
        ///Container type for all input parameters for the `log`function with signature `log(string)` and selector `[65, 48, 79, 172]`
        #[ethcall(name = "log", abi = "log(string)")]
        pub struct LogCall(pub String);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for LogCall {
            #[inline]
            fn clone(&self) -> LogCall {
                match *self {
                    LogCall(ref __self_0_0) => LogCall(::core::clone::Clone::clone(&(*__self_0_0))),
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for LogCall {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    LogCall(ref __self_0_0) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "LogCall");
                        let _ =
                            ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for LogCall {
            #[inline]
            fn default() -> LogCall {
                LogCall(::core::default::Default::default())
            }
        }
        impl ::core::marker::StructuralEq for LogCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for LogCall {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<String>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for LogCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for LogCall {
            #[inline]
            fn eq(&self, other: &LogCall) -> bool {
                match *other {
                    LogCall(ref __self_1_0) => match *self {
                        LogCall(ref __self_0_0) => (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &LogCall) -> bool {
                match *other {
                    LogCall(ref __self_1_0) => match *self {
                        LogCall(ref __self_0_0) => (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for LogCall
            where
                String: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != 1usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&1usize, &tokens.len(), &tokens) {
                                    (arg0, arg1, arg2) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Debug::fmt),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    let mut iter = tokens.into_iter();
                    Ok(Self(ethers_core::abi::Tokenizable::from_token(
                        iter.next()
                            .expect("tokens size is sufficient qed")
                            .into_token(),
                    )?))
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                (arg0,) => {
                                    [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                                }
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [self.0.into_token()]))
            }
        }
        impl ethers_core::abi::TokenizableItem for LogCall where String: ethers_core::abi::Tokenize {}
        impl ethers_contract::EthCall for LogCall {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                "log".into()
            }
            fn selector() -> ethers_core::types::Selector {
                [65, 48, 79, 172]
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                "log(string)".into()
            }
        }
        impl ethers_core::abi::AbiDecode for LogCall {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                let bytes = bytes.as_ref();
                if bytes.len() < 4 || bytes[..4] != <Self as ethers_contract::EthCall>::selector() {
                    return Err(ethers_contract::AbiError::WrongSelector);
                }
                let data_types = [ethers_core::abi::ParamType::String];
                let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                    ethers_core::abi::Token::Tuple(data_tokens),
                )?)
            }
        }
        impl ethers_core::abi::AbiEncode for LogCall {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens = ethers_core::abi::Tokenize::into_tokens(self);
                let selector = <Self as ethers_contract::EthCall>::selector();
                let encoded = ethers_core::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }
        impl ::std::fmt::Display for LogCall {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)?;
                Ok(())
            }
        }
        pub enum ConsoleLogCalls {
            LogWithAnd(LogWithAndCall),
            Log(LogCall),
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for ConsoleLogCalls {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&ConsoleLogCalls::LogWithAnd(ref __self_0),) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "LogWithAnd");
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                    (&ConsoleLogCalls::Log(ref __self_0),) => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_tuple(f, "Log");
                        let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0));
                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for ConsoleLogCalls {
            #[inline]
            fn clone(&self) -> ConsoleLogCalls {
                match (&*self,) {
                    (&ConsoleLogCalls::LogWithAnd(ref __self_0),) => {
                        ConsoleLogCalls::LogWithAnd(::core::clone::Clone::clone(&(*__self_0)))
                    }
                    (&ConsoleLogCalls::Log(ref __self_0),) => {
                        ConsoleLogCalls::Log(::core::clone::Clone::clone(&(*__self_0)))
                    }
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for ConsoleLogCalls {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for ConsoleLogCalls {
            #[inline]
            fn eq(&self, other: &ConsoleLogCalls) -> bool {
                {
                    let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                    let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            (
                                &ConsoleLogCalls::LogWithAnd(ref __self_0),
                                &ConsoleLogCalls::LogWithAnd(ref __arg_1_0),
                            ) => (*__self_0) == (*__arg_1_0),
                            (
                                &ConsoleLogCalls::Log(ref __self_0),
                                &ConsoleLogCalls::Log(ref __arg_1_0),
                            ) => (*__self_0) == (*__arg_1_0),
                            _ => unsafe { ::core::intrinsics::unreachable() },
                        }
                    } else {
                        false
                    }
                }
            }
            #[inline]
            fn ne(&self, other: &ConsoleLogCalls) -> bool {
                {
                    let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                    let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            (
                                &ConsoleLogCalls::LogWithAnd(ref __self_0),
                                &ConsoleLogCalls::LogWithAnd(ref __arg_1_0),
                            ) => (*__self_0) != (*__arg_1_0),
                            (
                                &ConsoleLogCalls::Log(ref __self_0),
                                &ConsoleLogCalls::Log(ref __arg_1_0),
                            ) => (*__self_0) != (*__arg_1_0),
                            _ => unsafe { ::core::intrinsics::unreachable() },
                        }
                    } else {
                        true
                    }
                }
            }
        }
        impl ::core::marker::StructuralEq for ConsoleLogCalls {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for ConsoleLogCalls {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<LogWithAndCall>;
                    let _: ::core::cmp::AssertParamIsEq<LogCall>;
                }
            }
        }
        impl ethers_core::abi::Tokenizable for ConsoleLogCalls {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
                where
                    Self: Sized,
            {
                if let Ok(decoded) = LogWithAndCall::from_token(token.clone()) {
                    return Ok(ConsoleLogCalls::LogWithAnd(decoded));
                }
                if let Ok(decoded) = LogCall::from_token(token.clone()) {
                    return Ok(ConsoleLogCalls::Log(decoded));
                }
                Err(ethers_core::abi::InvalidOutputType(
                    "Failed to decode all type variants".to_string(),
                ))
            }
            fn into_token(self) -> ethers_core::abi::Token {
                match self {
                    ConsoleLogCalls::LogWithAnd(element) => element.into_token(),
                    ConsoleLogCalls::Log(element) => element.into_token(),
                }
            }
        }
        impl ethers_core::abi::TokenizableItem for ConsoleLogCalls {}
        impl ethers_core::abi::AbiDecode for ConsoleLogCalls {
            fn decode(data: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                if let Ok(decoded) =
                <LogWithAndCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
                {
                    return Ok(ConsoleLogCalls::LogWithAnd(decoded));
                }
                if let Ok(decoded) = <LogCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
                {
                    return Ok(ConsoleLogCalls::Log(decoded));
                }
                Err(ethers_core::abi::Error::InvalidData.into())
            }
        }
        impl ethers_core::abi::AbiEncode for ConsoleLogCalls {
            fn encode(self) -> Vec<u8> {
                match self {
                    ConsoleLogCalls::LogWithAnd(element) => element.encode(),
                    ConsoleLogCalls::Log(element) => element.encode(),
                }
            }
        }
        impl ::std::fmt::Display for ConsoleLogCalls {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    ConsoleLogCalls::LogWithAnd(element) => element.fmt(f),
                    ConsoleLogCalls::Log(element) => element.fmt(f),
                }
            }
        }
        impl ::std::convert::From<LogWithAndCall> for ConsoleLogCalls {
            fn from(var: LogWithAndCall) -> Self {
                ConsoleLogCalls::LogWithAnd(var)
            }
        }
        impl ::std::convert::From<LogCall> for ConsoleLogCalls {
            fn from(var: LogCall) -> Self {
                ConsoleLogCalls::Log(var)
            }
        }
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const can_handle_underscore_functions: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("can_handle_underscore_functions"),
        ignore: false,
        allow_fail: false,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(can_handle_underscore_functions())),
};
fn can_handle_underscore_functions() {
    let body = async { pub mod __shared_types { } pub use simplestorage_mod :: * ; # [allow (clippy :: too_many_arguments)] mod simplestorage_mod { # ! [allow (clippy :: enum_variant_names)] # ! [allow (dead_code)] # ! [allow (clippy :: type_complexity)] # ! [allow (unused_imports)] # [doc = "SimpleStorage was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"] use std :: sync :: Arc ; use ethers_core :: { abi :: { Abi , Token , Detokenize , InvalidOutputType , Tokenizable } , types :: * } ; use ethers_contract :: { Contract , builders :: { ContractCall , Event } , Lazy } ; use ethers_providers :: Middleware ; pub static SIMPLESTORAGE_ABI : ethers_contract :: Lazy < ethers_core :: abi :: Abi > = ethers_contract :: Lazy :: new (| | ethers_core :: abi :: parse_abi_str ("[\n            _hashPuzzle() (uint256)\n        ]") . expect ("invalid abi")) ; pub struct SimpleStorage < M > (ethers_contract :: Contract < M >) ; # [automatically_derived] # [allow (unused_qualifications)] impl < M : :: core :: clone :: Clone > :: core :: clone :: Clone for SimpleStorage < M > { # [inline] fn clone (& self) -> SimpleStorage < M > { match * self { SimpleStorage (ref __self_0_0) => SimpleStorage (:: core :: clone :: Clone :: clone (& (* __self_0_0))) , } } } impl < M > std :: ops :: Deref for SimpleStorage < M > { type Target = ethers_contract :: Contract < M > ; fn deref (& self) -> & Self :: Target { & self . 0 } } impl < M : ethers_providers :: Middleware > std :: fmt :: Debug for SimpleStorage < M > { fn fmt (& self , f : & mut std :: fmt :: Formatter) -> std :: fmt :: Result { f . debug_tuple ("SimpleStorage") . field (& self . address ()) . finish () } } impl < 'a , M : ethers_providers :: Middleware > SimpleStorage < M > { # [doc = r" Creates a new contract instance with the specified `ethers`"] # [doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"] # [doc = r" object"] pub fn new < T : Into < ethers_core :: types :: Address > > (address : T , client : :: std :: sync :: Arc < M >) -> Self { let contract = ethers_contract :: Contract :: new (address . into () , SIMPLESTORAGE_ABI . clone () , client) ; Self (contract) } # [doc = "Calls the contract\'s `_hashPuzzle` (0x018ba991) function"] pub fn hash_puzzle (& self) -> ethers_contract :: builders :: ContractCall < M , ethers_core :: types :: U256 > { self . 0 . method_hash ([1 , 139 , 169 , 145] , ()) . expect ("method not found (this should never happen)") } } # [doc = "Container type for all input parameters for the `_hashPuzzle`function with signature `_hashPuzzle()` and selector `[1, 139, 169, 145]`"] # [ethcall (name = "_hashPuzzle" , abi = "_hashPuzzle()")] pub struct HashPuzzleCall ; # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: clone :: Clone for HashPuzzleCall { # [inline] fn clone (& self) -> HashPuzzleCall { match * self { HashPuzzleCall => HashPuzzleCall , } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: fmt :: Debug for HashPuzzleCall { fn fmt (& self , f : & mut :: core :: fmt :: Formatter) -> :: core :: fmt :: Result { match * self { HashPuzzleCall => { :: core :: fmt :: Formatter :: write_str (f , "HashPuzzleCall") } } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: default :: Default for HashPuzzleCall { # [inline] fn default () -> HashPuzzleCall { HashPuzzleCall { } } } impl :: core :: marker :: StructuralEq for HashPuzzleCall { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: Eq for HashPuzzleCall { # [inline] # [doc (hidden)] # [no_coverage] fn assert_receiver_is_total_eq (& self) -> () { { } } } impl :: core :: marker :: StructuralPartialEq for HashPuzzleCall { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: PartialEq for HashPuzzleCall { # [inline] fn eq (& self , other : & HashPuzzleCall) -> bool { match * other { HashPuzzleCall => match * self { HashPuzzleCall => true , } , } } } impl ethers_core :: abi :: Tokenizable for HashPuzzleCall { fn from_token (token : ethers_core :: abi :: Token) -> Result < Self , ethers_core :: abi :: InvalidOutputType > where Self : Sized { if let ethers_core :: abi :: Token :: Tuple (tokens) = token { if ! tokens . is_empty () { Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected empty tuple, got "] , & match (& tokens ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) } else { Ok (HashPuzzleCall { }) } } else { Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected Tuple, got "] , & match (& token ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) } } fn into_token (self) -> ethers_core :: abi :: Token { ethers_core :: abi :: Token :: Tuple (:: std :: vec :: Vec :: new ()) } } impl ethers_core :: abi :: TokenizableItem for HashPuzzleCall { } impl ethers_contract :: EthCall for HashPuzzleCall { fn function_name () -> :: std :: borrow :: Cow < 'static , str > { "_hashPuzzle" . into () } fn selector () -> ethers_core :: types :: Selector { [1 , 139 , 169 , 145] } fn abi_signature () -> :: std :: borrow :: Cow < 'static , str > { "_hashPuzzle()" . into () } } impl ethers_core :: abi :: AbiDecode for HashPuzzleCall { fn decode (bytes : impl AsRef < [u8] >) -> Result < Self , ethers_core :: abi :: AbiError > { let bytes = bytes . as_ref () ; if bytes . len () < 4 || bytes [.. 4] != < Self as ethers_contract :: EthCall > :: selector () { return Err (ethers_contract :: AbiError :: WrongSelector) ; } let data_types = [] ; let data_tokens = ethers_core :: abi :: decode (& data_types , & bytes [4 ..]) ? ; Ok (< Self as ethers_core :: abi :: Tokenizable > :: from_token (ethers_core :: abi :: Token :: Tuple (data_tokens)) ?) } } impl ethers_core :: abi :: AbiEncode for HashPuzzleCall { fn encode (self) -> :: std :: vec :: Vec < u8 > { let tokens = ethers_core :: abi :: Tokenize :: into_tokens (self) ; let selector = < Self as ethers_contract :: EthCall > :: selector () ; let encoded = ethers_core :: abi :: encode (& tokens) ; selector . iter () . copied () . chain (encoded . into_iter ()) . collect () } } impl :: std :: fmt :: Display for HashPuzzleCall { fn fmt (& self , f : & mut :: std :: fmt :: Formatter < '_ >) -> :: std :: fmt :: Result { Ok (()) } } } pub use simplestorage2_mod :: * ; # [allow (clippy :: too_many_arguments)] mod simplestorage2_mod { # ! [allow (clippy :: enum_variant_names)] # ! [allow (dead_code)] # ! [allow (clippy :: type_complexity)] # ! [allow (unused_imports)] # [doc = "SimpleStorage2 was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"] use std :: sync :: Arc ; use ethers_core :: { abi :: { Abi , Token , Detokenize , InvalidOutputType , Tokenizable } , types :: * } ; use ethers_contract :: { Contract , builders :: { ContractCall , Event } , Lazy } ; use ethers_providers :: Middleware ; pub static SIMPLESTORAGE2_ABI : ethers_contract :: Lazy < ethers_core :: abi :: Abi > = ethers_contract :: Lazy :: new (| | serde_json :: from_str ("[\n  {\n    \"inputs\": [{ \"internalType\": \"string\", \"name\": \"value\", \"type\": \"string\" }],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"constructor\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": true,\n        \"internalType\": \"address\",\n        \"name\": \"author\",\n        \"type\": \"address\"\n      },\n      {\n        \"indexed\": true,\n        \"internalType\": \"address\",\n        \"name\": \"oldAuthor\",\n        \"type\": \"address\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"string\",\n        \"name\": \"oldValue\",\n        \"type\": \"string\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"string\",\n        \"name\": \"newValue\",\n        \"type\": \"string\"\n      }\n    ],\n    \"name\": \"ValueChanged\",\n    \"type\": \"event\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"_hashPuzzle\",\n    \"outputs\": [{ \"internalType\": \"uint256\", \"name\": \"\", \"type\": \"uint256\" }],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"getValue\",\n    \"outputs\": [{ \"internalType\": \"string\", \"name\": \"\", \"type\": \"string\" }],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"lastSender\",\n    \"outputs\": [{ \"internalType\": \"address\", \"name\": \"\", \"type\": \"address\" }],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [{ \"internalType\": \"string\", \"name\": \"value\", \"type\": \"string\" }],\n    \"name\": \"setValue\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      { \"internalType\": \"string\", \"name\": \"value\", \"type\": \"string\" },\n      { \"internalType\": \"string\", \"name\": \"value2\", \"type\": \"string\" }\n    ],\n    \"name\": \"setValues\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  }\n]\n\n") . expect ("invalid abi")) ; pub struct SimpleStorage2 < M > (ethers_contract :: Contract < M >) ; # [automatically_derived] # [allow (unused_qualifications)] impl < M : :: core :: clone :: Clone > :: core :: clone :: Clone for SimpleStorage2 < M > { # [inline] fn clone (& self) -> SimpleStorage2 < M > { match * self { SimpleStorage2 (ref __self_0_0) => SimpleStorage2 (:: core :: clone :: Clone :: clone (& (* __self_0_0))) , } } } impl < M > std :: ops :: Deref for SimpleStorage2 < M > { type Target = ethers_contract :: Contract < M > ; fn deref (& self) -> & Self :: Target { & self . 0 } } impl < M : ethers_providers :: Middleware > std :: fmt :: Debug for SimpleStorage2 < M > { fn fmt (& self , f : & mut std :: fmt :: Formatter) -> std :: fmt :: Result { f . debug_tuple ("SimpleStorage2") . field (& self . address ()) . finish () } } impl < 'a , M : ethers_providers :: Middleware > SimpleStorage2 < M > { # [doc = r" Creates a new contract instance with the specified `ethers`"] # [doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"] # [doc = r" object"] pub fn new < T : Into < ethers_core :: types :: Address > > (address : T , client : :: std :: sync :: Arc < M >) -> Self { let contract = ethers_contract :: Contract :: new (address . into () , SIMPLESTORAGE2_ABI . clone () , client) ; Self (contract) } # [doc = "Calls the contract\'s `_hashPuzzle` (0x018ba991) function"] pub fn hash_puzzle (& self) -> ethers_contract :: builders :: ContractCall < M , ethers_core :: types :: U256 > { self . 0 . method_hash ([1 , 139 , 169 , 145] , ()) . expect ("method not found (this should never happen)") } # [doc = "Calls the contract\'s `getValue` (0x20965255) function"] pub fn get_value (& self) -> ethers_contract :: builders :: ContractCall < M , String > { self . 0 . method_hash ([32 , 150 , 82 , 85] , ()) . expect ("method not found (this should never happen)") } # [doc = "Calls the contract\'s `lastSender` (0x256fec88) function"] pub fn last_sender (& self) -> ethers_contract :: builders :: ContractCall < M , ethers_core :: types :: Address > { self . 0 . method_hash ([37 , 111 , 236 , 136] , ()) . expect ("method not found (this should never happen)") } # [doc = "Calls the contract\'s `setValue` (0x93a09352) function"] pub fn set_value (& self , value : String) -> ethers_contract :: builders :: ContractCall < M , () > { self . 0 . method_hash ([147 , 160 , 147 , 82] , value) . expect ("method not found (this should never happen)") } # [doc = "Calls the contract\'s `setValues` (0x7ffaa4b6) function"] pub fn set_values (& self , value : String , value_2 : String) -> ethers_contract :: builders :: ContractCall < M , () > { self . 0 . method_hash ([127 , 250 , 164 , 182] , (value , value_2)) . expect ("method not found (this should never happen)") } # [doc = "Gets the contract\'s `ValueChanged` event"] pub fn value_changed_filter (& self) -> ethers_contract :: builders :: Event < M , ValueChangedFilter > { self . 0 . event () } # [doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"] pub fn events (& self) -> ethers_contract :: builders :: Event < M , ValueChangedFilter > { self . 0 . event_with_filter (Default :: default ()) } } # [ethevent (name = "ValueChanged" , abi = "ValueChanged(address,address,string,string)")] pub struct ValueChangedFilter { # [ethevent (indexed)] pub author : ethers_core :: types :: Address , # [ethevent (indexed)] pub old_author : ethers_core :: types :: Address , pub old_value : String , pub new_value : String , } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: clone :: Clone for ValueChangedFilter { # [inline] fn clone (& self) -> ValueChangedFilter { match * self { ValueChangedFilter { author : ref __self_0_0 , old_author : ref __self_0_1 , old_value : ref __self_0_2 , new_value : ref __self_0_3 } => ValueChangedFilter { author : :: core :: clone :: Clone :: clone (& (* __self_0_0)) , old_author : :: core :: clone :: Clone :: clone (& (* __self_0_1)) , old_value : :: core :: clone :: Clone :: clone (& (* __self_0_2)) , new_value : :: core :: clone :: Clone :: clone (& (* __self_0_3)) , } , } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: fmt :: Debug for ValueChangedFilter { fn fmt (& self , f : & mut :: core :: fmt :: Formatter) -> :: core :: fmt :: Result { match * self { ValueChangedFilter { author : ref __self_0_0 , old_author : ref __self_0_1 , old_value : ref __self_0_2 , new_value : ref __self_0_3 } => { let debug_trait_builder = & mut :: core :: fmt :: Formatter :: debug_struct (f , "ValueChangedFilter") ; let _ = :: core :: fmt :: DebugStruct :: field (debug_trait_builder , "author" , & & (* __self_0_0)) ; let _ = :: core :: fmt :: DebugStruct :: field (debug_trait_builder , "old_author" , & & (* __self_0_1)) ; let _ = :: core :: fmt :: DebugStruct :: field (debug_trait_builder , "old_value" , & & (* __self_0_2)) ; let _ = :: core :: fmt :: DebugStruct :: field (debug_trait_builder , "new_value" , & & (* __self_0_3)) ; :: core :: fmt :: DebugStruct :: finish (debug_trait_builder) } } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: default :: Default for ValueChangedFilter { # [inline] fn default () -> ValueChangedFilter { ValueChangedFilter { author : :: core :: default :: Default :: default () , old_author : :: core :: default :: Default :: default () , old_value : :: core :: default :: Default :: default () , new_value : :: core :: default :: Default :: default () , } } } impl :: core :: marker :: StructuralEq for ValueChangedFilter { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: Eq for ValueChangedFilter { # [inline] # [doc (hidden)] # [no_coverage] fn assert_receiver_is_total_eq (& self) -> () { { let _ : :: core :: cmp :: AssertParamIsEq < ethers_core :: types :: Address > ; let _ : :: core :: cmp :: AssertParamIsEq < ethers_core :: types :: Address > ; let _ : :: core :: cmp :: AssertParamIsEq < String > ; let _ : :: core :: cmp :: AssertParamIsEq < String > ; } } } impl :: core :: marker :: StructuralPartialEq for ValueChangedFilter { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: PartialEq for ValueChangedFilter { # [inline] fn eq (& self , other : & ValueChangedFilter) -> bool { match * other { ValueChangedFilter { author : ref __self_1_0 , old_author : ref __self_1_1 , old_value : ref __self_1_2 , new_value : ref __self_1_3 } => match * self { ValueChangedFilter { author : ref __self_0_0 , old_author : ref __self_0_1 , old_value : ref __self_0_2 , new_value : ref __self_0_3 } => (* __self_0_0) == (* __self_1_0) && (* __self_0_1) == (* __self_1_1) && (* __self_0_2) == (* __self_1_2) && (* __self_0_3) == (* __self_1_3) , } , } } # [inline] fn ne (& self , other : & ValueChangedFilter) -> bool { match * other { ValueChangedFilter { author : ref __self_1_0 , old_author : ref __self_1_1 , old_value : ref __self_1_2 , new_value : ref __self_1_3 } => match * self { ValueChangedFilter { author : ref __self_0_0 , old_author : ref __self_0_1 , old_value : ref __self_0_2 , new_value : ref __self_0_3 } => (* __self_0_0) != (* __self_1_0) || (* __self_0_1) != (* __self_1_1) || (* __self_0_2) != (* __self_1_2) || (* __self_0_3) != (* __self_1_3) , } , } } } impl ethers_core :: abi :: Tokenizable for ValueChangedFilter < > where ethers_core :: types :: Address : ethers_core :: abi :: Tokenize , ethers_core :: types :: Address : ethers_core :: abi :: Tokenize , String : ethers_core :: abi :: Tokenize , String : ethers_core :: abi :: Tokenize { fn from_token (token : ethers_core :: abi :: Token) -> Result < Self , ethers_core :: abi :: InvalidOutputType > where Self : Sized { if let ethers_core :: abi :: Token :: Tuple (tokens) = token { if tokens . len () != 4usize { return Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected " , " tokens, got " , ": "] , & match (& 4usize , & tokens . len () , & tokens) { (arg0 , arg1 , arg2) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Display :: fmt) , :: core :: fmt :: ArgumentV1 :: new (arg1 , :: core :: fmt :: Display :: fmt) , :: core :: fmt :: ArgumentV1 :: new (arg2 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) ; } let mut iter = tokens . into_iter () ; Ok (Self { author : ethers_core :: abi :: Tokenizable :: from_token (iter . next () . expect ("tokens size is sufficient qed") . into_token ()) ? , old_author : ethers_core :: abi :: Tokenizable :: from_token (iter . next () . expect ("tokens size is sufficient qed") . into_token ()) ? , old_value : ethers_core :: abi :: Tokenizable :: from_token (iter . next () . expect ("tokens size is sufficient qed") . into_token ()) ? , new_value : ethers_core :: abi :: Tokenizable :: from_token (iter . next () . expect ("tokens size is sufficient qed") . into_token ()) ? , }) } else { Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected Tuple, got "] , & match (& token ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) } } fn into_token (self) -> ethers_core :: abi :: Token { ethers_core :: abi :: Token :: Tuple (< [_] > :: into_vec (box [self . author . into_token () , self . old_author . into_token () , self . old_value . into_token () , self . new_value . into_token ()])) } } impl ethers_core :: abi :: TokenizableItem for ValueChangedFilter < > where ethers_core :: types :: Address : ethers_core :: abi :: Tokenize , ethers_core :: types :: Address : ethers_core :: abi :: Tokenize , String : ethers_core :: abi :: Tokenize , String : ethers_core :: abi :: Tokenize { } impl ethers_contract :: EthEvent for ValueChangedFilter { fn name () -> :: std :: borrow :: Cow < 'static , str > { "ValueChanged" . into () } fn signature () -> ethers_core :: types :: H256 { ethers_core :: types :: H256 ([153 , 155 , 109 , 70 , 76 , 78 , 51 , 131 , 195 , 65 , 189 , 211 , 162 , 43 , 2 , 221 , 168 , 167 , 225 , 214 , 156 , 6 , 157 , 37 , 46 , 53 , 203 , 46 , 226 , 244 , 163 , 195]) } fn abi_signature () -> :: std :: borrow :: Cow < 'static , str > { "ValueChanged(address,address,string,string)" . into () } fn decode_log (log : & ethers_core :: abi :: RawLog) -> Result < Self , ethers_core :: abi :: Error > where Self : Sized { let ethers_core :: abi :: RawLog { data , topics } = log ; let event_signature = topics . get (0) . ok_or (ethers_core :: abi :: Error :: InvalidData) ? ; if event_signature != & Self :: signature () { return Err (ethers_core :: abi :: Error :: InvalidData) ; } let topic_types = < [_] > :: into_vec (box [ethers_core :: abi :: ParamType :: Address , ethers_core :: abi :: ParamType :: Address]) ; let data_types = [ethers_core :: abi :: ParamType :: String , ethers_core :: abi :: ParamType :: String] ; let flat_topics = topics . iter () . skip (1) . flat_map (| t | t . as_ref () . to_vec ()) . collect :: < Vec < u8 > > () ; let topic_tokens = ethers_core :: abi :: decode (& topic_types , & flat_topics) ? ; if topic_tokens . len () != topics . len () - 1 { return Err (ethers_core :: abi :: Error :: InvalidData) ; } let data_tokens = ethers_core :: abi :: decode (& data_types , data) ? ; let tokens : Vec < _ > = topic_tokens . into_iter () . chain (data_tokens . into_iter ()) . collect () ; ethers_core :: abi :: Tokenizable :: from_token (ethers_core :: abi :: Token :: Tuple (tokens)) . map_err (| _ | ethers_core :: abi :: Error :: InvalidData) } fn is_anonymous () -> bool { false } } impl :: std :: fmt :: Display for ValueChangedFilter { fn fmt (& self , f : & mut :: std :: fmt :: Formatter < '_ >) -> :: std :: fmt :: Result { f . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& [""] , & match (& & self . author ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ? ; f . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& [", "] , & match () { () => [] , })) ? ; f . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& [""] , & match (& & self . old_author ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ? ; f . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& [", "] , & match () { () => [] , })) ? ; self . old_value . fmt (f) ? ; f . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& [", "] , & match () { () => [] , })) ? ; self . new_value . fmt (f) ? ; Ok (()) } } # [doc = "Container type for all input parameters for the `_hashPuzzle`function with signature `_hashPuzzle()` and selector `[1, 139, 169, 145]`"] # [ethcall (name = "_hashPuzzle" , abi = "_hashPuzzle()")] pub struct HashPuzzleCall ; # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: clone :: Clone for HashPuzzleCall { # [inline] fn clone (& self) -> HashPuzzleCall { match * self { HashPuzzleCall => HashPuzzleCall , } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: fmt :: Debug for HashPuzzleCall { fn fmt (& self , f : & mut :: core :: fmt :: Formatter) -> :: core :: fmt :: Result { match * self { HashPuzzleCall => { :: core :: fmt :: Formatter :: write_str (f , "HashPuzzleCall") } } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: default :: Default for HashPuzzleCall { # [inline] fn default () -> HashPuzzleCall { HashPuzzleCall { } } } impl :: core :: marker :: StructuralEq for HashPuzzleCall { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: Eq for HashPuzzleCall { # [inline] # [doc (hidden)] # [no_coverage] fn assert_receiver_is_total_eq (& self) -> () { { } } } impl :: core :: marker :: StructuralPartialEq for HashPuzzleCall { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: PartialEq for HashPuzzleCall { # [inline] fn eq (& self , other : & HashPuzzleCall) -> bool { match * other { HashPuzzleCall => match * self { HashPuzzleCall => true , } , } } } impl ethers_core :: abi :: Tokenizable for HashPuzzleCall { fn from_token (token : ethers_core :: abi :: Token) -> Result < Self , ethers_core :: abi :: InvalidOutputType > where Self : Sized { if let ethers_core :: abi :: Token :: Tuple (tokens) = token { if ! tokens . is_empty () { Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected empty tuple, got "] , & match (& tokens ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) } else { Ok (HashPuzzleCall { }) } } else { Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected Tuple, got "] , & match (& token ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) } } fn into_token (self) -> ethers_core :: abi :: Token { ethers_core :: abi :: Token :: Tuple (:: std :: vec :: Vec :: new ()) } } impl ethers_core :: abi :: TokenizableItem for HashPuzzleCall { } impl ethers_contract :: EthCall for HashPuzzleCall { fn function_name () -> :: std :: borrow :: Cow < 'static , str > { "_hashPuzzle" . into () } fn selector () -> ethers_core :: types :: Selector { [1 , 139 , 169 , 145] } fn abi_signature () -> :: std :: borrow :: Cow < 'static , str > { "_hashPuzzle()" . into () } } impl ethers_core :: abi :: AbiDecode for HashPuzzleCall { fn decode (bytes : impl AsRef < [u8] >) -> Result < Self , ethers_core :: abi :: AbiError > { let bytes = bytes . as_ref () ; if bytes . len () < 4 || bytes [.. 4] != < Self as ethers_contract :: EthCall > :: selector () { return Err (ethers_contract :: AbiError :: WrongSelector) ; } let data_types = [] ; let data_tokens = ethers_core :: abi :: decode (& data_types , & bytes [4 ..]) ? ; Ok (< Self as ethers_core :: abi :: Tokenizable > :: from_token (ethers_core :: abi :: Token :: Tuple (data_tokens)) ?) } } impl ethers_core :: abi :: AbiEncode for HashPuzzleCall { fn encode (self) -> :: std :: vec :: Vec < u8 > { let tokens = ethers_core :: abi :: Tokenize :: into_tokens (self) ; let selector = < Self as ethers_contract :: EthCall > :: selector () ; let encoded = ethers_core :: abi :: encode (& tokens) ; selector . iter () . copied () . chain (encoded . into_iter ()) . collect () } } impl :: std :: fmt :: Display for HashPuzzleCall { fn fmt (& self , f : & mut :: std :: fmt :: Formatter < '_ >) -> :: std :: fmt :: Result { Ok (()) } } # [doc = "Container type for all input parameters for the `getValue`function with signature `getValue()` and selector `[32, 150, 82, 85]`"] # [ethcall (name = "getValue" , abi = "getValue()")] pub struct GetValueCall ; # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: clone :: Clone for GetValueCall { # [inline] fn clone (& self) -> GetValueCall { match * self { GetValueCall => GetValueCall , } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: fmt :: Debug for GetValueCall { fn fmt (& self , f : & mut :: core :: fmt :: Formatter) -> :: core :: fmt :: Result { match * self { GetValueCall => { :: core :: fmt :: Formatter :: write_str (f , "GetValueCall") } } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: default :: Default for GetValueCall { # [inline] fn default () -> GetValueCall { GetValueCall { } } } impl :: core :: marker :: StructuralEq for GetValueCall { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: Eq for GetValueCall { # [inline] # [doc (hidden)] # [no_coverage] fn assert_receiver_is_total_eq (& self) -> () { { } } } impl :: core :: marker :: StructuralPartialEq for GetValueCall { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: PartialEq for GetValueCall { # [inline] fn eq (& self , other : & GetValueCall) -> bool { match * other { GetValueCall => match * self { GetValueCall => true , } , } } } impl ethers_core :: abi :: Tokenizable for GetValueCall { fn from_token (token : ethers_core :: abi :: Token) -> Result < Self , ethers_core :: abi :: InvalidOutputType > where Self : Sized { if let ethers_core :: abi :: Token :: Tuple (tokens) = token { if ! tokens . is_empty () { Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected empty tuple, got "] , & match (& tokens ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) } else { Ok (GetValueCall { }) } } else { Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected Tuple, got "] , & match (& token ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) } } fn into_token (self) -> ethers_core :: abi :: Token { ethers_core :: abi :: Token :: Tuple (:: std :: vec :: Vec :: new ()) } } impl ethers_core :: abi :: TokenizableItem for GetValueCall { } impl ethers_contract :: EthCall for GetValueCall { fn function_name () -> :: std :: borrow :: Cow < 'static , str > { "getValue" . into () } fn selector () -> ethers_core :: types :: Selector { [32 , 150 , 82 , 85] } fn abi_signature () -> :: std :: borrow :: Cow < 'static , str > { "getValue()" . into () } } impl ethers_core :: abi :: AbiDecode for GetValueCall { fn decode (bytes : impl AsRef < [u8] >) -> Result < Self , ethers_core :: abi :: AbiError > { let bytes = bytes . as_ref () ; if bytes . len () < 4 || bytes [.. 4] != < Self as ethers_contract :: EthCall > :: selector () { return Err (ethers_contract :: AbiError :: WrongSelector) ; } let data_types = [] ; let data_tokens = ethers_core :: abi :: decode (& data_types , & bytes [4 ..]) ? ; Ok (< Self as ethers_core :: abi :: Tokenizable > :: from_token (ethers_core :: abi :: Token :: Tuple (data_tokens)) ?) } } impl ethers_core :: abi :: AbiEncode for GetValueCall { fn encode (self) -> :: std :: vec :: Vec < u8 > { let tokens = ethers_core :: abi :: Tokenize :: into_tokens (self) ; let selector = < Self as ethers_contract :: EthCall > :: selector () ; let encoded = ethers_core :: abi :: encode (& tokens) ; selector . iter () . copied () . chain (encoded . into_iter ()) . collect () } } impl :: std :: fmt :: Display for GetValueCall { fn fmt (& self , f : & mut :: std :: fmt :: Formatter < '_ >) -> :: std :: fmt :: Result { Ok (()) } } # [doc = "Container type for all input parameters for the `lastSender`function with signature `lastSender()` and selector `[37, 111, 236, 136]`"] # [ethcall (name = "lastSender" , abi = "lastSender()")] pub struct LastSenderCall ; # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: clone :: Clone for LastSenderCall { # [inline] fn clone (& self) -> LastSenderCall { match * self { LastSenderCall => LastSenderCall , } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: fmt :: Debug for LastSenderCall { fn fmt (& self , f : & mut :: core :: fmt :: Formatter) -> :: core :: fmt :: Result { match * self { LastSenderCall => { :: core :: fmt :: Formatter :: write_str (f , "LastSenderCall") } } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: default :: Default for LastSenderCall { # [inline] fn default () -> LastSenderCall { LastSenderCall { } } } impl :: core :: marker :: StructuralEq for LastSenderCall { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: Eq for LastSenderCall { # [inline] # [doc (hidden)] # [no_coverage] fn assert_receiver_is_total_eq (& self) -> () { { } } } impl :: core :: marker :: StructuralPartialEq for LastSenderCall { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: PartialEq for LastSenderCall { # [inline] fn eq (& self , other : & LastSenderCall) -> bool { match * other { LastSenderCall => match * self { LastSenderCall => true , } , } } } impl ethers_core :: abi :: Tokenizable for LastSenderCall { fn from_token (token : ethers_core :: abi :: Token) -> Result < Self , ethers_core :: abi :: InvalidOutputType > where Self : Sized { if let ethers_core :: abi :: Token :: Tuple (tokens) = token { if ! tokens . is_empty () { Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected empty tuple, got "] , & match (& tokens ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) } else { Ok (LastSenderCall { }) } } else { Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected Tuple, got "] , & match (& token ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) } } fn into_token (self) -> ethers_core :: abi :: Token { ethers_core :: abi :: Token :: Tuple (:: std :: vec :: Vec :: new ()) } } impl ethers_core :: abi :: TokenizableItem for LastSenderCall { } impl ethers_contract :: EthCall for LastSenderCall { fn function_name () -> :: std :: borrow :: Cow < 'static , str > { "lastSender" . into () } fn selector () -> ethers_core :: types :: Selector { [37 , 111 , 236 , 136] } fn abi_signature () -> :: std :: borrow :: Cow < 'static , str > { "lastSender()" . into () } } impl ethers_core :: abi :: AbiDecode for LastSenderCall { fn decode (bytes : impl AsRef < [u8] >) -> Result < Self , ethers_core :: abi :: AbiError > { let bytes = bytes . as_ref () ; if bytes . len () < 4 || bytes [.. 4] != < Self as ethers_contract :: EthCall > :: selector () { return Err (ethers_contract :: AbiError :: WrongSelector) ; } let data_types = [] ; let data_tokens = ethers_core :: abi :: decode (& data_types , & bytes [4 ..]) ? ; Ok (< Self as ethers_core :: abi :: Tokenizable > :: from_token (ethers_core :: abi :: Token :: Tuple (data_tokens)) ?) } } impl ethers_core :: abi :: AbiEncode for LastSenderCall { fn encode (self) -> :: std :: vec :: Vec < u8 > { let tokens = ethers_core :: abi :: Tokenize :: into_tokens (self) ; let selector = < Self as ethers_contract :: EthCall > :: selector () ; let encoded = ethers_core :: abi :: encode (& tokens) ; selector . iter () . copied () . chain (encoded . into_iter ()) . collect () } } impl :: std :: fmt :: Display for LastSenderCall { fn fmt (& self , f : & mut :: std :: fmt :: Formatter < '_ >) -> :: std :: fmt :: Result { Ok (()) } } # [doc = "Container type for all input parameters for the `setValue`function with signature `setValue(string)` and selector `[147, 160, 147, 82]`"] # [ethcall (name = "setValue" , abi = "setValue(string)")] pub struct SetValueCall { pub value : String , } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: clone :: Clone for SetValueCall { # [inline] fn clone (& self) -> SetValueCall { match * self { SetValueCall { value : ref __self_0_0 } => SetValueCall { value : :: core :: clone :: Clone :: clone (& (* __self_0_0)) , } , } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: fmt :: Debug for SetValueCall { fn fmt (& self , f : & mut :: core :: fmt :: Formatter) -> :: core :: fmt :: Result { match * self { SetValueCall { value : ref __self_0_0 } => { let debug_trait_builder = & mut :: core :: fmt :: Formatter :: debug_struct (f , "SetValueCall") ; let _ = :: core :: fmt :: DebugStruct :: field (debug_trait_builder , "value" , & & (* __self_0_0)) ; :: core :: fmt :: DebugStruct :: finish (debug_trait_builder) } } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: default :: Default for SetValueCall { # [inline] fn default () -> SetValueCall { SetValueCall { value : :: core :: default :: Default :: default () , } } } impl :: core :: marker :: StructuralEq for SetValueCall { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: Eq for SetValueCall { # [inline] # [doc (hidden)] # [no_coverage] fn assert_receiver_is_total_eq (& self) -> () { { let _ : :: core :: cmp :: AssertParamIsEq < String > ; } } } impl :: core :: marker :: StructuralPartialEq for SetValueCall { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: PartialEq for SetValueCall { # [inline] fn eq (& self , other : & SetValueCall) -> bool { match * other { SetValueCall { value : ref __self_1_0 } => match * self { SetValueCall { value : ref __self_0_0 } => (* __self_0_0) == (* __self_1_0) , } , } } # [inline] fn ne (& self , other : & SetValueCall) -> bool { match * other { SetValueCall { value : ref __self_1_0 } => match * self { SetValueCall { value : ref __self_0_0 } => (* __self_0_0) != (* __self_1_0) , } , } } } impl ethers_core :: abi :: Tokenizable for SetValueCall < > where String : ethers_core :: abi :: Tokenize { fn from_token (token : ethers_core :: abi :: Token) -> Result < Self , ethers_core :: abi :: InvalidOutputType > where Self : Sized { if let ethers_core :: abi :: Token :: Tuple (tokens) = token { if tokens . len () != 1usize { return Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected " , " tokens, got " , ": "] , & match (& 1usize , & tokens . len () , & tokens) { (arg0 , arg1 , arg2) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Display :: fmt) , :: core :: fmt :: ArgumentV1 :: new (arg1 , :: core :: fmt :: Display :: fmt) , :: core :: fmt :: ArgumentV1 :: new (arg2 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) ; } let mut iter = tokens . into_iter () ; Ok (Self { value : ethers_core :: abi :: Tokenizable :: from_token (iter . next () . expect ("tokens size is sufficient qed") . into_token ()) ? , }) } else { Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected Tuple, got "] , & match (& token ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) } } fn into_token (self) -> ethers_core :: abi :: Token { ethers_core :: abi :: Token :: Tuple (< [_] > :: into_vec (box [self . value . into_token ()])) } } impl ethers_core :: abi :: TokenizableItem for SetValueCall < > where String : ethers_core :: abi :: Tokenize { } impl ethers_contract :: EthCall for SetValueCall { fn function_name () -> :: std :: borrow :: Cow < 'static , str > { "setValue" . into () } fn selector () -> ethers_core :: types :: Selector { [147 , 160 , 147 , 82] } fn abi_signature () -> :: std :: borrow :: Cow < 'static , str > { "setValue(string)" . into () } } impl ethers_core :: abi :: AbiDecode for SetValueCall { fn decode (bytes : impl AsRef < [u8] >) -> Result < Self , ethers_core :: abi :: AbiError > { let bytes = bytes . as_ref () ; if bytes . len () < 4 || bytes [.. 4] != < Self as ethers_contract :: EthCall > :: selector () { return Err (ethers_contract :: AbiError :: WrongSelector) ; } let data_types = [ethers_core :: abi :: ParamType :: String] ; let data_tokens = ethers_core :: abi :: decode (& data_types , & bytes [4 ..]) ? ; Ok (< Self as ethers_core :: abi :: Tokenizable > :: from_token (ethers_core :: abi :: Token :: Tuple (data_tokens)) ?) } } impl ethers_core :: abi :: AbiEncode for SetValueCall { fn encode (self) -> :: std :: vec :: Vec < u8 > { let tokens = ethers_core :: abi :: Tokenize :: into_tokens (self) ; let selector = < Self as ethers_contract :: EthCall > :: selector () ; let encoded = ethers_core :: abi :: encode (& tokens) ; selector . iter () . copied () . chain (encoded . into_iter ()) . collect () } } impl :: std :: fmt :: Display for SetValueCall { fn fmt (& self , f : & mut :: std :: fmt :: Formatter < '_ >) -> :: std :: fmt :: Result { self . value . fmt (f) ? ; Ok (()) } } # [doc = "Container type for all input parameters for the `setValues`function with signature `setValues(string,string)` and selector `[127, 250, 164, 182]`"] # [ethcall (name = "setValues" , abi = "setValues(string,string)")] pub struct SetValuesCall { pub value : String , pub value_2 : String , } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: clone :: Clone for SetValuesCall { # [inline] fn clone (& self) -> SetValuesCall { match * self { SetValuesCall { value : ref __self_0_0 , value_2 : ref __self_0_1 } => SetValuesCall { value : :: core :: clone :: Clone :: clone (& (* __self_0_0)) , value_2 : :: core :: clone :: Clone :: clone (& (* __self_0_1)) , } , } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: fmt :: Debug for SetValuesCall { fn fmt (& self , f : & mut :: core :: fmt :: Formatter) -> :: core :: fmt :: Result { match * self { SetValuesCall { value : ref __self_0_0 , value_2 : ref __self_0_1 } => { let debug_trait_builder = & mut :: core :: fmt :: Formatter :: debug_struct (f , "SetValuesCall") ; let _ = :: core :: fmt :: DebugStruct :: field (debug_trait_builder , "value" , & & (* __self_0_0)) ; let _ = :: core :: fmt :: DebugStruct :: field (debug_trait_builder , "value_2" , & & (* __self_0_1)) ; :: core :: fmt :: DebugStruct :: finish (debug_trait_builder) } } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: default :: Default for SetValuesCall { # [inline] fn default () -> SetValuesCall { SetValuesCall { value : :: core :: default :: Default :: default () , value_2 : :: core :: default :: Default :: default () , } } } impl :: core :: marker :: StructuralEq for SetValuesCall { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: Eq for SetValuesCall { # [inline] # [doc (hidden)] # [no_coverage] fn assert_receiver_is_total_eq (& self) -> () { { let _ : :: core :: cmp :: AssertParamIsEq < String > ; let _ : :: core :: cmp :: AssertParamIsEq < String > ; } } } impl :: core :: marker :: StructuralPartialEq for SetValuesCall { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: PartialEq for SetValuesCall { # [inline] fn eq (& self , other : & SetValuesCall) -> bool { match * other { SetValuesCall { value : ref __self_1_0 , value_2 : ref __self_1_1 } => match * self { SetValuesCall { value : ref __self_0_0 , value_2 : ref __self_0_1 } => (* __self_0_0) == (* __self_1_0) && (* __self_0_1) == (* __self_1_1) , } , } } # [inline] fn ne (& self , other : & SetValuesCall) -> bool { match * other { SetValuesCall { value : ref __self_1_0 , value_2 : ref __self_1_1 } => match * self { SetValuesCall { value : ref __self_0_0 , value_2 : ref __self_0_1 } => (* __self_0_0) != (* __self_1_0) || (* __self_0_1) != (* __self_1_1) , } , } } } impl ethers_core :: abi :: Tokenizable for SetValuesCall < > where String : ethers_core :: abi :: Tokenize , String : ethers_core :: abi :: Tokenize { fn from_token (token : ethers_core :: abi :: Token) -> Result < Self , ethers_core :: abi :: InvalidOutputType > where Self : Sized { if let ethers_core :: abi :: Token :: Tuple (tokens) = token { if tokens . len () != 2usize { return Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected " , " tokens, got " , ": "] , & match (& 2usize , & tokens . len () , & tokens) { (arg0 , arg1 , arg2) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Display :: fmt) , :: core :: fmt :: ArgumentV1 :: new (arg1 , :: core :: fmt :: Display :: fmt) , :: core :: fmt :: ArgumentV1 :: new (arg2 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) ; } let mut iter = tokens . into_iter () ; Ok (Self { value : ethers_core :: abi :: Tokenizable :: from_token (iter . next () . expect ("tokens size is sufficient qed") . into_token ()) ? , value_2 : ethers_core :: abi :: Tokenizable :: from_token (iter . next () . expect ("tokens size is sufficient qed") . into_token ()) ? , }) } else { Err (ethers_core :: abi :: InvalidOutputType ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["Expected Tuple, got "] , & match (& token ,) { (arg0 ,) => [:: core :: fmt :: ArgumentV1 :: new (arg0 , :: core :: fmt :: Debug :: fmt)] , })) ; res })) } } fn into_token (self) -> ethers_core :: abi :: Token { ethers_core :: abi :: Token :: Tuple (< [_] > :: into_vec (box [self . value . into_token () , self . value_2 . into_token ()])) } } impl ethers_core :: abi :: TokenizableItem for SetValuesCall < > where String : ethers_core :: abi :: Tokenize , String : ethers_core :: abi :: Tokenize { } impl ethers_contract :: EthCall for SetValuesCall { fn function_name () -> :: std :: borrow :: Cow < 'static , str > { "setValues" . into () } fn selector () -> ethers_core :: types :: Selector { [127 , 250 , 164 , 182] } fn abi_signature () -> :: std :: borrow :: Cow < 'static , str > { "setValues(string,string)" . into () } } impl ethers_core :: abi :: AbiDecode for SetValuesCall { fn decode (bytes : impl AsRef < [u8] >) -> Result < Self , ethers_core :: abi :: AbiError > { let bytes = bytes . as_ref () ; if bytes . len () < 4 || bytes [.. 4] != < Self as ethers_contract :: EthCall > :: selector () { return Err (ethers_contract :: AbiError :: WrongSelector) ; } let data_types = [ethers_core :: abi :: ParamType :: String , ethers_core :: abi :: ParamType :: String] ; let data_tokens = ethers_core :: abi :: decode (& data_types , & bytes [4 ..]) ? ; Ok (< Self as ethers_core :: abi :: Tokenizable > :: from_token (ethers_core :: abi :: Token :: Tuple (data_tokens)) ?) } } impl ethers_core :: abi :: AbiEncode for SetValuesCall { fn encode (self) -> :: std :: vec :: Vec < u8 > { let tokens = ethers_core :: abi :: Tokenize :: into_tokens (self) ; let selector = < Self as ethers_contract :: EthCall > :: selector () ; let encoded = ethers_core :: abi :: encode (& tokens) ; selector . iter () . copied () . chain (encoded . into_iter ()) . collect () } } impl :: std :: fmt :: Display for SetValuesCall { fn fmt (& self , f : & mut :: std :: fmt :: Formatter < '_ >) -> :: std :: fmt :: Result { self . value . fmt (f) ? ; f . write_fmt (:: core :: fmt :: Arguments :: new_v1 (& [", "] , & match () { () => [] , })) ? ; self . value_2 . fmt (f) ? ; Ok (()) } } pub enum SimpleStorage2Calls { HashPuzzle (HashPuzzleCall) , GetValue (GetValueCall) , LastSender (LastSenderCall) , SetValue (SetValueCall) , SetValues (SetValuesCall) , } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: fmt :: Debug for SimpleStorage2Calls { fn fmt (& self , f : & mut :: core :: fmt :: Formatter) -> :: core :: fmt :: Result { match (& * self ,) { (& SimpleStorage2Calls :: HashPuzzle (ref __self_0) ,) => { let debug_trait_builder = & mut :: core :: fmt :: Formatter :: debug_tuple (f , "HashPuzzle") ; let _ = :: core :: fmt :: DebugTuple :: field (debug_trait_builder , & & (* __self_0)) ; :: core :: fmt :: DebugTuple :: finish (debug_trait_builder) } (& SimpleStorage2Calls :: GetValue (ref __self_0) ,) => { let debug_trait_builder = & mut :: core :: fmt :: Formatter :: debug_tuple (f , "GetValue") ; let _ = :: core :: fmt :: DebugTuple :: field (debug_trait_builder , & & (* __self_0)) ; :: core :: fmt :: DebugTuple :: finish (debug_trait_builder) } (& SimpleStorage2Calls :: LastSender (ref __self_0) ,) => { let debug_trait_builder = & mut :: core :: fmt :: Formatter :: debug_tuple (f , "LastSender") ; let _ = :: core :: fmt :: DebugTuple :: field (debug_trait_builder , & & (* __self_0)) ; :: core :: fmt :: DebugTuple :: finish (debug_trait_builder) } (& SimpleStorage2Calls :: SetValue (ref __self_0) ,) => { let debug_trait_builder = & mut :: core :: fmt :: Formatter :: debug_tuple (f , "SetValue") ; let _ = :: core :: fmt :: DebugTuple :: field (debug_trait_builder , & & (* __self_0)) ; :: core :: fmt :: DebugTuple :: finish (debug_trait_builder) } (& SimpleStorage2Calls :: SetValues (ref __self_0) ,) => { let debug_trait_builder = & mut :: core :: fmt :: Formatter :: debug_tuple (f , "SetValues") ; let _ = :: core :: fmt :: DebugTuple :: field (debug_trait_builder , & & (* __self_0)) ; :: core :: fmt :: DebugTuple :: finish (debug_trait_builder) } } } } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: clone :: Clone for SimpleStorage2Calls { # [inline] fn clone (& self) -> SimpleStorage2Calls { match (& * self ,) { (& SimpleStorage2Calls :: HashPuzzle (ref __self_0) ,) => SimpleStorage2Calls :: HashPuzzle (:: core :: clone :: Clone :: clone (& (* __self_0))) , (& SimpleStorage2Calls :: GetValue (ref __self_0) ,) => SimpleStorage2Calls :: GetValue (:: core :: clone :: Clone :: clone (& (* __self_0))) , (& SimpleStorage2Calls :: LastSender (ref __self_0) ,) => SimpleStorage2Calls :: LastSender (:: core :: clone :: Clone :: clone (& (* __self_0))) , (& SimpleStorage2Calls :: SetValue (ref __self_0) ,) => SimpleStorage2Calls :: SetValue (:: core :: clone :: Clone :: clone (& (* __self_0))) , (& SimpleStorage2Calls :: SetValues (ref __self_0) ,) => SimpleStorage2Calls :: SetValues (:: core :: clone :: Clone :: clone (& (* __self_0))) , } } } impl :: core :: marker :: StructuralPartialEq for SimpleStorage2Calls { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: PartialEq for SimpleStorage2Calls { # [inline] fn eq (& self , other : & SimpleStorage2Calls) -> bool { { let __self_vi = :: core :: intrinsics :: discriminant_value (& * self) ; let __arg_1_vi = :: core :: intrinsics :: discriminant_value (& * other) ; if true && __self_vi == __arg_1_vi { match (& * self , & * other) { (& SimpleStorage2Calls :: HashPuzzle (ref __self_0) , & SimpleStorage2Calls :: HashPuzzle (ref __arg_1_0)) => (* __self_0) == (* __arg_1_0) , (& SimpleStorage2Calls :: GetValue (ref __self_0) , & SimpleStorage2Calls :: GetValue (ref __arg_1_0)) => (* __self_0) == (* __arg_1_0) , (& SimpleStorage2Calls :: LastSender (ref __self_0) , & SimpleStorage2Calls :: LastSender (ref __arg_1_0)) => (* __self_0) == (* __arg_1_0) , (& SimpleStorage2Calls :: SetValue (ref __self_0) , & SimpleStorage2Calls :: SetValue (ref __arg_1_0)) => (* __self_0) == (* __arg_1_0) , (& SimpleStorage2Calls :: SetValues (ref __self_0) , & SimpleStorage2Calls :: SetValues (ref __arg_1_0)) => (* __self_0) == (* __arg_1_0) , _ => unsafe { :: core :: intrinsics :: unreachable () } } } else { false } } } # [inline] fn ne (& self , other : & SimpleStorage2Calls) -> bool { { let __self_vi = :: core :: intrinsics :: discriminant_value (& * self) ; let __arg_1_vi = :: core :: intrinsics :: discriminant_value (& * other) ; if true && __self_vi == __arg_1_vi { match (& * self , & * other) { (& SimpleStorage2Calls :: HashPuzzle (ref __self_0) , & SimpleStorage2Calls :: HashPuzzle (ref __arg_1_0)) => (* __self_0) != (* __arg_1_0) , (& SimpleStorage2Calls :: GetValue (ref __self_0) , & SimpleStorage2Calls :: GetValue (ref __arg_1_0)) => (* __self_0) != (* __arg_1_0) , (& SimpleStorage2Calls :: LastSender (ref __self_0) , & SimpleStorage2Calls :: LastSender (ref __arg_1_0)) => (* __self_0) != (* __arg_1_0) , (& SimpleStorage2Calls :: SetValue (ref __self_0) , & SimpleStorage2Calls :: SetValue (ref __arg_1_0)) => (* __self_0) != (* __arg_1_0) , (& SimpleStorage2Calls :: SetValues (ref __self_0) , & SimpleStorage2Calls :: SetValues (ref __arg_1_0)) => (* __self_0) != (* __arg_1_0) , _ => unsafe { :: core :: intrinsics :: unreachable () } } } else { true } } } } impl :: core :: marker :: StructuralEq for SimpleStorage2Calls { } # [automatically_derived] # [allow (unused_qualifications)] impl :: core :: cmp :: Eq for SimpleStorage2Calls { # [inline] # [doc (hidden)] # [no_coverage] fn assert_receiver_is_total_eq (& self) -> () { { let _ : :: core :: cmp :: AssertParamIsEq < HashPuzzleCall > ; let _ : :: core :: cmp :: AssertParamIsEq < GetValueCall > ; let _ : :: core :: cmp :: AssertParamIsEq < LastSenderCall > ; let _ : :: core :: cmp :: AssertParamIsEq < SetValueCall > ; let _ : :: core :: cmp :: AssertParamIsEq < SetValuesCall > ; } } } impl ethers_core :: abi :: Tokenizable for SimpleStorage2Calls { fn from_token (token : ethers_core :: abi :: Token) -> Result < Self , ethers_core :: abi :: InvalidOutputType > where Self : Sized { if let Ok (decoded) = HashPuzzleCall :: from_token (token . clone ()) { return Ok (SimpleStorage2Calls :: HashPuzzle (decoded)) } if let Ok (decoded) = GetValueCall :: from_token (token . clone ()) { return Ok (SimpleStorage2Calls :: GetValue (decoded)) } if let Ok (decoded) = LastSenderCall :: from_token (token . clone ()) { return Ok (SimpleStorage2Calls :: LastSender (decoded)) } if let Ok (decoded) = SetValueCall :: from_token (token . clone ()) { return Ok (SimpleStorage2Calls :: SetValue (decoded)) } if let Ok (decoded) = SetValuesCall :: from_token (token . clone ()) { return Ok (SimpleStorage2Calls :: SetValues (decoded)) } Err (ethers_core :: abi :: InvalidOutputType ("Failed to decode all type variants" . to_string ())) } fn into_token (self) -> ethers_core :: abi :: Token { match self { SimpleStorage2Calls :: HashPuzzle (element) => element . into_token () , SimpleStorage2Calls :: GetValue (element) => element . into_token () , SimpleStorage2Calls :: LastSender (element) => element . into_token () , SimpleStorage2Calls :: SetValue (element) => element . into_token () , SimpleStorage2Calls :: SetValues (element) => element . into_token () , } } } impl ethers_core :: abi :: TokenizableItem for SimpleStorage2Calls { } impl ethers_core :: abi :: AbiDecode for SimpleStorage2Calls { fn decode (data : impl AsRef < [u8] >) -> Result < Self , ethers_core :: abi :: AbiError > { if let Ok (decoded) = < HashPuzzleCall as ethers_core :: abi :: AbiDecode > :: decode (data . as_ref ()) { return Ok (SimpleStorage2Calls :: HashPuzzle (decoded)) } if let Ok (decoded) = < GetValueCall as ethers_core :: abi :: AbiDecode > :: decode (data . as_ref ()) { return Ok (SimpleStorage2Calls :: GetValue (decoded)) } if let Ok (decoded) = < LastSenderCall as ethers_core :: abi :: AbiDecode > :: decode (data . as_ref ()) { return Ok (SimpleStorage2Calls :: LastSender (decoded)) } if let Ok (decoded) = < SetValueCall as ethers_core :: abi :: AbiDecode > :: decode (data . as_ref ()) { return Ok (SimpleStorage2Calls :: SetValue (decoded)) } if let Ok (decoded) = < SetValuesCall as ethers_core :: abi :: AbiDecode > :: decode (data . as_ref ()) { return Ok (SimpleStorage2Calls :: SetValues (decoded)) } Err (ethers_core :: abi :: Error :: InvalidData . into ()) } } impl ethers_core :: abi :: AbiEncode for SimpleStorage2Calls { fn encode (self) -> Vec < u8 > { match self { SimpleStorage2Calls :: HashPuzzle (element) => element . encode () , SimpleStorage2Calls :: GetValue (element) => element . encode () , SimpleStorage2Calls :: LastSender (element) => element . encode () , SimpleStorage2Calls :: SetValue (element) => element . encode () , SimpleStorage2Calls :: SetValues (element) => element . encode () , } } } impl :: std :: fmt :: Display for SimpleStorage2Calls { fn fmt (& self , f : & mut :: std :: fmt :: Formatter < '_ >) -> :: std :: fmt :: Result { match self { SimpleStorage2Calls :: HashPuzzle (element) => element . fmt (f) , SimpleStorage2Calls :: GetValue (element) => element . fmt (f) , SimpleStorage2Calls :: LastSender (element) => element . fmt (f) , SimpleStorage2Calls :: SetValue (element) => element . fmt (f) , SimpleStorage2Calls :: SetValues (element) => element . fmt (f) , } } } impl :: std :: convert :: From < HashPuzzleCall > for SimpleStorage2Calls { fn from (var : HashPuzzleCall) -> Self { SimpleStorage2Calls :: HashPuzzle (var) } } impl :: std :: convert :: From < GetValueCall > for SimpleStorage2Calls { fn from (var : GetValueCall) -> Self { SimpleStorage2Calls :: GetValue (var) } } impl :: std :: convert :: From < LastSenderCall > for SimpleStorage2Calls { fn from (var : LastSenderCall) -> Self { SimpleStorage2Calls :: LastSender (var) } } impl :: std :: convert :: From < SetValueCall > for SimpleStorage2Calls { fn from (var : SetValueCall) -> Self { SimpleStorage2Calls :: SetValue (var) } } impl :: std :: convert :: From < SetValuesCall > for SimpleStorage2Calls { fn from (var : SetValuesCall) -> Self { SimpleStorage2Calls :: SetValues (var) } } } let ganache = ethers_core :: utils :: Ganache :: new () . spawn () ; let from = ganache . addresses () [0] ; let provider = Provider :: try_from (ganache . endpoint ()) . unwrap () . with_sender (from) . interval (std :: time :: Duration :: from_millis (10)) ; let client = Arc :: new (provider) ; let compiled = Solc :: new ("./tests/solidity-contracts/SimpleStorage.sol") . build () . unwrap () ; let compiled = compiled . get ("SimpleStorage") . unwrap () ; let factory = ethers_contract :: ContractFactory :: new (compiled . abi . clone () , compiled . bytecode . clone () , client . clone ()) ; let addr = factory . deploy ("hi" . to_string ()) . unwrap () . legacy () . send () . await . unwrap () . address () ; let contract = SimpleStorage :: new (addr , client . clone ()) ; let contract2 = SimpleStorage2 :: new (addr , client . clone ()) ; let res = contract . hash_puzzle () . call () . await . unwrap () ; let res2 = contract2 . hash_puzzle () . call () . await . unwrap () ; let res3 = contract . method :: < _ , U256 > ("_hashPuzzle" , ()) . unwrap () . call () . await . unwrap () ; let res4 = contract2 . method :: < _ , U256 > ("_hashPuzzle" , ()) . unwrap () . call () . await . unwrap () ; use ethers_providers :: Middleware ; let data = simplestorage_mod :: HashPuzzleCall . encode () ; let tx = Eip1559TransactionRequest :: new () . data (data) . to (addr) ; let tx = TypedTransaction :: Eip1559 (tx) ; let res5 = client . call (& tx , None) . await . unwrap () ; let res5 = U256 :: from (res5 . as_ref ()) ; { match (& res , & 100 . into ()) { (left_val , right_val) => { if ! (* left_val == * right_val) { let kind = :: core :: panicking :: AssertKind :: Eq ; :: core :: panicking :: assert_failed (kind , & * left_val , & * right_val , :: core :: option :: Option :: None) ; } } } } ; { match (& res , & res2) { (left_val , right_val) => { if ! (* left_val == * right_val) { let kind = :: core :: panicking :: AssertKind :: Eq ; :: core :: panicking :: assert_failed (kind , & * left_val , & * right_val , :: core :: option :: Option :: None) ; } } } } ; { match (& res , & res3) { (left_val , right_val) => { if ! (* left_val == * right_val) { let kind = :: core :: panicking :: AssertKind :: Eq ; :: core :: panicking :: assert_failed (kind , & * left_val , & * right_val , :: core :: option :: Option :: None) ; } } } } ; { match (& res , & res4) { (left_val , right_val) => { if ! (* left_val == * right_val) { let kind = :: core :: panicking :: AssertKind :: Eq ; :: core :: panicking :: assert_failed (kind , & * left_val , & * right_val , :: core :: option :: Option :: None) ; } } } } ; { match (& res , & res5) { (left_val , right_val) => { if ! (* left_val == * right_val) { let kind = :: core :: panicking :: AssertKind :: Eq ; :: core :: panicking :: assert_failed (kind , & * left_val , & * right_val , :: core :: option :: Option :: None) ; } } } } ; };
    #[allow(clippy::expect_used)]
        tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(body);
}
#[rustc_main]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[
        &can_gen_human_readable,
        &can_gen_human_readable_multiple,
        &can_gen_structs_readable,
        &can_gen_structs_with_arrays_readable,
        &can_generate_internal_structs,
        &can_generate_internal_structs_multiple,
        &can_gen_human_readable_with_structs,
        &can_handle_overloaded_functions,
        &can_handle_even_more_overloaded_functions,
        &can_handle_underscore_functions,
    ])
}

Process finished with exit code 0
