//! This module contains a helper type for `uint8`
//!
//! The reason this exists is to circumvent ambiguity with fixed bytes arrays

use crate::abi::{InvalidOutputType, Tokenizable, TokenizableItem};
use ethabi::{ethereum_types::U256, Token};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

/// A wrapper for `u8`
///
/// Note: this type is only necessary in conjunction with `FixedBytes` so that `[Uint8; 8]` is
/// recognized as `uint8[8]` and not fixed bytes.
///
/// See also <https://github.com/gakonst/ethers-rs/issues/1636>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Ord, PartialOrd)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Uint8(u8);

impl From<u8> for Uint8 {
    fn from(val: u8) -> Self {
        Uint8(val)
    }
}

impl From<Uint8> for u8 {
    fn from(val: Uint8) -> Self {
        val.0
    }
}

impl From<Uint8> for U256 {
    fn from(val: Uint8) -> Self {
        U256::from(val.0)
    }
}

impl PartialEq<u8> for Uint8 {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

impl Add for Uint8 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Uint8(self.0 + rhs.0)
    }
}

impl Sub for Uint8 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Uint8(self.0 - rhs.0)
    }
}

impl Add<u8> for Uint8 {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        Uint8(self.0 + rhs)
    }
}

impl Sub<u8> for Uint8 {
    type Output = Self;

    fn sub(self, rhs: u8) -> Self::Output {
        Uint8(self.0 - rhs)
    }
}

impl Tokenizable for Uint8 {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        match token {
            Token::Int(data) | Token::Uint(data) => {
                if data > U256::from(u8::MAX) {
                    return Err(InvalidOutputType("Integer overflow when casting to u8".to_string()))
                }
                Ok(Uint8(data.low_u32() as u8))
            }
            other => Err(InvalidOutputType(format!("Expected `uint8`, got {other:?}"))),
        }
    }
    fn into_token(self) -> Token {
        Token::Uint(self.into())
    }
}

impl TokenizableItem for Uint8 {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::AbiType;
    use ethabi::ParamType;

    #[test]
    fn uint8_array() {
        assert_eq!(
            <[Uint8; 8usize]>::param_type(),
            ParamType::FixedArray(Box::new(ParamType::Uint(8),), 8)
        );
    }
}
