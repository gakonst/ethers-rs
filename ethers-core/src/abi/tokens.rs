//! Contract Functions Output types.
// Adapted from: [rust-web3](https://github.com/tomusdrw/rust-web3/blob/master/src/contract/tokens.rs)
#![allow(clippy::all)]
use crate::{
    abi::Token,
    types::{Address, Bytes, H256, U128, U256},
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
    fn from_tokens(_: Vec<Token>) -> std::result::Result<Self, InvalidOutputType>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl<T: Tokenizable> Detokenize for T {
    fn from_tokens(mut tokens: Vec<Token>) -> Result<Self, InvalidOutputType> {
        if tokens.len() != 1 {
            Err(InvalidOutputType(format!(
                "Expected single element, got a list: {:?}",
                tokens
            )))
        } else {
            Self::from_token(
                tokens
                    .drain(..)
                    .next()
                    .expect("At least one element in vector; qed"),
            )
        }
    }
}

macro_rules! impl_output {
  ($num: expr, $( $ty: ident , )+) => {
    impl<$($ty, )+> Detokenize for ($($ty,)+) where
      $(
        $ty: Tokenizable,
      )+
    {
      fn from_tokens(mut tokens: Vec<Token>) -> Result<Self, InvalidOutputType> {
        if tokens.len() != $num {
          return Err(InvalidOutputType(format!(
            "Expected {} elements, got a list of {}: {:?}",
            $num,
            tokens.len(),
            tokens
          )));
        }
        let mut it = tokens.drain(..);
        Ok(($(
          $ty::from_token(it.next().expect("All elements are in vector; qed"))?,
        )+))
      }
    }
  }
}

impl_output!(1, A,);
impl_output!(2, A, B,);
impl_output!(3, A, B, C,);
impl_output!(4, A, B, C, D,);
impl_output!(5, A, B, C, D, E,);
impl_output!(6, A, B, C, D, E, F,);
impl_output!(7, A, B, C, D, E, F, G,);
impl_output!(8, A, B, C, D, E, F, G, H,);
impl_output!(9, A, B, C, D, E, F, G, H, I,);
impl_output!(10, A, B, C, D, E, F, G, H, I, J,);
impl_output!(11, A, B, C, D, E, F, G, H, I, J, K,);
impl_output!(12, A, B, C, D, E, F, G, H, I, J, K, L,);
impl_output!(13, A, B, C, D, E, F, G, H, I, J, K, L, M,);
impl_output!(14, A, B, C, D, E, F, G, H, I, J, K, L, M, N,);
impl_output!(15, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O,);
impl_output!(16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P,);

/// Tokens conversion trait
pub trait Tokenize {
    /// Convert to list of tokens
    fn into_tokens(self) -> Vec<Token>;
}

impl<'a> Tokenize for &'a [Token] {
    fn into_tokens(self) -> Vec<Token> {
        self.to_vec()
    }
}

impl<T: Tokenizable> Tokenize for T {
    fn into_tokens(self) -> Vec<Token> {
        vec![self.into_token()]
    }
}

impl Tokenize for () {
    fn into_tokens(self) -> Vec<Token> {
        vec![]
    }
}

macro_rules! impl_tokens {
  ($( $ty: ident : $no: tt, )+) => {
    impl<$($ty, )+> Tokenize for ($($ty,)+) where
      $(
        $ty: Tokenizable,
      )+
    {
      fn into_tokens(self) -> Vec<Token> {
        vec![
          $( self.$no.into_token(), )+
        ]
      }
    }
  }
}

impl_tokens!(A:0, );
impl_tokens!(A:0, B:1, );
impl_tokens!(A:0, B:1, C:2, );
impl_tokens!(A:0, B:1, C:2, D:3, );
impl_tokens!(A:0, B:1, C:2, D:3, E:4, );
impl_tokens!(A:0, B:1, C:2, D:3, E:4, F:5, );
impl_tokens!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, );
impl_tokens!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, );
impl_tokens!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, );
impl_tokens!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, );
impl_tokens!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, );
impl_tokens!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, );
impl_tokens!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, );
impl_tokens!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, );
impl_tokens!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, );
impl_tokens!(A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, );

/// Simplified output type for single value.
pub trait Tokenizable {
    /// Converts a `Token` into expected type.
    fn from_token(token: Token) -> Result<Self, InvalidOutputType>
    where
        Self: Sized;
    /// Converts a specified type back into token.
    fn into_token(self) -> Token;
}

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
            other => Err(InvalidOutputType(format!(
                "Expected `String`, got {:?}",
                other
            ))),
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
            other => Err(InvalidOutputType(format!(
                "Expected `Bytes`, got {:?}",
                other
            ))),
        }
    }

    fn into_token(self) -> Token {
        Token::Bytes(self.0)
    }
}

impl Tokenizable for H256 {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::FixedBytes(mut s) => {
                if s.len() != 32 {
                    return Err(InvalidOutputType(format!("Expected `H256`, got {:?}", s)));
                }
                let mut data = [0; 32];
                for (idx, val) in s.drain(..).enumerate() {
                    data[idx] = val;
                }
                Ok(data.into())
            }
            other => Err(InvalidOutputType(format!(
                "Expected `H256`, got {:?}",
                other
            ))),
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
            other => Err(InvalidOutputType(format!(
                "Expected `Address`, got {:?}",
                other
            ))),
        }
    }

    fn into_token(self) -> Token {
        Token::Address(self)
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
                    other => Err(InvalidOutputType(format!(
                        "Expected `{}`, got {:?}",
                        $name, other
                    ))
                    .into()),
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

impl Tokenizable for bool {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::Bool(data) => Ok(data),
            other => Err(InvalidOutputType(format!(
                "Expected `bool`, got {:?}",
                other
            ))),
        }
    }
    fn into_token(self) -> Token {
        Token::Bool(self)
    }
}

/// Marker trait for `Tokenizable` types that are can tokenized to and from a
/// `Token::Array` and `Token:FixedArray`.
pub trait TokenizableItem: Tokenizable {}

macro_rules! tokenizable_item {
    ($($type: ty,)*) => {
        $(
            impl TokenizableItem for $type {}
        )*
    };
}

tokenizable_item! {
    Token, String, Address, H256, U256, U128, bool, Vec<u8>,
    i8, i16, i32, i64, i128, u16, u32, u64, u128,
}

impl Tokenizable for Vec<u8> {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::Bytes(data) => Ok(data),
            Token::FixedBytes(data) => Ok(data),
            other => Err(InvalidOutputType(format!(
                "Expected `bytes`, got {:?}",
                other
            ))),
        }
    }
    fn into_token(self) -> Token {
        Token::Bytes(self)
    }
}

impl<T: TokenizableItem> Tokenizable for Vec<T> {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::FixedArray(tokens) | Token::Array(tokens) => {
                tokens.into_iter().map(Tokenizable::from_token).collect()
            }
            other => Err(InvalidOutputType(format!(
                "Expected `Array`, got {:?}",
                other
            ))),
        }
    }

    fn into_token(self) -> Token {
        Token::Array(self.into_iter().map(Tokenizable::into_token).collect())
    }
}

impl<T: TokenizableItem> TokenizableItem for Vec<T> {}

macro_rules! impl_fixed_types {
    ($num: expr) => {
        impl Tokenizable for [u8; $num] {
            fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
                match token {
                    Token::FixedBytes(bytes) => {
                        if bytes.len() != $num {
                            return Err(InvalidOutputType(format!(
                                "Expected `FixedBytes({})`, got FixedBytes({})",
                                $num,
                                bytes.len()
                            )));
                        }

                        let mut arr = [0; $num];
                        arr.copy_from_slice(&bytes);
                        Ok(arr)
                    }
                    other => Err(InvalidOutputType(format!(
                        "Expected `FixedBytes({})`, got {:?}",
                        $num, other
                    ))
                    .into()),
                }
            }

            fn into_token(self) -> Token {
                Token::FixedBytes(self.to_vec())
            }
        }

        impl TokenizableItem for [u8; $num] {}

        impl<T: TokenizableItem + Clone> Tokenizable for [T; $num] {
            fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
                match token {
                    Token::FixedArray(tokens) => {
                        if tokens.len() != $num {
                            return Err(InvalidOutputType(format!(
                                "Expected `FixedArray({})`, got FixedArray({})",
                                $num,
                                tokens.len()
                            )));
                        }

                        let mut arr = ArrayVec::<[T; $num]>::new();
                        let mut it = tokens.into_iter().map(T::from_token);
                        for _ in 0..$num {
                            arr.push(it.next().expect("Length validated in guard; qed")?);
                        }
                        // Can't use expect here because [T; $num]: Debug is not satisfied.
                        match arr.into_inner() {
                            Ok(arr) => Ok(arr),
                            Err(_) => panic!("All elements inserted so the array is full; qed"),
                        }
                    }
                    other => Err(InvalidOutputType(format!(
                        "Expected `FixedArray({})`, got {:?}",
                        $num, other
                    ))
                    .into()),
                }
            }

            fn into_token(self) -> Token {
                Token::FixedArray(
                    ArrayVec::from(self)
                        .into_iter()
                        .map(T::into_token)
                        .collect(),
                )
            }
        }

        impl<T: TokenizableItem + Clone> TokenizableItem for [T; $num] {}
    };
}

impl_fixed_types!(1);
impl_fixed_types!(2);
impl_fixed_types!(3);
impl_fixed_types!(4);
impl_fixed_types!(5);
impl_fixed_types!(6);
impl_fixed_types!(7);
impl_fixed_types!(8);
impl_fixed_types!(9);
impl_fixed_types!(10);
impl_fixed_types!(11);
impl_fixed_types!(12);
impl_fixed_types!(13);
impl_fixed_types!(14);
impl_fixed_types!(15);
impl_fixed_types!(16);
impl_fixed_types!(32);
impl_fixed_types!(64);
impl_fixed_types!(128);
impl_fixed_types!(256);
impl_fixed_types!(512);
impl_fixed_types!(1024);

#[cfg(test)]
mod tests {
    use super::{Detokenize, Tokenizable};
    use crate::types::{Address, U256};
    use ethabi::Token;

    fn output<R: Detokenize>() -> R {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn should_be_able_to_compile() {
        let _tokens: Vec<Token> = output();
        let _uint: U256 = output();
        let _address: Address = output();
        let _string: String = output();
        let _bool: bool = output();
        let _bytes: Vec<u8> = output();

        let _pair: (U256, bool) = output();
        let _vec: Vec<U256> = output();
        let _array: [U256; 4] = output();
        let _bytes: Vec<[[u8; 1]; 64]> = output();

        let _mixed: (Vec<Vec<u8>>, [U256; 4], Vec<U256>, U256) = output();

        let _ints: (i16, i32, i64, i128) = output();
        let _uints: (u16, u32, u64, u128) = output();
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
}
