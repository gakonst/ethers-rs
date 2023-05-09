//! Contract Functions Output types.
//!
//! Adapted from [rust-web3](https://github.com/tomusdrw/rust-web3/blob/master/src/contract/tokens.rs).

use crate::{
    abi::Token,
    types::{Address, Bytes, H256, I256, U128, U256},
};
use arrayvec::ArrayVec;
use thiserror::Error;

#[derive(Clone, Debug, Error)]
#[error("{0}")]
pub struct InvalidOutputType(pub String);

/// Output type possible to deserialize from Contract ABI
pub trait Detokenize {
    /// Creates a new instance from parsed ABI tokens.
    fn from_tokens(tokens: Vec<Token>) -> Result<Self, InvalidOutputType>
    where
        Self: Sized;
}

impl Detokenize for () {
    fn from_tokens(_: Vec<Token>) -> std::result::Result<Self, InvalidOutputType> {
        Ok(())
    }
}

impl<T: Tokenizable> Detokenize for T {
    fn from_tokens(mut tokens: Vec<Token>) -> Result<Self, InvalidOutputType> {
        let token = if tokens.len() == 1 { tokens.pop().unwrap() } else { Token::Tuple(tokens) };
        Self::from_token(token)
    }
}

/// Convert types into [`Token`]s.
pub trait Tokenize {
    /// Converts `self` into a `Vec<Token>`.
    fn into_tokens(self) -> Vec<Token>;
}

impl<'a> Tokenize for &'a [Token] {
    fn into_tokens(self) -> Vec<Token> {
        let mut tokens = self.to_vec();
        if tokens.len() == 1 {
            flatten_token(tokens.pop().unwrap())
        } else {
            tokens
        }
    }
}

impl<T: Tokenizable> Tokenize for T {
    fn into_tokens(self) -> Vec<Token> {
        flatten_token(self.into_token())
    }
}

impl Tokenize for () {
    fn into_tokens(self) -> Vec<Token> {
        vec![]
    }
}

/// Simplified output type for single value.
pub trait Tokenizable {
    /// Converts a `Token` into expected type.
    fn from_token(token: Token) -> Result<Self, InvalidOutputType>
    where
        Self: Sized;

    /// Converts a specified type back into token.
    fn into_token(self) -> Token;
}

macro_rules! impl_tuples {
    ($num:expr, $( $ty:ident : $no:tt ),+ $(,)?) => {
        impl<$( $ty ),+> Tokenizable for ($( $ty, )+)
        where
            $(
                $ty: Tokenizable,
            )+
        {
            fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
                match token {
                    Token::Tuple(tokens) if tokens.len() == $num => {
                        let mut it = tokens.into_iter();
                        // SAFETY: length checked above
                        unsafe {
                            Ok(($(
                                <$ty as Tokenizable>::from_token(it.next().unwrap_unchecked())?,
                            )+))
                        }
                    },
                    other => Err(InvalidOutputType(format!(
                        concat!(
                            "Expected `Tuple` of length ",
                            stringify!($num),
                            ", got {:?}",
                        ),
                        other,
                    ))),
                }
            }

            fn into_token(self) -> Token {
                Token::Tuple(vec![
                    $( self.$no.into_token(), )+
                ])
            }
        }
    }
}

impl_tuples!(1, A:0, );
impl_tuples!(2, A:0, B:1, );
impl_tuples!(3, A:0, B:1, C:2, );
impl_tuples!(4, A:0, B:1, C:2, D:3, );
impl_tuples!(5, A:0, B:1, C:2, D:3, E:4, );
impl_tuples!(6, A:0, B:1, C:2, D:3, E:4, F:5, );
impl_tuples!(7, A:0, B:1, C:2, D:3, E:4, F:5, G:6, );
impl_tuples!(8, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, );
impl_tuples!(9, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, );
impl_tuples!(10, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, );
impl_tuples!(11, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, );
impl_tuples!(12, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, );
impl_tuples!(13, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, );
impl_tuples!(14, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, );
impl_tuples!(15, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, );
impl_tuples!(16, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, );
impl_tuples!(17, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16,);
impl_tuples!(18, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17,);
impl_tuples!(19, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18,);
impl_tuples!(20, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18, T:19,);
impl_tuples!(21, A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18, T:19, U:20,);

impl Tokenizable for Token {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        Ok(token)
    }

    fn into_token(self) -> Token {
        self
    }
}

impl Tokenizable for String {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::String(s) => Ok(s),
            other => Err(InvalidOutputType(format!("Expected `String`, got {other:?}"))),
        }
    }

    fn into_token(self) -> Token {
        Token::String(self)
    }
}

impl Tokenizable for Bytes {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::Bytes(s) => Ok(s.into()),
            other => Err(InvalidOutputType(format!("Expected `Bytes`, got {other:?}"))),
        }
    }

    fn into_token(self) -> Token {
        Token::Bytes(self.to_vec())
    }
}

impl Tokenizable for bytes::Bytes {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::Bytes(s) => Ok(s.into()),
            other => Err(InvalidOutputType(format!("Expected `Bytes`, got {other:?}"))),
        }
    }

    fn into_token(self) -> Token {
        Token::Bytes(self.to_vec())
    }
}

impl Tokenizable for H256 {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::FixedBytes(mut s) => {
                if s.len() != 32 {
                    return Err(InvalidOutputType(format!("Expected `H256`, got {s:?}")))
                }
                let mut data = [0; 32];
                for (idx, val) in s.drain(..).enumerate() {
                    data[idx] = val;
                }
                Ok(data.into())
            }
            other => Err(InvalidOutputType(format!("Expected `H256`, got {other:?}"))),
        }
    }

    fn into_token(self) -> Token {
        Token::FixedBytes(self.as_ref().to_vec())
    }
}

impl Tokenizable for Address {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::Address(data) => Ok(data),
            other => Err(InvalidOutputType(format!("Expected `Address`, got {other:?}"))),
        }
    }

    fn into_token(self) -> Token {
        Token::Address(self)
    }
}

impl Tokenizable for bool {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::Bool(data) => Ok(data),
            other => Err(InvalidOutputType(format!("Expected `bool`, got {other:?}"))),
        }
    }
    fn into_token(self) -> Token {
        Token::Bool(self)
    }
}

macro_rules! eth_uint_tokenizable {
    ($uint: ident, $name: expr) => {
        impl Tokenizable for $uint {
            fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
                match token {
                    Token::Int(data) | Token::Uint(data) => {
                        Ok(::std::convert::TryInto::try_into(data).unwrap())
                    }
                    other => {
                        Err(InvalidOutputType(format!("Expected `{}`, got {:?}", $name, other))
                            .into())
                    }
                }
            }

            fn into_token(self) -> Token {
                Token::Uint(self.into())
            }
        }
    };
}

eth_uint_tokenizable!(U256, "U256");
eth_uint_tokenizable!(U128, "U128");

macro_rules! int_tokenizable {
    ($int: ident, $token: ident) => {
        impl Tokenizable for $int {
            fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
                match token {
                    Token::Int(data) | Token::Uint(data) => Ok(data.low_u128() as _),
                    other => Err(InvalidOutputType(format!(
                        "Expected `{}`, got {:?}",
                        stringify!($int),
                        other
                    ))),
                }
            }

            fn into_token(self) -> Token {
                // this should get optimized away by the compiler for unsigned integers
                #[allow(unused_comparisons)]
                let data = if self < 0 {
                    // NOTE: Rust does sign extension when converting from a
                    // signed integer to an unsigned integer, so:
                    // `-1u8 as u128 == u128::max_value()`
                    U256::from(self as u128) | U256([0, 0, u64::max_value(), u64::max_value()])
                } else {
                    self.into()
                };
                Token::$token(data)
            }
        }
    };
}

int_tokenizable!(i8, Int);
int_tokenizable!(i16, Int);
int_tokenizable!(i32, Int);
int_tokenizable!(i64, Int);
int_tokenizable!(i128, Int);
int_tokenizable!(u8, Uint);
int_tokenizable!(u16, Uint);
int_tokenizable!(u32, Uint);
int_tokenizable!(u64, Uint);
int_tokenizable!(u128, Uint);

impl Tokenizable for Vec<u8> {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::Bytes(data) => Ok(data),
            Token::Array(data) => data.into_iter().map(u8::from_token).collect(),
            Token::FixedBytes(data) => Ok(data),
            other => Err(InvalidOutputType(format!("Expected `bytes`, got {other:?}"))),
        }
    }

    fn into_token(self) -> Token {
        Token::Array(self.into_iter().map(Tokenizable::into_token).collect())
    }
}

impl<T: TokenizableItem> Tokenizable for Vec<T> {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::FixedArray(tokens) | Token::Array(tokens) => {
                tokens.into_iter().map(Tokenizable::from_token).collect()
            }
            other => Err(InvalidOutputType(format!("Expected `Array`, got {other:?}"))),
        }
    }

    fn into_token(self) -> Token {
        Token::Array(self.into_iter().map(Tokenizable::into_token).collect())
    }
}

impl<const N: usize> Tokenizable for [u8; N] {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::FixedBytes(bytes) => {
                if bytes.len() != N {
                    return Err(InvalidOutputType(format!(
                        "Expected `FixedBytes({})`, got FixedBytes({})",
                        N,
                        bytes.len()
                    )))
                }

                let mut arr = [0; N];
                arr.copy_from_slice(&bytes);
                Ok(arr)
            }
            other => Err(InvalidOutputType(format!("Expected `FixedBytes({N})`, got {other:?}"))),
        }
    }

    fn into_token(self) -> Token {
        Token::FixedBytes(self.to_vec())
    }
}

impl<T: TokenizableItem + Clone, const N: usize> Tokenizable for [T; N] {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::FixedArray(tokens) => {
                if tokens.len() != N {
                    return Err(InvalidOutputType(format!(
                        "Expected `FixedArray({})`, got FixedArray({})",
                        N,
                        tokens.len()
                    )))
                }

                let mut arr = ArrayVec::<T, N>::new();
                let mut it = tokens.into_iter().map(T::from_token);
                for _ in 0..N {
                    arr.push(it.next().expect("Length validated in guard; qed")?);
                }
                // Can't use expect here because [T; N]: Debug is not satisfied.
                match arr.into_inner() {
                    Ok(arr) => Ok(arr),
                    Err(_) => panic!("All elements inserted so the array is full; qed"),
                }
            }
            other => Err(InvalidOutputType(format!("Expected `FixedArray({N})`, got {other:?}"))),
        }
    }

    fn into_token(self) -> Token {
        Token::FixedArray(ArrayVec::from(self).into_iter().map(T::into_token).collect())
    }
}

/// Marker trait for `Tokenizable` types that are can tokenized to and from a `Token::Array` and
/// `Token:FixedArray`.
pub trait TokenizableItem: Tokenizable {}

macro_rules! tokenizable_item {
    ($($type: ty,)*) => {
        $(
            impl TokenizableItem for $type {}
        )*
    };
}

tokenizable_item! {
    Token, String, Address, H256, U256, I256, U128, bool, Vec<u8>,
    i8, i16, i32, i64, i128, u16, u32, u64, u128, Bytes, bytes::Bytes,
}

impl<T: TokenizableItem> TokenizableItem for Vec<T> {}

impl<const N: usize> TokenizableItem for [u8; N] {}

impl<T: TokenizableItem + Clone, const N: usize> TokenizableItem for [T; N] {}

macro_rules! impl_tokenizable_item_tuple {
    ($( $ty:ident ),+ $(,)?) => {
        impl<$( $ty ),+> TokenizableItem for ($( $ty, )+)
        where
            $(
                $ty: Tokenizable,
            )+
        {}
    }
}

impl_tokenizable_item_tuple!(A,);
impl_tokenizable_item_tuple!(A, B,);
impl_tokenizable_item_tuple!(A, B, C,);
impl_tokenizable_item_tuple!(A, B, C, D,);
impl_tokenizable_item_tuple!(A, B, C, D, E,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I, J,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I, J, K,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I, J, K, L,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T,);
impl_tokenizable_item_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U,);

/// Helper for flattening non-nested tokens into their inner types;
///
/// e.g. `(A, B, C)` would get tokenized to `Tuple([A, B, C])` when in fact we need `[A, B, C]`.
#[inline]
fn flatten_token(token: Token) -> Vec<Token> {
    // flatten the tokens if required and there is no nesting
    match token {
        Token::Tuple(inner) => inner,
        token => vec![token],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, U256};
    use ethabi::Token;

    fn assert_detokenize<T: Detokenize>() -> T {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn should_be_able_to_compile() {
        let _tokens: Vec<Token> = assert_detokenize();
        let _uint: U256 = assert_detokenize();
        let _address: Address = assert_detokenize();
        let _string: String = assert_detokenize();
        let _bool: bool = assert_detokenize();
        let _bytes: Vec<u8> = assert_detokenize();

        let _pair: (U256, bool) = assert_detokenize();
        let _vec: Vec<U256> = assert_detokenize();
        let _array: [U256; 4] = assert_detokenize();
        let _bytes: Vec<[[u8; 1]; 64]> = assert_detokenize();

        let _mixed: (Vec<Vec<u8>>, [U256; 4], Vec<U256>, U256) = assert_detokenize();

        let _ints: (i16, i32, i64, i128) = assert_detokenize();
        let _uints: (u16, u32, u64, u128) = assert_detokenize();

        let _tuple: (Address, Vec<Vec<u8>>) = assert_detokenize();
        let _vec_of_tuple: Vec<(Address, String)> = assert_detokenize();
        #[allow(clippy::type_complexity)]
        let _vec_of_tuple_5: Vec<(Address, Vec<Vec<u8>>, String, U256, bool)> = assert_detokenize();
    }

    #[test]
    fn nested_tokenization() {
        let x = (1u64, (2u64, 3u64));
        let tokens = x.into_tokens();
        assert_eq!(
            tokens,
            vec![
                Token::Uint(1.into()),
                Token::Tuple(vec![Token::Uint(2.into()), Token::Uint(3.into())])
            ]
        );

        let x = (1u64, 2u64);
        let tokens = x.into_tokens();
        assert_eq!(tokens, vec![Token::Uint(1.into()), Token::Uint(2.into()),]);
    }

    #[test]
    fn should_decode_array_of_fixed_bytes() {
        // byte[8][]
        let tokens = vec![Token::FixedArray(vec![
            Token::FixedBytes(vec![1]),
            Token::FixedBytes(vec![2]),
            Token::FixedBytes(vec![3]),
            Token::FixedBytes(vec![4]),
            Token::FixedBytes(vec![5]),
            Token::FixedBytes(vec![6]),
            Token::FixedBytes(vec![7]),
            Token::FixedBytes(vec![8]),
        ])];
        let data: [[u8; 1]; 8] = Detokenize::from_tokens(tokens).unwrap();
        assert_eq!(data[0][0], 1);
        assert_eq!(data[1][0], 2);
        assert_eq!(data[2][0], 3);
        assert_eq!(data[7][0], 8);
    }

    #[test]
    fn should_sign_extend_negative_integers() {
        assert_eq!((-1i8).into_token(), Token::Int(U256::MAX));
        assert_eq!((-2i16).into_token(), Token::Int(U256::MAX - 1));
        assert_eq!((-3i32).into_token(), Token::Int(U256::MAX - 2));
        assert_eq!((-4i64).into_token(), Token::Int(U256::MAX - 3));
        assert_eq!((-5i128).into_token(), Token::Int(U256::MAX - 4));
    }

    #[test]
    fn should_detokenize() {
        // handle tuple of one element
        let tokens = vec![Token::FixedBytes(vec![1, 2, 3, 4]), Token::Bool(true)];
        let tokens = vec![Token::Tuple(tokens)];
        let data: ([u8; 4], bool) = Detokenize::from_tokens(tokens).unwrap();
        assert_eq!(data.0[0], 1);
        assert_eq!(data.0[1], 2);
        assert_eq!(data.0[2], 3);
        assert_eq!(data.0[3], 4);
        assert!(data.1);

        // handle vector of more than one elements
        let tokens = vec![Token::Bool(false), Token::Uint(U256::from(13u8))];
        let data: (bool, u8) = Detokenize::from_tokens(tokens).unwrap();
        assert!(!data.0);
        assert_eq!(data.1, 13u8);

        // handle more than two tuples
        let tokens1 = vec![Token::FixedBytes(vec![1, 2, 3, 4]), Token::Bool(true)];
        let tokens2 = vec![Token::Bool(false), Token::Uint(U256::from(13u8))];
        let tokens = vec![Token::Tuple(tokens1), Token::Tuple(tokens2)];
        let data: (([u8; 4], bool), (bool, u8)) = Detokenize::from_tokens(tokens).unwrap();
        assert_eq!((data.0).0[0], 1);
        assert_eq!((data.0).0[1], 2);
        assert_eq!((data.0).0[2], 3);
        assert_eq!((data.0).0[3], 4);
        assert!((data.0).1);
        assert!(!(data.1).0);
        assert_eq!((data.1).1, 13u8);

        // error if no tokens in the vector
        let tokens = vec![];
        let data: Result<U256, InvalidOutputType> = Detokenize::from_tokens(tokens);
        assert!(data.is_err());
    }
}
