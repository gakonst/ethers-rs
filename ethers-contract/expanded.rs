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
};
use ethers_providers::Provider;
use ethers_solc::Solc;
use std::{convert::TryFrom, sync::Arc};
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const can_generate_nested_types: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("can_generate_nested_types"),
        ignore: false,
        allow_fail: false,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(can_generate_nested_types())),
};
fn can_generate_nested_types() {
    pub use test_mod::*;
    #[allow(clippy::too_many_arguments)]
    mod test_mod {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
        #![allow(clippy::type_complexity)]
        #![allow(unused_imports)]
        ///Test was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
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
        pub static TEST_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
            ethers_contract::Lazy::new(|| {
                ethers_core :: abi :: parse_abi_str ("[\n        struct Outer {Inner inner; uint256[] arr;}\n        struct Inner {uint256 inner;}\n        function myfun(Outer calldata a)\n    ]") . expect ("invalid abi")
            });
        pub struct Test<M>(ethers_contract::Contract<M>);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<M: ::core::clone::Clone> ::core::clone::Clone for Test<M> {
            #[inline]
            fn clone(&self) -> Test<M> {
                match *self {
                    Test(ref __self_0_0) => Test(::core::clone::Clone::clone(&(*__self_0_0))),
                }
            }
        }
        impl<M> std::ops::Deref for Test<M> {
            type Target = ethers_contract::Contract<M>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<M: ethers_providers::Middleware> std::fmt::Debug for Test<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple("Test").field(&self.address()).finish()
            }
        }
        impl<'a, M: ethers_providers::Middleware> Test<M> {
            /// Creates a new contract instance with the specified `ethers`
            /// client at the given `Address`. The contract derefs to a `ethers::Contract`
            /// object
            pub fn new<T: Into<ethers_core::types::Address>>(
                address: T,
                client: ::std::sync::Arc<M>,
            ) -> Self {
                let contract =
                    ethers_contract::Contract::new(address.into(), TEST_ABI.clone(), client);
                Self(contract)
            }
            ///Calls the contract's `myfun` (0x6f049945) function
            pub fn myfun(&self, a: Outer) -> ethers_contract::builders::ContractCall<M, ()> {
                self.0
                    .method_hash([111, 4, 153, 69], (a,))
                    .expect("method not found (this should never happen)")
            }
        }
        ///Container type for all input parameters for the `myfun`function with signature `myfun(((uint256),uint256[]))` and selector `[111, 4, 153, 69]`
        #[ethcall(name = "myfun", abi = "myfun(((uint256),uint256[]))")]
        pub struct MyfunCall {
            pub a: Outer,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for MyfunCall {
            #[inline]
            fn clone(&self) -> MyfunCall {
                match *self {
                    MyfunCall { a: ref __self_0_0 } => MyfunCall {
                        a: ::core::clone::Clone::clone(&(*__self_0_0)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for MyfunCall {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    MyfunCall { a: ref __self_0_0 } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "MyfunCall");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "a",
                            &&(*__self_0_0),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for MyfunCall {
            #[inline]
            fn default() -> MyfunCall {
                MyfunCall {
                    a: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for MyfunCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for MyfunCall {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<Outer>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for MyfunCall {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for MyfunCall {
            #[inline]
            fn eq(&self, other: &MyfunCall) -> bool {
                match *other {
                    MyfunCall { a: ref __self_1_0 } => match *self {
                        MyfunCall { a: ref __self_0_0 } => (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &MyfunCall) -> bool {
                match *other {
                    MyfunCall { a: ref __self_1_0 } => match *self {
                        MyfunCall { a: ref __self_0_0 } => (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for MyfunCall
        where
            Outer: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
            where
                Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(mut tokens) = token {
                    if tokens.len() != 1usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&1usize, &tokens.len(), &tokens) {
                                    _args => [
                                        ::core::fmt::ArgumentV1::new(
                                            _args.0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            _args.1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            _args.2,
                                            ::core::fmt::Debug::fmt,
                                        ),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    Ok(Self {
                        a: ethers_core::abi::Tokenizable::from_token(tokens.remove(0))?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                _args => [::core::fmt::ArgumentV1::new(
                                    _args.0,
                                    ::core::fmt::Debug::fmt,
                                )],
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [self.a.into_token()]))
            }
        }
        impl ethers_core::abi::TokenizableItem for MyfunCall where Outer: ethers_core::abi::Tokenize {}
        impl ethers_contract::EthCall for MyfunCall {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                "myfun".into()
            }
            fn selector() -> ethers_core::types::Selector {
                ethers_core::utils::id(Self::abi_signature())
            }
            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Owned({
                    let params: String = [<Outer as ethers_core::abi::AbiType>::param_type()]
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<::std::vec::Vec<_>>()
                        .join(",");
                    let function_name = "myfun";
                    {
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["", "(", ")"],
                            &match (&function_name, &params) {
                                _args => [
                                    ::core::fmt::ArgumentV1::new(
                                        _args.0,
                                        ::core::fmt::Display::fmt,
                                    ),
                                    ::core::fmt::ArgumentV1::new(
                                        _args.1,
                                        ::core::fmt::Display::fmt,
                                    ),
                                ],
                            },
                        ));
                        res
                    }
                })
            }
        }
        impl ethers_core::abi::AbiDecode for MyfunCall {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
                let bytes = bytes.as_ref();
                if bytes.len() < 4 || bytes[..4] != <Self as ethers_contract::EthCall>::selector() {
                    return Err(ethers_contract::AbiError::WrongSelector);
                }
                let data_types = [<Outer as ethers_core::abi::AbiType>::param_type()];
                let data_tokens = ethers_core::abi::decode(&data_types, &bytes[4..])?;
                Ok(<Self as ethers_core::abi::Tokenizable>::from_token(
                    ethers_core::abi::Token::Tuple(data_tokens),
                )?)
            }
        }
        impl ethers_core::abi::AbiEncode for MyfunCall {
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
        impl ::std::fmt::Display for MyfunCall {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&&self.a,) {
                        _args => [::core::fmt::ArgumentV1::new(
                            _args.0,
                            ::core::fmt::Debug::fmt,
                        )],
                    },
                ))?;
                Ok(())
            }
        }
        ///`Inner(uint256)`
        pub struct Inner {
            pub inner: ethers_core::types::U256,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Inner {
            #[inline]
            fn clone(&self) -> Inner {
                match *self {
                    Inner {
                        inner: ref __self_0_0,
                    } => Inner {
                        inner: ::core::clone::Clone::clone(&(*__self_0_0)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Inner {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    Inner {
                        inner: ref __self_0_0,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "Inner");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "inner",
                            &&(*__self_0_0),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for Inner {
            #[inline]
            fn default() -> Inner {
                Inner {
                    inner: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for Inner {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for Inner {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<ethers_core::types::U256>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for Inner {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Inner {
            #[inline]
            fn eq(&self, other: &Inner) -> bool {
                match *other {
                    Inner {
                        inner: ref __self_1_0,
                    } => match *self {
                        Inner {
                            inner: ref __self_0_0,
                        } => (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &Inner) -> bool {
                match *other {
                    Inner {
                        inner: ref __self_1_0,
                    } => match *self {
                        Inner {
                            inner: ref __self_0_0,
                        } => (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for Inner
        where
            ethers_core::types::U256: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
            where
                Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(mut tokens) = token {
                    if tokens.len() != 1usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&1usize, &tokens.len(), &tokens) {
                                    _args => [
                                        ::core::fmt::ArgumentV1::new(
                                            _args.0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            _args.1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            _args.2,
                                            ::core::fmt::Debug::fmt,
                                        ),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    Ok(Self {
                        inner: ethers_core::abi::Tokenizable::from_token(tokens.remove(0))?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                _args => [::core::fmt::ArgumentV1::new(
                                    _args.0,
                                    ::core::fmt::Debug::fmt,
                                )],
                            },
                        ));
                        res
                    }))
                }
            }
            fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(<[_]>::into_vec(box [self.inner.into_token()]))
            }
        }
        impl ethers_core::abi::TokenizableItem for Inner where
            ethers_core::types::U256: ethers_core::abi::Tokenize
        {
        }
        ///`Outer((uint256),uint256[])`
        pub struct Outer {
            pub inner: Inner,
            pub arr: Vec<ethers_core::types::U256>,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Outer {
            #[inline]
            fn clone(&self) -> Outer {
                match *self {
                    Outer {
                        inner: ref __self_0_0,
                        arr: ref __self_0_1,
                    } => Outer {
                        inner: ::core::clone::Clone::clone(&(*__self_0_0)),
                        arr: ::core::clone::Clone::clone(&(*__self_0_1)),
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Outer {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    Outer {
                        inner: ref __self_0_0,
                        arr: ref __self_0_1,
                    } => {
                        let debug_trait_builder =
                            &mut ::core::fmt::Formatter::debug_struct(f, "Outer");
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "inner",
                            &&(*__self_0_0),
                        );
                        let _ = ::core::fmt::DebugStruct::field(
                            debug_trait_builder,
                            "arr",
                            &&(*__self_0_1),
                        );
                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for Outer {
            #[inline]
            fn default() -> Outer {
                Outer {
                    inner: ::core::default::Default::default(),
                    arr: ::core::default::Default::default(),
                }
            }
        }
        impl ::core::marker::StructuralEq for Outer {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for Outer {
            #[inline]
            #[doc(hidden)]
            #[no_coverage]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<Inner>;
                    let _: ::core::cmp::AssertParamIsEq<Vec<ethers_core::types::U256>>;
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for Outer {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Outer {
            #[inline]
            fn eq(&self, other: &Outer) -> bool {
                match *other {
                    Outer {
                        inner: ref __self_1_0,
                        arr: ref __self_1_1,
                    } => match *self {
                        Outer {
                            inner: ref __self_0_0,
                            arr: ref __self_0_1,
                        } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &Outer) -> bool {
                match *other {
                    Outer {
                        inner: ref __self_1_0,
                        arr: ref __self_1_1,
                    } => match *self {
                        Outer {
                            inner: ref __self_0_0,
                            arr: ref __self_0_1,
                        } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                    },
                }
            }
        }
        impl ethers_core::abi::Tokenizable for Outer
        where
            Inner: ethers_core::abi::Tokenize,
            Vec<ethers_core::types::U256>: ethers_core::abi::Tokenize,
        {
            fn from_token(
                token: ethers_core::abi::Token,
            ) -> Result<Self, ethers_core::abi::InvalidOutputType>
            where
                Self: Sized,
            {
                if let ethers_core::abi::Token::Tuple(mut tokens) = token {
                    if tokens.len() != 2usize {
                        return Err(ethers_core::abi::InvalidOutputType({
                            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                                &["Expected ", " tokens, got ", ": "],
                                &match (&2usize, &tokens.len(), &tokens) {
                                    _args => [
                                        ::core::fmt::ArgumentV1::new(
                                            _args.0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            _args.1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            _args.2,
                                            ::core::fmt::Debug::fmt,
                                        ),
                                    ],
                                },
                            ));
                            res
                        }));
                    }
                    Ok(Self {
                        inner: ethers_core::abi::Tokenizable::from_token(tokens.remove(0))?,
                        arr: ethers_core::abi::Tokenizable::from_token(tokens.remove(0))?,
                    })
                } else {
                    Err(ethers_core::abi::InvalidOutputType({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["Expected Tuple, got "],
                            &match (&token,) {
                                _args => [::core::fmt::ArgumentV1::new(
                                    _args.0,
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
                    self.inner.into_token(),
                    self.arr.into_token(),
                ]))
            }
        }
        impl ethers_core::abi::TokenizableItem for Outer
        where
            Inner: ethers_core::abi::Tokenize,
            Vec<ethers_core::types::U256>: ethers_core::abi::Tokenize,
        {
        }
    }
}
#[rustc_main]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[&can_generate_nested_types])
}
