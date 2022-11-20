use crate::{
    abi::{
        AbiArrayType, AbiError, AbiType, Detokenize, Token, Tokenizable, TokenizableItem, Tokenize,
    },
    types::{Address, Bytes, Uint8, H256, I256, U128, U256},
};

/// Trait for ABI encoding
pub trait AbiEncode {
    /// ABI encode the type
    fn encode(self) -> Vec<u8>;

    /// Returns the encoded value as hex string, _with_ a `0x` prefix
    fn encode_hex(self) -> String
    where
        Self: Sized,
    {
        format!("0x{}", hex::encode(self.encode()))
    }
}

/// Trait for ABI decoding
pub trait AbiDecode: Sized {
    /// Decodes the ABI encoded data
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError>;

    /// Decode hex encoded ABI encoded data
    ///
    /// Expects a hex encoded string, with optional `0x` prefix
    fn decode_hex(data: impl AsRef<str>) -> Result<Self, AbiError> {
        let bytes: Bytes = data.as_ref().parse()?;
        Self::decode(bytes)
    }
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
    Bytes,
    bytes::Bytes,
    Address,
    bool,
    String,
    H256,
    U128,
    U256,
    I256,
    Uint8,
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

impl<'a> AbiEncode for &'a str {
    fn encode(self) -> Vec<u8> {
        self.to_string().encode()
    }
}

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
                crate::abi::encode(&self.into_tokens()).into()
            }
        }

        impl<$($ty, )+> AbiDecode for ($($ty,)+) where
            $(
                $ty: AbiType +  Tokenizable,
            )+ {
                fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError> {
                    if let crate::abi::ParamType::Tuple(params) = <Self as AbiType>::param_type() {
                      let tokens = crate::abi::decode(&params, bytes.as_ref())?;
                      Ok(<Self as Tokenizable>::from_token(Token::Tuple(tokens))?)
                    } else {
                        Err(
                            crate::abi::InvalidOutputType("Expected tuple".to_string()).into()
                        )
                    }
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
impl_abi_codec_tuple!(16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_abi_codec_tuple!(17, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_abi_codec_tuple!(18, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_abi_codec_tuple!(19, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_abi_codec_tuple!(20, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);

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

    #[test]
    fn bytes_codec() {
        let bytes: Bytes = std::iter::repeat_with(random::<u8>).take(10).collect::<Vec<_>>().into();
        let v = vec![bytes];
        assert_codec(v);
    }

    #[test]
    fn tuple_array() {
        let nested: Vec<[u8; 4]> = vec![[0, 0, 0, 1]];
        assert_codec(nested.clone());
        let tuple: Vec<(Address, u8, Vec<[u8; 4]>)> = vec![(Address::random(), 0, nested)];
        assert_codec(tuple);
    }

    #[test]
    fn str_encoding() {
        let value = "str value";
        let encoded = value.encode();
        assert_eq!(value, String::decode(encoded).unwrap());
    }

    #[test]
    fn should_decode_array_of_fixed_uint8() {
        // uint8[8]
        let tokens = vec![Token::FixedArray(vec![
            Token::Uint(1.into()),
            Token::Uint(2.into()),
            Token::Uint(3.into()),
            Token::Uint(4.into()),
            Token::Uint(5.into()),
            Token::Uint(6.into()),
            Token::Uint(7.into()),
            Token::Uint(8.into()),
        ])];
        let data: [Uint8; 8] = Detokenize::from_tokens(tokens).unwrap();
        assert_eq!(data[0], 1);
        assert_eq!(data[1], 2);
        assert_eq!(data[2], 3);
        assert_eq!(data[7], 8);
    }
}
