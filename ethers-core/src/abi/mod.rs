//! This module implements extensions to the [`ethabi`](https://docs.rs/ethabi) API.
// Adapted from [Gnosis' ethcontract](https://github.com/gnosis/ethcontract-rs/blob/master/common/src/abiext.rs)
use crate::{types::Selector, utils::id};

pub use ethabi::{Contract as Abi, *};

mod tokens;
pub use tokens::{Detokenize, InvalidOutputType, Tokenizable, TokenizableItem, Tokenize};

pub mod struct_def;
pub use struct_def::SolStruct;

mod codec;
pub use codec::{AbiDecode, AbiEncode};

mod error;
pub use error::{AbiError, ParseError};

mod human_readable;
pub use human_readable::{parse as parse_abi, parse_str as parse_abi_str, AbiParser};

use crate::types::{H256, H512, U128, U256, U64};

/// Extension trait for `ethabi::Function`.
pub trait FunctionExt {
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
pub trait EventExt {
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

/// A trait for types that can be represented in the ethereum ABI.
pub trait AbiType {
    /// The native ABI type this type represents.
    fn param_type() -> ParamType;
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
    Vec<u8> => Bytes,
    Address => Address,
    bool => Bool,
    String => String,
    H256 => FixedBytes(32),
    H512 => FixedBytes(64),
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
    i128 => Int(128)
);

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
                $ty: AbiArrayType,
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
        assert_eq!(ParamType::Bytes, Vec::<u8>::param_type());
        assert_eq!(ParamType::Array(Box::new(ParamType::Bytes)), Vec::<Vec<u8>>::param_type());
        assert_eq!(
            ParamType::Array(Box::new(ParamType::Array(Box::new(ParamType::Bytes)))),
            Vec::<Vec<Vec<u8>>>::param_type()
        );

        assert_eq!(ParamType::Array(Box::new(ParamType::Uint(16))), Vec::<u16>::param_type());

        assert_eq!(
            ParamType::Tuple(vec![ParamType::Bytes, ParamType::Address]),
            <(Vec<u8>, Address)>::param_type()
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
    }
}
