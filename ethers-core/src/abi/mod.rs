//! Extensions to the [`ethabi`](https://docs.rs/ethabi) API.
//!
//! Adapted from [Gnosis' `ethcontract-rs`](https://github.com/gnosis/ethcontract-rs).

use crate::{
    types::{self, Selector, Uint8, H256, H512, I256, U128, U256, U64},
    utils::id,
};
pub use ethabi::{self, Contract as Abi, *};

mod tokens;
pub use tokens::{Detokenize, InvalidOutputType, Tokenizable, TokenizableItem, Tokenize};

pub mod struct_def;
pub use struct_def::SolStruct;

mod codec;
pub use codec::{AbiDecode, AbiEncode};

mod error;
pub use error::{AbiError, ParseError};

mod human_readable;
pub use human_readable::{
    lexer::HumanReadableParser, parse as parse_abi, parse_str as parse_abi_str, AbiParser,
};

mod raw;
pub use raw::{AbiObject, Component, Item, JsonAbi, RawAbi};

mod packed;
pub use packed::{encode_packed, EncodePackedError};

mod sealed {
    use ethabi::{Event, Function};

    /// private trait to ensure extension traits are used as intended
    pub trait Sealed {}
    impl Sealed for Function {}
    impl Sealed for Event {}
    impl Sealed for ethabi::AbiError {}
}

/// Extension trait for `ethabi::Function`.
pub trait FunctionExt: sealed::Sealed {
    /// Compute the method signature in the standard ABI format. This does not
    /// include the output types.
    fn abi_signature(&self) -> String;

    /// Compute the Keccak256 function selector used by contract ABIs.
    fn selector(&self) -> Selector;
}

impl FunctionExt for Function {
    fn abi_signature(&self) -> String {
        let mut full_signature = self.signature();
        if let Some(colon) = full_signature.find(':') {
            full_signature.truncate(colon);
        }

        full_signature
    }

    fn selector(&self) -> Selector {
        id(self.abi_signature())
    }
}

/// Extension trait for `ethabi::Event`.
pub trait EventExt: sealed::Sealed {
    /// Compute the event signature in human-readable format. The `keccak256`
    /// hash of this value is the actual event signature that is used as topic0
    /// in the transaction logs.
    fn abi_signature(&self) -> String;
}

impl EventExt for Event {
    fn abi_signature(&self) -> String {
        format!(
            "{}({}){}",
            self.name,
            self.inputs.iter().map(|input| input.kind.to_string()).collect::<Vec<_>>().join(","),
            if self.anonymous { " anonymous" } else { "" },
        )
    }
}

/// Extension trait for `ethabi::AbiError`.
pub trait ErrorExt: sealed::Sealed {
    /// Compute the method signature in the standard ABI format.
    fn abi_signature(&self) -> String;

    /// Compute the Keccak256 error selector used by contract ABIs.
    fn selector(&self) -> Selector;
}

impl ErrorExt for ethabi::AbiError {
    fn abi_signature(&self) -> String {
        if self.inputs.is_empty() {
            return format!("{}()", self.name)
        }
        let inputs = self.inputs.iter().map(|p| p.kind.to_string()).collect::<Vec<_>>().join(",");
        format!("{}({inputs})", self.name)
    }

    fn selector(&self) -> Selector {
        id(self.abi_signature())
    }
}

/// A trait for types that can be represented in the Ethereum ABI.
pub trait AbiType {
    /// The native ABI type this type represents.
    fn param_type() -> ParamType;

    /// A hint of the minimum number of bytes this type takes up in the ABI.
    fn minimum_size() -> usize {
        minimum_size(&Self::param_type())
    }
}

/// Returns the minimum number of bytes that `ty` takes up in the ABI.
pub fn minimum_size(ty: &ParamType) -> usize {
    match ty {
        // 1 word
        ParamType::Uint(_) |
        ParamType::Int(_) |
        ParamType::Bool |
        ParamType::Address |
        ParamType::FixedBytes(_) => 32,
        // min 2 words (offset, length)
        ParamType::Bytes | ParamType::String | ParamType::Array(_) => 64,
        // sum of all elements
        ParamType::FixedArray(ty, len) => minimum_size(ty) * len,
        ParamType::Tuple(tys) => tys.iter().map(minimum_size).sum(),
    }
}

impl AbiType for u8 {
    fn param_type() -> ParamType {
        ParamType::Uint(8)
    }
}

/// Additional trait for types that can appear in arrays
///
/// NOTE: this is necessary to handle the special case of `Vec<u8> => Bytes`
pub trait AbiArrayType: AbiType {}

impl<T: AbiArrayType> AbiType for Vec<T> {
    fn param_type() -> ParamType {
        ParamType::Array(Box::new(T::param_type()))
    }
}
impl<T: AbiArrayType> AbiArrayType for Vec<T> {}

impl<T: AbiArrayType, const N: usize> AbiType for [T; N] {
    fn param_type() -> ParamType {
        ParamType::FixedArray(Box::new(T::param_type()), N)
    }
}

impl<T: AbiArrayType, const N: usize> AbiArrayType for [T; N] {}

impl<const N: usize> AbiType for [u8; N] {
    fn param_type() -> ParamType {
        ParamType::FixedBytes(N)
    }
}
impl<const N: usize> AbiArrayType for [u8; N] {}

macro_rules! impl_abi_type {
    ($($name:ty => $var:ident $(($value:expr))? ),*) => {
        $(
            impl AbiType for $name {
                fn param_type() -> ParamType {
                    ParamType::$var $( ($value) )?
                }
            }

            impl AbiArrayType for $name {}
        )*
    };
}

impl_abi_type!(
    types::Bytes => Bytes,
    bytes::Bytes => Bytes,
    Vec<u8> =>  Array(Box::new(ParamType::Uint(8))),
    Address => Address,
    bool => Bool,
    String => String,
    str => String,
    H256 => FixedBytes(32),
    H512 => FixedBytes(64),
    Uint8 => Uint(8),
    U64 => Uint(64),
    U128 => Uint(128),
    U256 => Uint(256),
    u16 => Uint(16),
    u32 => Uint(32),
    u64 => Uint(64),
    u128 => Uint(128),
    i8 => Int(8),
    i16 => Int(16),
    i32 => Int(32),
    i64 => Int(64),
    i128 => Int(128),
    I256 => Int(256)
);

impl<'a> AbiType for &'a str {
    fn param_type() -> ParamType {
        ParamType::String
    }
}

impl<'a> AbiArrayType for &'a str {}

macro_rules! impl_abi_type_tuple {
    ($num: expr, $( $ty: ident),+) => {
        impl<$($ty, )+> AbiType for ($($ty,)+) where
            $(
                $ty: AbiType,
            )+
        {
            fn param_type() -> ParamType {
                ParamType::Tuple(
                    ::std::vec![
                         $(
                           $ty::param_type(),
                        )+
                    ]
                )
            }
        }

        impl<$($ty, )+> AbiArrayType for ($($ty,)+) where
            $(
                $ty: AbiType,
            )+ {}
    }
}

impl_abi_type_tuple!(1, A);
impl_abi_type_tuple!(2, A, B);
impl_abi_type_tuple!(3, A, B, C);
impl_abi_type_tuple!(4, A, B, C, D);
impl_abi_type_tuple!(5, A, B, C, D, E);
impl_abi_type_tuple!(6, A, B, C, D, E, F);
impl_abi_type_tuple!(7, A, B, C, D, E, F, G);
impl_abi_type_tuple!(8, A, B, C, D, E, F, G, H);
impl_abi_type_tuple!(9, A, B, C, D, E, F, G, H, I);
impl_abi_type_tuple!(10, A, B, C, D, E, F, G, H, I, J);
impl_abi_type_tuple!(11, A, B, C, D, E, F, G, H, I, J, K);
impl_abi_type_tuple!(12, A, B, C, D, E, F, G, H, I, J, K, L);
impl_abi_type_tuple!(13, A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_abi_type_tuple!(14, A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_abi_type_tuple!(15, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_abi_type_tuple!(16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_abi_type_tuple!(17, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_abi_type_tuple!(18, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_abi_type_tuple!(19, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_abi_type_tuple!(20, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_abi_type_tuple!(21, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);

#[allow(clippy::extra_unused_type_parameters)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_function_signature() {
        for (f, expected) in &[
            (
                r#"{"name":"foo","inputs":[],"outputs":[], "stateMutability": "nonpayable"}"#,
                "foo()",
            ),
            (
                r#"{"name":"bar","inputs":[{"name":"a","type":"uint256"},{"name":"b","type":"bool"}],"outputs":[], "stateMutability": "nonpayable"}"#,
                "bar(uint256,bool)",
            ),
            (
                r#"{"name":"baz","inputs":[{"name":"a","type":"uint256"}],"outputs":[{"name":"b","type":"bool"}], "stateMutability": "nonpayable"}"#,
                "baz(uint256)",
            ),
            (
                r#"{"name":"bax","inputs":[],"outputs":[{"name":"a","type":"uint256"},{"name":"b","type":"bool"}], "stateMutability": "nonpayable"}"#,
                "bax()",
            ),
        ] {
            let function: Function = serde_json::from_str(f).expect("invalid function JSON");
            let signature = function.abi_signature();
            assert_eq!(signature, *expected);
        }
    }

    #[test]
    fn format_event_signature() {
        for (e, expected) in &[
            (r#"{"name":"foo","inputs":[],"anonymous":false}"#, "foo()"),
            (
                r#"{"name":"bar","inputs":[{"name":"a","type":"uint256"},{"name":"b","type":"bool"}],"anonymous":false}"#,
                "bar(uint256,bool)",
            ),
            (
                r#"{"name":"baz","inputs":[{"name":"a","type":"uint256"}],"anonymous":true}"#,
                "baz(uint256) anonymous",
            ),
            (r#"{"name":"bax","inputs":[],"anonymous":true}"#, "bax() anonymous"),
        ] {
            let event: Event = serde_json::from_str(e).expect("invalid event JSON");
            let signature = event.abi_signature();
            assert_eq!(signature, *expected);
        }
    }

    #[test]
    fn abi_type_works() {
        assert_eq!(ParamType::Bytes, types::Bytes::param_type());
        assert_eq!(ParamType::Array(Box::new(ParamType::Uint(8))), Vec::<u8>::param_type());
        assert_eq!(ParamType::Array(Box::new(ParamType::Bytes)), Vec::<types::Bytes>::param_type());
        assert_eq!(
            ParamType::Array(Box::new(ParamType::Array(Box::new(ParamType::Uint(8))))),
            Vec::<Vec<u8>>::param_type()
        );
        assert_eq!(
            ParamType::Array(Box::new(ParamType::Array(Box::new(ParamType::Array(Box::new(
                ParamType::Uint(8)
            )))))),
            Vec::<Vec<Vec<u8>>>::param_type()
        );

        assert_eq!(ParamType::Array(Box::new(ParamType::Uint(16))), Vec::<u16>::param_type());

        assert_eq!(
            ParamType::Tuple(vec![ParamType::Bytes, ParamType::Address]),
            <(types::Bytes, Address)>::param_type()
        );

        assert_eq!(ParamType::FixedBytes(32), <[u8; 32]>::param_type());
        assert_eq!(
            ParamType::Array(Box::new(ParamType::FixedBytes(32))),
            Vec::<[u8; 32]>::param_type()
        );

        assert_eq!(
            ParamType::FixedArray(Box::new(ParamType::Uint(16)), 32),
            <[u16; 32]>::param_type()
        );

        assert_eq!(ParamType::String, str::param_type());
        assert_eq!(ParamType::String, <&str>::param_type());
    }

    #[test]
    fn abi_type_tuples_work() {
        fn assert_abitype<T: AbiType>() {}
        fn assert_abiarraytype<T: AbiArrayType>() {}

        assert_abitype::<(u64, u64)>();
        assert_abiarraytype::<(u64, u64)>();

        assert_abitype::<(u8, u8)>();
        assert_abiarraytype::<(u8, u8)>();

        assert_abitype::<Vec<(u64, u64)>>();
        assert_abiarraytype::<Vec<(u64, u64)>>();

        assert_abitype::<Vec<(u8, u8)>>();
        assert_abiarraytype::<Vec<(u8, u8)>>();
    }
}
