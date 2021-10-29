use crate::{
    abi::{AbiArrayType, AbiError, AbiType, Detokenize, Tokenizable, TokenizableItem},
    types::{Address, H256, U128, U256},
};

/// Trait for ABI encoding
pub trait AbiEncode {
    /// ABI encode the type
    fn encode(self) -> Vec<u8>;
}

/// Trait for ABI decoding
pub trait AbiDecode: Sized {
    /// Decodes the ABI encoded data
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError>;
}

macro_rules! impl_abi_codec {
    ($($name:ty),*) => {
        $(
            impl AbiEncode for $name {
                fn encode(self) -> Vec<u8> {
                    let token = self.into_token();
                    crate::abi::encode(&[token]).into()
                }
            }
            impl AbiDecode for $name {
                fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError> {
                    let tokens = crate::abi::decode(
                        &[Self::param_type()], bytes.as_ref()
                    )?;
                    Ok(<Self as Detokenize>::from_tokens(tokens)?)
                }
            }
        )*
    };
}

impl_abi_codec!(
    Vec<u8>,
    Address,
    bool,
    String,
    H256,
    U128,
    U256,
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128
);

impl<T: TokenizableItem + Clone, const N: usize> AbiEncode for [T; N] {
    fn encode(self) -> Vec<u8> {
        let token = self.into_token();
        crate::abi::encode(&[token])
    }
}

impl<const N: usize> AbiEncode for [u8; N] {
    fn encode(self) -> Vec<u8> {
        let token = self.into_token();
        crate::abi::encode(&[token])
    }
}

impl<const N: usize> AbiDecode for [u8; N] {
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError> {
        let tokens = crate::abi::decode(&[Self::param_type()], bytes.as_ref())?;
        Ok(<Self as Detokenize>::from_tokens(tokens)?)
    }
}

impl<T, const N: usize> AbiDecode for [T; N]
where
    T: TokenizableItem + AbiArrayType + Clone,
{
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError> {
        let tokens = crate::abi::decode(&[Self::param_type()], bytes.as_ref())?;
        Ok(<Self as Detokenize>::from_tokens(tokens)?)
    }
}

impl<T: TokenizableItem + AbiArrayType> AbiEncode for Vec<T> {
    fn encode(self) -> Vec<u8> {
        let token = self.into_token();
        crate::abi::encode(&[token])
    }
}

impl<T: TokenizableItem + AbiArrayType> AbiDecode for Vec<T> {
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError> {
        let tokens = crate::abi::decode(&[Self::param_type()], bytes.as_ref())?;
        Ok(<Self as Detokenize>::from_tokens(tokens)?)
    }
}

macro_rules! impl_abi_codec_tuple {
    ($num: expr, $( $ty: ident),+) => {
        impl<$($ty, )+> AbiEncode for ($($ty,)+) where
            $(
                $ty: Tokenizable,
            )+
        {
            fn encode(self) -> Vec<u8> {
                let token = self.into_token();
                crate::abi::encode(&[token]).into()
            }
        }

        impl<$($ty, )+> AbiDecode for ($($ty,)+) where
            $(
                $ty: AbiType +  Tokenizable,
            )+ {
                fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError> {
                    let tokens = crate::abi::decode(
                    &[Self::param_type()], bytes.as_ref()
                    )?;
                    Ok(<Self as Detokenize>::from_tokens(tokens)?)
                }
        }
    }
}

impl_abi_codec_tuple!(1, A);
impl_abi_codec_tuple!(2, A, B);
impl_abi_codec_tuple!(3, A, B, C);
impl_abi_codec_tuple!(4, A, B, C, D);
impl_abi_codec_tuple!(5, A, B, C, D, E);
impl_abi_codec_tuple!(6, A, B, C, D, E, F);
impl_abi_codec_tuple!(7, A, B, C, D, E, F, G);
impl_abi_codec_tuple!(8, A, B, C, D, E, F, G, H);
impl_abi_codec_tuple!(9, A, B, C, D, E, F, G, H, I);
impl_abi_codec_tuple!(10, A, B, C, D, E, F, G, H, I, J);
impl_abi_codec_tuple!(11, A, B, C, D, E, F, G, H, I, J, K);
impl_abi_codec_tuple!(12, A, B, C, D, E, F, G, H, I, J, K, L);
impl_abi_codec_tuple!(13, A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_abi_codec_tuple!(14, A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_abi_codec_tuple!(15, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_abi_codec_tuple!(16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    use crate::abi::{AbiArrayType, TokenizableItem};
    use rand::{
        distributions::{Alphanumeric, Distribution, Standard},
        random, thread_rng, Rng,
    };

    fn assert_codec<T>(val: T)
    where
        T: AbiDecode + AbiEncode + Clone + PartialEq + Debug,
    {
        let encoded = val.clone().encode();
        assert_eq!(val, T::decode(encoded).unwrap());
    }

    macro_rules! roundtrip_alloc {
        ($val:expr) => {
            assert_codec($val);
            assert_codec(($val, $val));
            assert_codec(std::iter::repeat_with(|| $val).take(10).collect::<Vec<_>>());
        };
    }

    macro_rules! roundtrip_all {
        ($val:expr) => {
            roundtrip_alloc!($val);
            assert_codec([$val; 10]);
        };
    }

    fn test_codec_rng<T>()
    where
        Standard: Distribution<T>,
        T: AbiDecode + AbiEncode + Copy + PartialEq + Debug + AbiArrayType + TokenizableItem,
    {
        roundtrip_all!(random::<T>());
    }

    #[test]
    fn address_codec() {
        test_codec_rng::<Address>();
    }

    #[test]
    fn uint_codec() {
        test_codec_rng::<u16>();
        test_codec_rng::<u32>();
        test_codec_rng::<u64>();
        test_codec_rng::<u128>();

        test_codec_rng::<i8>();
        test_codec_rng::<i16>();
        test_codec_rng::<i32>();
        test_codec_rng::<i64>();
        test_codec_rng::<i128>();
    }

    #[test]
    fn u8_codec() {
        assert_codec(random::<u8>());
        assert_codec((random::<u8>(), random::<u8>()));
        assert_codec(std::iter::repeat_with(random::<u8>).take(10).collect::<Vec<_>>());
        assert_codec([random::<u8>(); 10]);
    }

    #[test]
    fn string_codec() {
        roundtrip_alloc! { thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect::<String>()
        };
    }
}
