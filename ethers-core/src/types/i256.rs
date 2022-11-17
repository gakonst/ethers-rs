//! This module contains an 256-bit signed integer implementation.
//! This module was derived for ethers-core via <https://github.com/gnosis/ethcontract-rs/>
#![allow(clippy::wrong_self_convention)]
use crate::{
    abi::{InvalidOutputType, Token, Tokenizable},
    types::U256,
    utils::ParseUnits,
};
use ethabi::ethereum_types::FromDecStrErr;
use serde::{Deserialize, Serialize};
use std::{
    cmp,
    convert::{TryFrom, TryInto},
    fmt, i128, i64, iter, ops,
    ops::Sub,
    str, u64,
};
use thiserror::Error;

/// The error type that is returned when conversion to or from a 256-bit integer
/// fails.
#[derive(Clone, Copy, Debug, Error)]
#[error("output of range integer conversion attempted")]
pub struct TryFromBigIntError;

/// The error type that is returned when parsing a 256-bit signed integer.
#[derive(Clone, Copy, Debug, Error)]
pub enum ParseI256Error {
    /// Error that occurs when an invalid digit is encountered while parsing.
    #[error("invalid digit found in string")]
    InvalidDigit,

    /// Error that occurs when the number is too large or too small (negative)
    /// and does not fit in a 256-bit signed integer.
    #[error("number does not fit in 256-bit integer")]
    IntegerOverflow,
}

impl From<FromDecStrErr> for ParseI256Error {
    fn from(err: FromDecStrErr) -> Self {
        match err {
            FromDecStrErr::InvalidCharacter => ParseI256Error::InvalidDigit,
            FromDecStrErr::InvalidLength => ParseI256Error::IntegerOverflow,
        }
    }
}

/// Compute the two's complement of a U256.
fn twos_complement(u: U256) -> U256 {
    let (twos_complement, _) = (!u).overflowing_add(U256::one());
    twos_complement
}

/// Overflow handling that depends on whether or not the binary is built in
/// debug mode.
///
/// # Panics
///
/// This function will panic on overflows in debug mode, and simply ignore the
/// overflow in non-debug mode.
#[inline(always)]
fn handle_overflow<T>((result, overflow): (T, bool)) -> T {
    #[cfg(debug_assertions)]
    {
        assert!(!overflow, "overflow");
    }

    let _ = overflow;
    result
}

/// Enum to represent the sign of a 256-bit signed integer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Sign {
    /// Greater than or equal to zero.
    Positive,
    /// Less than zero.
    Negative,
}

impl Sign {
    /// Computes the `Sign` given a signum.
    fn from_signum64(sign: i64) -> Self {
        match sign {
            0 | 1 => Sign::Positive,
            -1 => Sign::Negative,
            _ => unreachable!(),
        }
    }
}

/// Little-endian 256-bit signed integer.
///
/// ## Diversion from standard numeric types
/// The right shift operator on I256 doesn't act in the same manner as standard numeric types
/// (e.g. `i8`, `i16` etc). On standard types if the number is negative right shift will perform
/// an arithmetic shift, whereas on I256 this will perform a bit-wise shift.
/// Arithmetic shift on I256 is done via the [asr](I256::asr) and [asl](I256::asl) functions.
#[derive(Clone, Copy, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct I256(U256);

impl I256 {
    /// Maximum value.
    pub const MAX: I256 = I256(U256([u64::MAX, u64::MAX, u64::MAX, i64::MAX as _]));

    /// Minimum value.
    pub const MIN: I256 = I256(U256([0, 0, 0, i64::MIN as _]));

    /// Zero (additive identity) of this type.
    pub const fn zero() -> Self {
        I256(U256([0, 0, 0, 0]))
    }

    /// One (multiplicative identity) of this type.
    pub const fn one() -> Self {
        I256(U256([1, 0, 0, 0]))
    }

    /// Minus one (multiplicative inverse) of this type.
    pub const fn minus_one() -> Self {
        I256(U256([u64::MAX, u64::MAX, u64::MAX, u64::MAX]))
    }

    /// The maximum value which can be inhabited by this type.
    pub const fn max_value() -> Self {
        I256::MAX
    }

    /// The minimum value which can be inhabited by this type.
    pub const fn min_value() -> Self {
        I256::MIN
    }

    /// Creates an I256 from a sign and an absolute value. Returns the value and
    /// a bool that is true if the conversion caused an overflow.
    pub fn overflowing_from_sign_and_abs(sign: Sign, abs: U256) -> (Self, bool) {
        let value = I256(match sign {
            Sign::Positive => abs,
            Sign::Negative => twos_complement(abs),
        });
        (value, value.sign() != sign)
    }

    /// Creates an I256 from an absolute value and a negative flag. Returns
    /// `None` if it would overflow an `I256`.
    pub fn checked_from_sign_and_abs(sign: Sign, abs: U256) -> Option<Self> {
        let (result, overflow) = I256::overflowing_from_sign_and_abs(sign, abs);
        if overflow {
            None
        } else {
            Some(result)
        }
    }

    /// Splits a I256 into its absolute value and negative flag.
    pub fn into_sign_and_abs(self) -> (Sign, U256) {
        let sign = self.sign();
        let abs = match sign {
            Sign::Positive => self.0,
            Sign::Negative => twos_complement(self.0),
        };
        (sign, abs)
    }

    /// Returns the sign of self.
    pub fn sign(self) -> Sign {
        let most_significant_word = (self.0).0[3];
        match most_significant_word & (1 << 63) {
            0 => Sign::Positive,
            _ => Sign::Negative,
        }
    }

    /// Coerces an unsigned integer into a signed one. If the unsigned integer
    /// is greater than the greater than or equal to `1 << 255`, then the result
    /// will overflow into a negative value.
    pub const fn from_raw(raw: U256) -> Self {
        I256(raw)
    }

    /// Returns the signed integer as a unsigned integer. If the value of `self`
    /// negative, then the two's complement of its absolute value will be
    /// returned.
    pub fn into_raw(self) -> U256 {
        self.0
    }

    /// Conversion to i32
    pub fn low_i32(&self) -> i32 {
        self.0.low_u32() as _
    }

    /// Conversion to u32
    pub fn low_u32(&self) -> u32 {
        self.0.low_u32()
    }

    /// Conversion to i64
    pub fn low_i64(&self) -> i64 {
        self.0.low_u64() as _
    }

    /// Conversion to u64
    pub fn low_u64(&self) -> u64 {
        self.0.low_u64() as _
    }

    /// Conversion to i128
    pub fn low_i128(&self) -> i128 {
        self.0.low_u128() as _
    }

    /// Conversion to u128
    pub fn low_u128(&self) -> u128 {
        self.0.low_u128() as _
    }

    /// Conversion to i128
    pub fn low_isize(&self) -> isize {
        self.0.low_u64() as _
    }

    /// Conversion to usize
    pub fn low_usize(&self) -> usize {
        self.0.low_u64() as _
    }

    /// Conversion to i32 with overflow checking
    ///
    /// # Panics
    ///
    /// Panics if the number is outside the range [`i32::MIN`, `i32::MAX`].
    pub fn as_i32(&self) -> i32 {
        (*self).try_into().unwrap()
    }

    /// Conversion to u32 with overflow checking
    ///
    /// # Panics
    ///
    /// Panics if the number is outside the range [`0`, `u32::MAX`].
    pub fn as_u32(&self) -> u32 {
        (*self).try_into().unwrap()
    }

    /// Conversion to i64 with overflow checking
    ///
    /// # Panics
    ///
    /// Panics if the number is outside the range [`i64::MIN`, `i64::MAX`].
    pub fn as_i64(&self) -> i64 {
        (*self).try_into().unwrap()
    }

    /// Conversion to u64 with overflow checking
    ///
    /// # Panics
    ///
    /// Panics if the number is outside the range [`0`, `u64::MAX`].
    pub fn as_u64(&self) -> u64 {
        (*self).try_into().unwrap()
    }

    /// Conversion to i128 with overflow checking
    ///
    /// # Panics
    ///
    /// Panics if the number is outside the range [`i128::MIN`, `i128::MAX`].
    pub fn as_i128(&self) -> i128 {
        (*self).try_into().unwrap()
    }

    /// Conversion to u128 with overflow checking
    ///
    /// # Panics
    ///
    /// Panics if the number is outside the range [`0`, `u128::MAX`].
    pub fn as_u128(&self) -> u128 {
        (*self).try_into().unwrap()
    }

    /// Conversion to isize with overflow checking
    ///
    /// # Panics
    ///
    /// Panics if the number is outside the range [`isize::MIN`, `isize::MAX`].
    pub fn as_isize(&self) -> usize {
        (*self).try_into().unwrap()
    }

    /// Conversion to usize with overflow checking
    ///
    /// # Panics
    ///
    /// Panics if the number is outside the range [`0`, `usize::MAX`].
    pub fn as_usize(&self) -> usize {
        (*self).try_into().unwrap()
    }

    /// Convert from a decimal string.
    pub fn from_dec_str(value: &str) -> Result<Self, ParseI256Error> {
        let (sign, value) = match value.as_bytes().first() {
            Some(b'+') => (Sign::Positive, &value[1..]),
            Some(b'-') => (Sign::Negative, &value[1..]),
            _ => (Sign::Positive, value),
        };

        let abs = U256::from_dec_str(value)?;
        let result =
            I256::checked_from_sign_and_abs(sign, abs).ok_or(ParseI256Error::IntegerOverflow)?;

        Ok(result)
    }

    /// Convert from a hexadecimal string.
    pub fn from_hex_str(value: &str) -> Result<Self, ParseI256Error> {
        let (sign, value) = match value.as_bytes().first() {
            Some(b'+') => (Sign::Positive, &value[1..]),
            Some(b'-') => (Sign::Negative, &value[1..]),
            _ => (Sign::Positive, value),
        };

        // NOTE: Do the hex conversion here as `U256` implementation can panic.
        if value.len() > 64 {
            return Err(ParseI256Error::IntegerOverflow)
        }
        let mut abs = U256::zero();
        for (i, word) in value.as_bytes().rchunks(16).enumerate() {
            let word = str::from_utf8(word).map_err(|_| ParseI256Error::InvalidDigit)?;
            abs.0[i] = u64::from_str_radix(word, 16).map_err(|_| ParseI256Error::InvalidDigit)?;
        }

        let result =
            I256::checked_from_sign_and_abs(sign, abs).ok_or(ParseI256Error::IntegerOverflow)?;

        Ok(result)
    }

    /// Returns the sign of the number.
    #[must_use]
    pub fn signum(self) -> Self {
        self.signum64().into()
    }

    /// Returns an `i64` representing the sign of the number.
    fn signum64(self) -> i64 {
        match self.sign() {
            Sign::Positive => (!self.is_zero()) as i64,
            Sign::Negative => -1,
        }
    }

    /// Returns `true` if `self` is positive and `false` if the number is zero
    /// or negative.
    pub fn is_positive(self) -> bool {
        self.signum64().is_positive()
    }

    /// Returns `true` if `self` is negative and `false` if the number is zero
    /// or negative.
    pub fn is_negative(self) -> bool {
        self.signum64().is_negative()
    }

    /// Returns `true` if `self` is negative and `false` if the number is zero
    /// or positive.
    pub fn is_zero(self) -> bool {
        self.0.is_zero()
    }

    /// Gets the absolute value.
    ///
    /// # Panics
    ///
    /// In debug mode, will panic if it overflows.
    #[must_use]
    pub fn abs(self) -> Self {
        handle_overflow(self.overflowing_abs())
    }

    /// Computes the absolute value of self.
    ///
    /// Returns a tuple of the absolute version of self along with a boolean
    /// indicating whether an overflow happened. If self is the minimum value
    /// (e.g., `I256::MIN` for values of type `I256`), then the minimum value
    /// will be returned again and true will be returned for an overflow
    /// happening.
    pub fn overflowing_abs(self) -> (Self, bool) {
        if self == I256::MIN {
            (self, true)
        } else {
            (I256(self.abs_unsigned()), false)
        }
    }

    /// Checked absolute value. Computes `self.abs()`, returning `None` if
    /// `self == MIN`.
    pub fn checked_abs(self) -> Option<Self> {
        let (result, overflow) = self.overflowing_abs();
        if overflow {
            None
        } else {
            Some(result)
        }
    }

    /// Saturating absolute value. Computes `self.abs()`, returning `MAX` if
    /// `self == MIN` instead of overflowing.
    #[must_use]
    pub fn saturating_abs(self) -> Self {
        self.checked_abs().unwrap_or(I256::MAX)
    }

    /// Wrapping absolute value. Computes `self.abs()`, wrapping around at the
    /// boundary of the type.
    #[must_use]
    pub fn wrapping_abs(self) -> Self {
        let (result, _) = self.overflowing_abs();
        result
    }

    /// Gets the absolute value as an unsigned integer.
    fn abs_unsigned(self) -> U256 {
        let (_, abs) = self.into_sign_and_abs();
        abs
    }

    /// Negates self, overflowing if this is equal to the minimum value.
    ///
    /// Returns a tuple of the negated version of `self` along with a boolean
    /// indicating whether an overflow happened. If `self` is the minimum value,
    /// then the minimum value will be returned again and `true` will be
    /// returned for an overflow happening.
    pub fn overflowing_neg(self) -> (Self, bool) {
        if self == I256::MIN {
            (self, true)
        } else {
            (I256(twos_complement(self.0)), false)
        }
    }

    /// Checked negation. Computes `self.neg()`, returning `None` if
    /// `self == MIN`.
    pub fn checked_neg(self) -> Option<Self> {
        let (result, overflow) = self.overflowing_neg();
        if overflow {
            None
        } else {
            Some(result)
        }
    }

    /// Saturating negation. Computes `self.neg()`, returning `MAX` if
    /// `self == MIN` instead of overflowing.
    #[must_use]
    pub fn saturating_neg(self) -> Self {
        self.checked_neg().unwrap_or(I256::MAX)
    }

    /// Wrapping negation. Computes `self.neg()`, returning `MIN` if
    /// `self == MIN` instead of overflowing.
    #[must_use]
    pub fn wrapping_neg(self) -> Self {
        let (result, _) = self.overflowing_neg();
        result
    }

    /// Return the least number of bits needed to represent the number.
    pub fn bits(&self) -> u32 {
        let unsigned = self.abs_unsigned();
        let unsigned_bits = unsigned.bits();

        // NOTE: We need to deal with two special cases:
        //   - the number is 0
        //   - the number is a negative power of `2`. These numbers are written as `0b11..1100..00`.
        //   In the case of a negative power of two, the number of bits required
        //   to represent the negative signed value is equal to the number of
        //   bits required to represent its absolute value as an unsigned
        //   integer. This is best illustrated by an example: the number of bits
        //   required to represent `-128` is `8` since it is equal to `i8::MIN`
        //   and, therefore, obviously fits in `8` bits. This is equal to the
        //   number of bits required to represent `128` as an unsigned integer
        //   (which fits in a `u8`).  However, the number of bits required to
        //   represent `128` as a signed integer is `9`, as it is greater than
        //   `i8::MAX`.  In the general case, an extra bit is needed to
        //   represent the sign.
        let bits = if self.count_zeros() == self.trailing_zeros() {
            // `self` is zero or a negative power of two
            unsigned_bits
        } else {
            unsigned_bits + 1
        };

        bits as _
    }

    /// Return if specific bit is set.
    ///
    /// # Panics
    ///
    /// Panics if index exceeds the bit width of the number.
    pub fn bit(&self, index: usize) -> bool {
        self.0.bit(index)
    }

    /// Returns the number of ones in the binary representation of `self`.
    pub fn count_ones(&self) -> u32 {
        (self.0).0.iter().map(|word| word.count_ones()).sum()
    }

    /// Returns the number of zeros in the binary representation of `self`.
    pub fn count_zeros(&self) -> u32 {
        (self.0).0.iter().map(|word| word.count_zeros()).sum()
    }

    /// Returns the number of leading zeros in the binary representation of
    /// `self`.
    pub fn leading_zeros(&self) -> u32 {
        self.0.leading_zeros()
    }

    /// Returns the number of leading zeros in the binary representation of
    /// `self`.
    pub fn trailing_zeros(&self) -> u32 {
        self.0.trailing_zeros()
    }

    /// Return specific byte.
    ///
    /// # Panics
    ///
    /// Panics if index exceeds the byte width of the number.
    pub fn byte(&self, index: usize) -> u8 {
        self.0.byte(index)
    }

    /// Write to the slice in big-endian format.
    pub fn to_big_endian(&self, bytes: &mut [u8]) {
        self.0.to_big_endian(bytes)
    }

    /// Write to the slice in little-endian format.
    pub fn to_little_endian(&self, bytes: &mut [u8]) {
        self.0.to_little_endian(bytes)
    }

    /// Calculates `self` + `rhs`
    ///
    /// Returns a tuple of the addition along with a boolean indicating whether
    /// an arithmetic overflow would occur. If an overflow would have occurred
    /// then the wrapped value is returned.
    pub fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let (unsigned, _) = self.0.overflowing_add(rhs.0);
        let result = I256(unsigned);

        // NOTE: Overflow is determined by checking the sign of the operands and
        //   the result.
        let overflow = matches!(
            (self.sign(), rhs.sign(), result.sign()),
            (Sign::Positive, Sign::Positive, Sign::Negative) |
                (Sign::Negative, Sign::Negative, Sign::Positive)
        );

        (result, overflow)
    }

    /// Checked addition. Returns None if overflow occurred.
    pub fn checked_add(self, other: Self) -> Option<Self> {
        let (result, overflow) = self.overflowing_add(other);
        if overflow {
            None
        } else {
            Some(result)
        }
    }

    /// Addition which saturates at the maximum value (Self::max_value()).
    #[must_use]
    pub fn saturating_add(self, other: Self) -> Self {
        let (result, overflow) = self.overflowing_add(other);
        if overflow {
            match result.sign() {
                Sign::Positive => I256::MIN,
                Sign::Negative => I256::MAX,
            }
        } else {
            result
        }
    }

    /// Wrapping addition.
    #[must_use]
    pub fn wrapping_add(self, other: Self) -> Self {
        let (result, _) = self.overflowing_add(other);
        result
    }

    /// Calculates `self` - `rhs`
    ///
    /// Returns a tuple of the subtraction along with a boolean indicating
    /// whether an arithmetic overflow would occur. If an overflow would have
    /// occurred then the wrapped value is returned.
    pub fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        // NOTE: We can't just compute the `self + (-rhs)` because `-rhs` does
        //   not always exist, specifically this would be a problem in case
        //   `rhs == I256::MIN`

        let (unsigned, _) = self.0.overflowing_sub(rhs.0);
        let result = I256(unsigned);

        // NOTE: Overflow is determined by checking the sign of the operands and
        //   the result.
        let overflow = matches!(
            (self.sign(), rhs.sign(), result.sign()),
            (Sign::Positive, Sign::Negative, Sign::Negative) |
                (Sign::Negative, Sign::Positive, Sign::Positive)
        );

        (result, overflow)
    }

    /// Checked subtraction. Returns None if overflow occurred.
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        let (result, overflow) = self.overflowing_sub(other);
        if overflow {
            None
        } else {
            Some(result)
        }
    }

    /// Subtraction which saturates at zero.
    #[must_use]
    pub fn saturating_sub(self, other: Self) -> Self {
        let (result, overflow) = self.overflowing_sub(other);
        if overflow {
            match result.sign() {
                Sign::Positive => I256::MIN,
                Sign::Negative => I256::MAX,
            }
        } else {
            result
        }
    }

    /// Wrapping subtraction.
    #[must_use]
    pub fn wrapping_sub(self, other: Self) -> Self {
        let (result, _) = self.overflowing_sub(other);
        result
    }

    /// Calculates `self` * `rhs`
    ///
    /// Returns a tuple of the multiplication along with a boolean indicating
    /// whether an arithmetic overflow would occur. If an overflow would have
    /// occurred then the wrapped value is returned.
    pub fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
        let sign = Sign::from_signum64(self.signum64() * rhs.signum64());
        let (unsigned, overflow_mul) = self.abs_unsigned().overflowing_mul(rhs.abs_unsigned());
        let (result, overflow_conv) = I256::overflowing_from_sign_and_abs(sign, unsigned);

        (result, overflow_mul || overflow_conv)
    }

    /// Checked multiplication. Returns None if overflow occurred.
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        let (result, overflow) = self.overflowing_mul(other);
        if overflow {
            None
        } else {
            Some(result)
        }
    }

    /// Multiplication which saturates at the maximum value..
    #[must_use]
    pub fn saturating_mul(self, rhs: Self) -> Self {
        self.checked_mul(rhs).unwrap_or_else(|| {
            match Sign::from_signum64(self.signum64() * rhs.signum64()) {
                Sign::Positive => I256::MAX,
                Sign::Negative => I256::MIN,
            }
        })
    }

    /// Wrapping multiplication.
    #[must_use]
    pub fn wrapping_mul(self, rhs: Self) -> Self {
        let (result, _) = self.overflowing_mul(rhs);
        result
    }

    /// Calculates `self` / `rhs`
    ///
    /// Returns a tuple of the division along with a boolean indicating
    /// whether an arithmetic overflow would occur. If an overflow would have
    /// occurred then the wrapped value is returned.
    pub fn overflowing_div(self, rhs: Self) -> (Self, bool) {
        // Panic early when with division by zero while evaluating sign.
        let sign = Sign::from_signum64(self.signum64() / rhs.signum64());
        // Note, signed division can't overflow!
        let unsigned = self.abs_unsigned() / rhs.abs_unsigned();
        let (result, overflow_conv) = I256::overflowing_from_sign_and_abs(sign, unsigned);

        (result, overflow_conv && !result.is_zero())
    }

    /// Checked division. Returns None if overflow occurred or if rhs == 0.
    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs == I256::zero() || (self == Self::min_value() && rhs == -I256::one()) {
            None
        } else {
            Some(self.overflowing_div(rhs).0)
        }
    }

    /// Division which saturates at the maximum value.
    #[must_use]
    pub fn saturating_div(self, rhs: Self) -> Self {
        // There is only one overflow (I256::MIN / -1 = I256::MAX)
        self.checked_div(rhs).unwrap_or(I256::MAX)
    }

    /// Wrapping division.
    #[must_use]
    pub fn wrapping_div(self, rhs: Self) -> Self {
        self.overflowing_div(rhs).0
    }

    /// Calculates `self` % `rhs`
    ///
    /// Returns a tuple of the remainder along with a boolean indicating
    /// whether an arithmetic overflow would occur. If an overflow would have
    /// occurred then the wrapped value is returned.
    pub fn overflowing_rem(self, rhs: Self) -> (Self, bool) {
        if self == Self::MIN && rhs == Self::from(-1) {
            (Self::zero(), true)
        } else {
            let div_res = self / rhs;
            (self - div_res * rhs, false)
        }
    }

    /// Checked remainder. Returns None if overflow occurred or rhs == 0
    pub fn checked_rem(self, rhs: Self) -> Option<Self> {
        if rhs == I256::zero() || (self == Self::min_value() && rhs == -I256::one()) {
            None
        } else {
            Some(self.overflowing_rem(rhs).0)
        }
    }

    /// Wrapping remainder. Returns the result of the operation %
    /// regardless of whether or not the division overflowed.
    #[must_use]
    pub fn wrapping_rem(self, rhs: Self) -> Self {
        self.overflowing_rem(rhs).0
    }

    /// Calculates the quotient of Euclidean division of self by rhs.
    ///
    /// This computes the integer `n` such that `self = n * rhs + self.rem_euclid(rhs)`,
    /// with `0 <= self.rem_euclid(rhs) < rhs`.
    /// In other words, the result is `self / rhs` rounded to the integer `n` such that `self >= n *
    /// rhs`:
    /// * If `self > 0`, this is equal to round towards zero (the default in Rust);
    /// * If `self < 0`, this is equal to round towards +/- infinity.
    #[must_use]
    pub fn div_euclid(self, rhs: Self) -> Self {
        let q = self / rhs;
        if (self % rhs).is_negative() {
            return if rhs.is_positive() { q - I256::one() } else { q + I256::one() }
        }
        q
    }

    /// Calculates the least non-negative remainder of self (mod rhs).
    /// This is done as if by the _Euclidean division algorithm_
    /// given `r = self.rem_euclid(rhs)`, `self = rhs * self.div_euclid(rhs) + r, and 0 <= r <
    /// abs(rhs)`.
    #[must_use]
    pub fn rem_euclid(self, rhs: Self) -> Self {
        let r = self % rhs;
        if r < Self::zero() {
            if rhs < Self::zero() {
                r - rhs
            } else {
                r + rhs
            }
        } else {
            r
        }
    }

    /// Calculates the quotient of Euclidean division `self.div_euclid(rhs)`.
    /// Returns a tuple of the divisor along with a boolean indicating whether an arithmetic
    /// overflow would occur. If an overflow would occur then `self` is returned.
    pub fn overflowing_div_euclid(self, rhs: Self) -> (Self, bool) {
        if self == Self::min_value() && rhs == -I256::one() {
            (self, true)
        } else {
            (self.div_euclid(rhs), false)
        }
    }

    /// Checked Euclidean division. Computes `self.div_euclid(rhs)`,
    /// returning None if `rhs == 0` or the division results in overflow.
    pub fn checked_div_euclid(self, rhs: Self) -> Option<Self> {
        if rhs == I256::zero() || (self == Self::min_value() && rhs == -I256::one()) {
            None
        } else {
            Some(self.div_euclid(rhs))
        }
    }

    /// Wrapping Euclidean division.
    /// Computes `self.div_euclid(rhs)`, wrapping around at the boundary of the type.
    /// Wrapping only occurs in `MIN / -1` on a signed type
    /// (where `MIN` is the negative minimal value for the type).
    /// This is equivalent to `-MIN`, a positive value that is too large to represent in the type.
    /// In this case, this method returns `MIN` itself.
    #[must_use]
    pub fn wrapping_div_euclid(self, rhs: Self) -> Self {
        self.overflowing_div_euclid(rhs).0
    }

    /// Overflowing Euclidean remainder. Calculates `self.rem_euclid(rhs)`.
    /// Returns a tuple of the remainder after dividing along with a boolean indicating whether
    /// an arithmetic overflow would occur. If an overflow would occur then `0` is returned.
    /// Panics if `rhs == 0`
    pub fn overflowing_rem_euclid(self, rhs: Self) -> (Self, bool) {
        if self == Self::min_value() && rhs == -Self::one() {
            (Self::zero(), true)
        } else {
            (self.rem_euclid(rhs), false)
        }
    }

    /// Wrapping Euclidean remainder.
    /// Computes `self.rem_euclid(rhs)`, wrapping around at the boundary of the type.
    /// Wrapping will only occur in `MIN % -1` on a signed type
    /// (where `MIN` is the negative minimal value for the type).
    /// In this case, this method returns `0`.
    /// Panics when `rhs == 0`
    #[must_use]
    pub fn wrapping_rem_euclid(self, rhs: Self) -> Self {
        self.overflowing_rem_euclid(rhs).0
    }

    /// Checked Euclidean remainder. Computes `self.rem_euclid(rhs)`,
    /// returning `None` if `rhs == 0` or the division results in overflow.
    pub fn checked_rem_euclid(self, rhs: Self) -> Option<Self> {
        if rhs == I256::zero() || (self == Self::min_value() && rhs == -I256::one()) {
            None
        } else {
            Some(self.rem_euclid(rhs))
        }
    }

    /// Returns the sign of `self` to the exponent `exp`.
    ///
    /// Note that this method does not actually try to compute the `self` to the
    /// exponent `exp`, but instead uses the property that a negative number to
    /// an odd exponent will be negative. This means that the sign of the result
    /// of exponentiation can be computed even if the actual result is too large
    /// to fit in 256-bit signed integer.
    fn pow_sign(self, exp: u32) -> Sign {
        let is_exp_odd = exp % 2 != 0;
        if is_exp_odd && self.is_negative() {
            Sign::Negative
        } else {
            Sign::Positive
        }
    }

    /// Create 10**n as this type.
    ///
    /// # Panics
    ///
    /// Panics if the result overflows the type.
    pub fn exp10(n: usize) -> Self {
        U256::exp10(n).try_into().expect("overflow")
    }

    /// Raise self to the power of `exp`.
    ///
    /// # Panics
    ///
    /// Panics if the result overflows the type in debug mode.
    #[must_use]
    pub fn pow(self, exp: u32) -> Self {
        handle_overflow(self.overflowing_pow(exp))
    }

    /// Raises self to the power of `exp`.
    ///
    /// Returns a tuple of the exponentiation along with a bool indicating
    /// whether an overflow happened.
    pub fn overflowing_pow(self, exp: u32) -> (Self, bool) {
        let sign = self.pow_sign(exp);
        let (unsigned, overflow_pow) = self.abs_unsigned().overflowing_pow(exp.into());
        let (result, overflow_conv) = I256::overflowing_from_sign_and_abs(sign, unsigned);

        (result, overflow_pow || overflow_conv)
    }

    /// Raises self to the power of `exp`. Returns None if overflow occurred.
    pub fn checked_pow(self, exp: u32) -> Option<Self> {
        let (result, overflow) = self.overflowing_pow(exp);
        if overflow {
            None
        } else {
            Some(result)
        }
    }

    /// Raises self to the power of `exp`, saturating at the numeric bounds
    /// instead of overflowing.
    #[must_use]
    pub fn saturating_pow(self, exp: u32) -> Self {
        let (result, overflow) = self.overflowing_pow(exp);
        if overflow {
            match self.pow_sign(exp) {
                Sign::Positive => I256::MAX,
                Sign::Negative => I256::MIN,
            }
        } else {
            result
        }
    }

    /// Wrapping powolute value. Computes `self.pow()`, wrapping around at the
    /// boundary of the type.
    #[must_use]
    pub fn wrapping_pow(self, exp: u32) -> Self {
        let (result, _) = self.overflowing_pow(exp);
        result
    }

    /// Arithmetic Shift Right operation. Shifts `shift` number of times to the right maintaining
    /// the original sign. If the number is positive this is the same as logic shift right.
    pub fn asr(self, shift: u32) -> Self {
        // Avoid shifting if we are going to know the result regardless of the value.
        if shift == 0 {
            self
        } else if shift >= 255u32 {
            match self.sign() {
                // It's always going to be zero (i.e. 00000000...00000000)
                Sign::Positive => Self::zero(),
                // It's always going to be -1 (i.e. 11111111...11111111)
                Sign::Negative => Self::minus_one(),
            }
        } else {
            // Perform the shift.
            match self.sign() {
                Sign::Positive => self >> shift,
                // We need to do: `for 0..shift { self >> 1 | 2^255 }`
                // We can avoid the loop by doing: `self >> shift | ~(2^(255 - shift) - 1)`
                // where '~' represents ones complement
                Sign::Negative => {
                    let bitwise_or =
                        Self::from_raw(!U256::from(2u8).pow(U256::from(255u32 - shift)).sub(1u8));
                    (self >> shift) | bitwise_or
                }
            }
        }
    }

    /// Arithmetic Shift Left operation. Shifts `shift` number of times to the left, checking for
    /// overflow on the final result.
    ///
    /// Returns `None` if the operation overflowed (most significant bit changes).
    pub fn asl(self, shift: u32) -> Option<Self> {
        if shift == 0 {
            Some(self)
        } else {
            let result = self << shift;
            if result.sign() != self.sign() {
                // Overflow occurred
                None
            } else {
                Some(result)
            }
        }
    }

    /// Compute the twos complement of the I256
    pub fn twos_complement(self) -> U256 {
        match self.sign() {
            Sign::Positive => self.into_raw(),
            Sign::Negative => twos_complement(self.into_raw()),
        }
    }
}

macro_rules! impl_from {
    ($( $t:ty ),*) => {
        $(
            impl From<$t> for I256 {
                fn from(value: $t) -> Self {
                    #[allow(unused_comparisons)]
                    I256(if value < 0 {
                        let abs = (!(value as u128)).wrapping_add(1);
                        twos_complement(U256::from(abs))
                    } else {
                        U256::from(value)
                    })
                }
            }

            impl TryFrom<I256> for $t {
                type Error = TryFromBigIntError;

                fn try_from(value: I256) -> Result<Self, Self::Error> {
                    if value < I256::from(Self::min_value()) ||
                        value > I256::from(Self::max_value()) {
                        return Err(TryFromBigIntError);
                    }

                    Ok(value.0.low_u128() as _)
                }
            }
        )*
    };
}

impl_from!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize);

impl TryFrom<U256> for I256 {
    type Error = TryFromBigIntError;

    fn try_from(from: U256) -> Result<Self, Self::Error> {
        let value = I256(from);
        match value.sign() {
            Sign::Positive => Ok(value),
            Sign::Negative => Err(TryFromBigIntError),
        }
    }
}

impl TryFrom<I256> for U256 {
    type Error = TryFromBigIntError;

    fn try_from(value: I256) -> Result<Self, Self::Error> {
        match value.sign() {
            Sign::Positive => Ok(value.0),
            Sign::Negative => Err(TryFromBigIntError),
        }
    }
}

impl From<ParseUnits> for I256 {
    fn from(n: ParseUnits) -> Self {
        match n {
            ParseUnits::U256(n) => Self::from_raw(n),
            ParseUnits::I256(n) => n,
        }
    }
}

impl str::FromStr for I256 {
    type Err = ParseI256Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        I256::from_hex_str(value)
    }
}

impl fmt::Debug for I256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self, f.sign_plus()) {
            (Sign::Positive, false) => Ok(()),
            (Sign::Positive, true) => write!(f, "+"),
            (Sign::Negative, _) => write!(f, "-"),
        }
    }
}

impl fmt::Display for I256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (sign, abs) = self.into_sign_and_abs();
        sign.fmt(f)?;
        write!(f, "{abs}")
    }
}

impl fmt::LowerHex for I256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (sign, abs) = self.into_sign_and_abs();
        fmt::Display::fmt(&sign, f)?;
        write!(f, "{abs:x}")
    }
}

impl fmt::UpperHex for I256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (sign, abs) = self.into_sign_and_abs();
        fmt::Display::fmt(&sign, f)?;

        // NOTE: Work around `U256: !UpperHex`.
        let mut buffer = format!("{abs:x}");
        buffer.make_ascii_uppercase();
        write!(f, "{buffer}")
    }
}

impl cmp::PartialOrd for I256 {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for I256 {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        // TODO(nlordell): Once subtraction is implemented:
        // self.saturating_sub(*other).signum64().partial_cmp(&0)

        use cmp::Ordering::*;
        use Sign::*;

        match (self.into_sign_and_abs(), other.into_sign_and_abs()) {
            ((Positive, _), (Negative, _)) => Greater,
            ((Negative, _), (Positive, _)) => Less,
            ((Positive, this), (Positive, other)) => this.cmp(&other),
            ((Negative, this), (Negative, other)) => other.cmp(&this),
        }
    }
}

impl ops::Neg for I256 {
    type Output = I256;

    fn neg(self) -> Self::Output {
        handle_overflow(self.overflowing_neg())
    }
}

impl ops::Not for I256 {
    type Output = I256;

    fn not(self) -> Self::Output {
        I256(!self.0)
    }
}

impl ops::BitAnd for I256 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        I256(self.0 & rhs.0)
    }
}

impl ops::BitAndAssign for I256 {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl ops::BitOr for I256 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        I256(self.0 | rhs.0)
    }
}

impl ops::BitOrAssign for I256 {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl ops::BitXor for I256 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        I256(self.0 ^ rhs.0)
    }
}

impl ops::BitXorAssign for I256 {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

macro_rules! impl_shift {
    ($( $t:ty $( [ $convert:ident ] )? ),*) => {
        $(
            impl_shift!(__impl $t $([$convert])*);
        )*
    };
    (__impl $t:ty) => {
        impl_shift!(__impl $t [ from ]);
    };
    (__impl $t:ty [ $convert:ident ]) => {
        impl ops::Shl<$t> for I256 {
            type Output = Self;

            fn shl(self, rhs: $t) -> Self::Output {
                // NOTE: We are OK with wrapping behaviour here, that is how
                //   Rust behaves with the primitive integer types.
                I256(self.0 << I256::$convert(rhs).0)
            }
        }

        impl ops::ShlAssign<$t> for I256 {
            fn shl_assign(&mut self, rhs: $t) {
                *self = *self << rhs;
            }
        }

        /// Implements the logical shift right operation
        impl ops::Shr<$t> for I256 {
            type Output = Self;

            fn shr(self, rhs: $t) -> Self::Output {
                I256(self.0 >> I256::$convert(rhs).0)
            }
        }

        impl ops::ShrAssign<$t> for I256 {
            fn shr_assign(&mut self, rhs: $t) {
                *self = *self >> rhs;
            }
        }
    };
}

impl_shift!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize);
impl_shift!(I256, U256[from_raw]);

impl ops::Add for I256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        handle_overflow(self.overflowing_add(rhs))
    }
}

impl ops::AddAssign for I256 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl ops::Sub for I256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        handle_overflow(self.overflowing_sub(rhs))
    }
}

impl ops::SubAssign for I256 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl ops::Mul for I256 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        handle_overflow(self.overflowing_mul(rhs))
    }
}

impl ops::MulAssign for I256 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl ops::Div for I256 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        handle_overflow(self.overflowing_div(rhs))
    }
}

impl ops::DivAssign for I256 {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl ops::Rem for I256 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        handle_overflow(self.overflowing_rem(rhs))
    }
}

impl ops::RemAssign for I256 {
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl iter::Sum for I256 {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(I256::zero(), |acc, x| acc + x)
    }
}

impl iter::Product for I256 {
    fn product<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(I256::one(), |acc, x| acc * x)
    }
}

impl Tokenizable for I256 {
    fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
        Ok(I256(U256::from_token(token)?))
    }

    fn into_token(self) -> Token {
        Token::Int(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::Tokenizable;
    use once_cell::sync::Lazy;
    use serde_json::json;
    use std::ops::Neg;

    static MIN_ABS: Lazy<U256> = Lazy::new(|| U256::from(1) << 255);

    #[test]
    fn identities() {
        const ONE: I256 = I256::from_raw(U256([1, 0, 0, 0]));
        assert_eq!(ONE, I256::one());

        assert_eq!(I256::zero().to_string(), "0");
        assert_eq!(I256::one().to_string(), "1");
        assert_eq!(I256::minus_one().to_string(), "-1");
        assert_eq!(
            I256::max_value().to_string(),
            "57896044618658097711785492504343953926634992332820282019728792003956564819967"
        );
        assert_eq!(
            I256::min_value().to_string(),
            "-57896044618658097711785492504343953926634992332820282019728792003956564819968"
        );
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn std_num_conversion() {
        let small_positive = I256::from(42);
        let small_negative = I256::from(-42);
        let large_positive =
            I256::from_dec_str("314159265358979323846264338327950288419716").unwrap();
        let large_negative =
            I256::from_dec_str("-314159265358979323846264338327950288419716").unwrap();
        let large_unsigned =
            U256::from_dec_str("314159265358979323846264338327950288419716").unwrap();

        macro_rules! assert_from {
            ($signed:ty, $unsigned:ty) => {
                assert_eq!(I256::from(-42 as $signed).to_string(), "-42");
                assert_eq!(I256::from(42 as $signed).to_string(), "42");
                assert_eq!(
                    I256::from(<$signed>::max_value()).to_string(),
                    <$signed>::max_value().to_string(),
                );
                assert_eq!(
                    I256::from(<$signed>::min_value()).to_string(),
                    <$signed>::min_value().to_string(),
                );

                assert_eq!(I256::from(42 as $unsigned).to_string(), "42");
                assert_eq!(
                    I256::from(<$unsigned>::max_value()).to_string(),
                    <$unsigned>::max_value().to_string(),
                );

                assert!(matches!(<$signed>::try_from(small_positive), Ok(42)));
                assert!(matches!(<$signed>::try_from(small_negative), Ok(-42)));
                assert!(matches!(<$signed>::try_from(large_positive), Err(_)));
                assert!(matches!(<$signed>::try_from(large_negative), Err(_)));

                assert!(matches!(<$unsigned>::try_from(small_positive), Ok(42)));
                assert!(matches!(<$unsigned>::try_from(small_negative), Err(_)));
                assert!(matches!(<$unsigned>::try_from(large_positive), Err(_)));
                assert!(matches!(<$unsigned>::try_from(large_negative), Err(_)));
            };
        }

        assert_eq!(I256::from(0).to_string(), "0");

        assert_from!(i8, u8);
        assert_from!(i16, u16);
        assert_from!(i32, u32);
        assert_from!(i64, u64);
        assert_from!(i128, u128);

        assert_eq!(I256::try_from(large_unsigned).unwrap(), large_positive);
        assert_eq!(U256::try_from(large_positive).unwrap(), large_unsigned);
        I256::try_from(U256::MAX).unwrap_err();
        U256::try_from(small_negative).unwrap_err();
        U256::try_from(large_negative).unwrap_err();
    }

    #[test]
    fn parse_dec_str() {
        let unsigned = U256::from_dec_str("314159265358979323846264338327950288419716").unwrap();

        let value = I256::from_dec_str(&format!("-{unsigned}")).unwrap();
        assert_eq!(value.into_sign_and_abs(), (Sign::Negative, unsigned));

        let value = I256::from_dec_str(&format!("{unsigned}")).unwrap();
        assert_eq!(value.into_sign_and_abs(), (Sign::Positive, unsigned));

        let value = I256::from_dec_str(&format!("+{unsigned}")).unwrap();
        assert_eq!(value.into_sign_and_abs(), (Sign::Positive, unsigned));

        let err = I256::from_dec_str("invalid string").unwrap_err();
        assert!(matches!(err, ParseI256Error::InvalidDigit));

        let err = I256::from_dec_str(&format!("1{}", U256::MAX)).unwrap_err();
        assert!(matches!(err, ParseI256Error::IntegerOverflow));

        let err = I256::from_dec_str(&format!("-{}", U256::MAX)).unwrap_err();
        assert!(matches!(err, ParseI256Error::IntegerOverflow));

        let value = I256::from_dec_str(&format!("-{}", *MIN_ABS)).unwrap();
        assert_eq!(value.into_sign_and_abs(), (Sign::Negative, *MIN_ABS));

        let err = I256::from_dec_str(&format!("{}", *MIN_ABS)).unwrap_err();
        assert!(matches!(err, ParseI256Error::IntegerOverflow));
    }

    #[test]
    fn parse_hex_str() {
        let unsigned = U256::from_dec_str("314159265358979323846264338327950288419716").unwrap();

        let value = I256::from_hex_str(&format!("-{unsigned:x}")).unwrap();
        assert_eq!(value.into_sign_and_abs(), (Sign::Negative, unsigned));

        let value = I256::from_hex_str(&format!("{unsigned:x}")).unwrap();
        assert_eq!(value.into_sign_and_abs(), (Sign::Positive, unsigned));

        let value = I256::from_hex_str(&format!("+{unsigned:x}")).unwrap();
        assert_eq!(value.into_sign_and_abs(), (Sign::Positive, unsigned));

        let err = I256::from_hex_str("invalid string").unwrap_err();
        assert!(matches!(err, ParseI256Error::InvalidDigit));

        let err = I256::from_hex_str(&format!("1{:x}", U256::MAX)).unwrap_err();
        assert!(matches!(err, ParseI256Error::IntegerOverflow));

        let err = I256::from_hex_str(&format!("-{:x}", U256::MAX)).unwrap_err();
        assert!(matches!(err, ParseI256Error::IntegerOverflow));

        let value = I256::from_hex_str(&format!("-{:x}", *MIN_ABS)).unwrap();
        assert_eq!(value.into_sign_and_abs(), (Sign::Negative, *MIN_ABS));

        let err = I256::from_hex_str(&format!("{:x}", *MIN_ABS)).unwrap_err();
        assert!(matches!(err, ParseI256Error::IntegerOverflow));
    }

    #[test]
    fn formatting() {
        let unsigned = U256::from_dec_str("314159265358979323846264338327950288419716").unwrap();
        let positive = I256::try_from(unsigned).unwrap();
        let negative = -positive;

        assert_eq!(format!("{positive}"), format!("{unsigned}"));
        assert_eq!(format!("{negative}"), format!("-{unsigned}"));
        assert_eq!(format!("{positive:+}"), format!("+{unsigned}"));
        assert_eq!(format!("{negative:+}"), format!("-{unsigned}"));

        assert_eq!(format!("{positive:x}"), format!("{unsigned:x}"));
        assert_eq!(format!("{negative:x}"), format!("-{unsigned:x}"));
        assert_eq!(format!("{positive:+x}"), format!("+{unsigned:x}"));
        assert_eq!(format!("{negative:+x}"), format!("-{unsigned:x}"));

        assert_eq!(format!("{positive:X}"), format!("{unsigned:x}").to_uppercase());
        assert_eq!(format!("{negative:X}"), format!("-{unsigned:x}").to_uppercase());
        assert_eq!(format!("{positive:+X}"), format!("+{unsigned:x}").to_uppercase());
        assert_eq!(format!("{negative:+X}"), format!("-{unsigned:x}").to_uppercase());
    }

    #[test]
    fn signs() {
        assert_eq!(I256::MAX.signum(), I256::one());
        assert!(I256::MAX.is_positive());
        assert!(!I256::MAX.is_negative());
        assert!(!I256::MAX.is_zero());

        assert_eq!(I256::one().signum(), I256::one());
        assert!(I256::one().is_positive());
        assert!(!I256::one().is_negative());
        assert!(!I256::one().is_zero());

        assert_eq!(I256::MIN.signum(), I256::minus_one());
        assert!(!I256::MIN.is_positive());
        assert!(I256::MIN.is_negative());
        assert!(!I256::MIN.is_zero());

        assert_eq!(I256::minus_one().signum(), I256::minus_one());
        assert!(!I256::minus_one().is_positive());
        assert!(I256::minus_one().is_negative());
        assert!(!I256::minus_one().is_zero());

        assert_eq!(I256::zero().signum(), I256::zero());
        assert!(!I256::zero().is_positive());
        assert!(!I256::zero().is_negative());
        assert!(I256::zero().is_zero());

        assert_eq!(
            I256::from_dec_str("314159265358979323846264338327950288419716").unwrap().signum(),
            I256::one(),
        );
        assert_eq!(
            I256::from_dec_str("-314159265358979323846264338327950288419716").unwrap().signum(),
            I256::minus_one(),
        );
    }

    #[test]
    fn abs() {
        let positive =
            I256::from_dec_str("314159265358979323846264338327950288419716").unwrap().signum();
        let negative = -positive;

        assert_eq!(positive.abs(), positive);
        assert_eq!(negative.abs(), positive);

        assert_eq!(I256::zero().abs(), I256::zero());
        assert_eq!(I256::MAX.abs(), I256::MAX);
        assert_eq!((-I256::MAX).abs(), I256::MAX);
        assert_eq!(I256::MIN.checked_abs(), None);
    }

    #[test]
    fn neg() {
        let positive =
            I256::from_dec_str("314159265358979323846264338327950288419716").unwrap().signum();
        let negative = -positive;

        assert_eq!(-positive, negative);
        assert_eq!(-negative, positive);

        assert_eq!(-I256::zero(), I256::zero());
        assert_eq!(-(-I256::MAX), I256::MAX);
        assert_eq!(I256::MIN.checked_neg(), None);
    }

    #[test]
    fn bits() {
        assert_eq!(I256::from(0b1000).bits(), 5);
        assert_eq!(I256::from(-0b1000).bits(), 4);

        assert_eq!(I256::from(i64::MAX).bits(), 64);
        assert_eq!(I256::from(i64::MIN).bits(), 64);

        assert_eq!(I256::MAX.bits(), 256);
        assert_eq!(I256::MIN.bits(), 256);

        assert_eq!(I256::zero().bits(), 0);
    }

    #[test]
    fn bit_shift() {
        assert_eq!(I256::one() << 255, I256::MIN);
        assert_eq!(I256::MIN >> 255, I256::one());
    }

    #[test]
    fn arithmetic_shift_right() {
        let value = I256::from_raw(U256::from(2u8).pow(U256::from(254u8))).neg();
        let expected_result = I256::from_raw(U256::MAX.sub(1u8));
        assert_eq!(value.asr(253u32), expected_result, "1011...1111 >> 253 was not 1111...1110");

        let value = I256::minus_one();
        let expected_result = I256::minus_one();
        assert_eq!(value.asr(250u32), expected_result, "-1 >> any_amount was not -1");

        let value = I256::from_raw(U256::from(2u8).pow(U256::from(254u8))).neg();
        let expected_result = I256::minus_one();
        assert_eq!(value.asr(255u32), expected_result, "1011...1111 >> 255 was not -1");

        let value = I256::from_raw(U256::from(2u8).pow(U256::from(254u8))).neg();
        let expected_result = I256::minus_one();
        assert_eq!(value.asr(1024u32), expected_result, "1011...1111 >> 1024 was not -1");

        let value = I256::from(1024i32);
        let expected_result = I256::from(32i32);
        assert_eq!(value.asr(5u32), expected_result, "1024 >> 5 was not 32");

        let value = I256::MAX;
        let expected_result = I256::zero();
        assert_eq!(value.asr(255u32), expected_result, "I256::MAX >> 255 was not 0");

        let value = I256::from_raw(U256::from(2u8).pow(U256::from(254u8))).neg();
        let expected_result = value;
        assert_eq!(value.asr(0u32), expected_result, "1011...1111 >> 0 was not 1011...111");
    }

    #[test]
    fn arithmetic_shift_left() {
        let value = I256::minus_one();
        let expected_result = Some(value);
        assert_eq!(value.asl(0u32), expected_result, "-1 << 0 was not -1");

        let value = I256::minus_one();
        let expected_result = None;
        assert_eq!(
            value.asl(256u32),
            expected_result,
            "-1 << 256 did not overflow (result should be 0000...0000)"
        );

        let value = I256::minus_one();
        let expected_result = Some(I256::from_raw(U256::from(2u8).pow(U256::from(255u8))));
        assert_eq!(value.asl(255u32), expected_result, "-1 << 255 was not 1000...0000");

        let value = I256::from(-1024i32);
        let expected_result = Some(I256::from(-32768i32));
        assert_eq!(value.asl(5u32), expected_result, "-1024 << 5 was not -32768");

        let value = I256::from(1024i32);
        let expected_result = Some(I256::from(32768i32));
        assert_eq!(value.asl(5u32), expected_result, "1024 << 5 was not 32768");

        let value = I256::from(1024i32);
        let expected_result = None;
        assert_eq!(
            value.asl(245u32),
            expected_result,
            "1024 << 245 did not overflow (result should be 1000...0000)"
        );

        let value = I256::zero();
        let expected_result = Some(value);
        assert_eq!(value.asl(1024u32), expected_result, "0 << anything was not 0");
    }

    #[test]
    fn addition() {
        assert_eq!(I256::MIN.overflowing_add(I256::MIN), (I256::zero(), true));
        assert_eq!(I256::MAX.overflowing_add(I256::MAX), (I256::from(-2), true));

        assert_eq!(I256::MIN.overflowing_add(I256::minus_one()), (I256::MAX, true));
        assert_eq!(I256::MAX.overflowing_add(I256::one()), (I256::MIN, true));

        assert_eq!(I256::MAX + I256::MIN, I256::minus_one());
        assert_eq!(I256::from(2) + I256::from(40), I256::from(42));

        assert_eq!(I256::zero() + I256::zero(), I256::zero());

        assert_eq!(I256::MAX.saturating_add(I256::MAX), I256::MAX);
        assert_eq!(I256::MIN.saturating_add(I256::minus_one()), I256::MIN);
    }

    #[test]
    #[allow(clippy::eq_op)]
    fn subtraction() {
        assert_eq!(I256::MIN.overflowing_sub(I256::MAX), (I256::one(), true));
        assert_eq!(I256::MAX.overflowing_sub(I256::MIN), (I256::minus_one(), true));

        assert_eq!(I256::MIN.overflowing_sub(I256::one()), (I256::MAX, true));
        assert_eq!(I256::MAX.overflowing_sub(I256::minus_one()), (I256::MIN, true));

        assert_eq!(I256::zero().overflowing_sub(I256::MIN), (I256::MIN, true));

        assert_eq!(I256::MAX - I256::MAX, I256::zero());
        assert_eq!(I256::from(2) - I256::from(44), I256::from(-42));

        assert_eq!(I256::zero() - I256::zero(), I256::zero());

        assert_eq!(I256::MAX.saturating_sub(I256::MIN), I256::MAX);
        assert_eq!(I256::MIN.saturating_sub(I256::one()), I256::MIN);
    }

    #[test]
    fn multiplication() {
        assert_eq!(I256::MIN.overflowing_mul(I256::MAX), (I256::MIN, true));
        assert_eq!(I256::MAX.overflowing_mul(I256::MIN), (I256::MIN, true));

        assert_eq!(I256::MIN * I256::one(), I256::MIN);
        assert_eq!(I256::from(2) * I256::from(-21), I256::from(-42));

        assert_eq!(I256::MAX.saturating_mul(I256::MAX), I256::MAX);
        assert_eq!(I256::MAX.saturating_mul(I256::from(2)), I256::MAX);
        assert_eq!(I256::MIN.saturating_mul(I256::from(-2)), I256::MAX);

        assert_eq!(I256::MIN.saturating_mul(I256::MAX), I256::MIN);
        assert_eq!(I256::MIN.saturating_mul(I256::from(2)), I256::MIN);
        assert_eq!(I256::MAX.saturating_mul(I256::from(-2)), I256::MIN);

        assert_eq!(I256::zero() * I256::zero(), I256::zero());
        assert_eq!(I256::one() * I256::zero(), I256::zero());
        assert_eq!(I256::MAX * I256::zero(), I256::zero());
        assert_eq!(I256::MIN * I256::zero(), I256::zero());
    }

    #[test]
    fn division() {
        // The only case for overflow.
        assert_eq!(I256::MIN.overflowing_div(I256::from(-1)), (I256::MIN, true));

        assert_eq!(I256::MIN / I256::MAX, I256::from(-1));
        assert_eq!(I256::MAX / I256::MIN, I256::zero());

        assert_eq!(I256::MIN / I256::one(), I256::MIN);
        assert_eq!(I256::from(-42) / I256::from(-21), I256::from(2));
        assert_eq!(I256::from(-42) / I256::from(2), I256::from(-21));
        assert_eq!(I256::from(42) / I256::from(-21), I256::from(-2));
        assert_eq!(I256::from(42) / I256::from(21), I256::from(2));

        // The only saturating corner case.
        assert_eq!(I256::MIN.saturating_div(I256::from(-1)), I256::MAX);
    }

    #[test]
    #[should_panic]
    fn division_by_zero() {
        let _ = I256::one() / I256::zero();
    }

    #[test]
    fn div_euclid() {
        let a = I256::from(7);
        let b = I256::from(4);

        assert_eq!(a.div_euclid(b), I256::one()); // 7 >= 4 * 1
        assert_eq!(a.div_euclid(-b), -I256::one()); // 7 >= -4 * -1
        assert_eq!((-a).div_euclid(b), -I256::from(2)); // -7 >= 4 * -2
        assert_eq!((-a).div_euclid(-b), I256::from(2)); // -7 >= -4 * 2

        // Overflowing
        assert_eq!(I256::MIN.overflowing_div_euclid(-I256::one()), (I256::MIN, true));
        // Wrapping
        assert_eq!(I256::MIN.wrapping_div_euclid(-I256::one()), I256::MIN);
        // // Checked
        assert_eq!(I256::MIN.checked_div_euclid(-I256::one()), None);
        assert_eq!(I256::one().checked_div_euclid(I256::zero()), None);
    }

    #[test]
    fn rem_euclid() {
        let a = I256::from(7); // or any other integer type
        let b = I256::from(4);

        assert_eq!(a.rem_euclid(b), I256::from(3));
        assert_eq!((-a).rem_euclid(b), I256::one());
        assert_eq!(a.rem_euclid(-b), I256::from(3));
        assert_eq!((-a).rem_euclid(-b), I256::one());

        // Overflowing
        assert_eq!(a.overflowing_rem_euclid(b), (I256::from(3), false));
        assert_eq!(I256::min_value().overflowing_rem_euclid(-I256::one()), (I256::zero(), true));

        // Wrapping
        assert_eq!(I256::from(100).wrapping_rem_euclid(I256::from(10)), I256::zero());
        assert_eq!(I256::min_value().wrapping_rem_euclid(-I256::one()), I256::zero());

        // Checked
        assert_eq!(a.checked_rem_euclid(b), Some(I256::from(3)));
        assert_eq!(a.checked_rem_euclid(I256::zero()), None);
        assert_eq!(I256::min_value().checked_rem_euclid(-I256::one()), None);
    }

    #[test]
    #[should_panic]
    fn div_euclid_by_zero() {
        let _ = I256::one().div_euclid(I256::zero());
        assert_eq!(I256::MIN.div_euclid(-I256::one()), I256::MAX);
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn div_euclid_overflow() {
        let _ = I256::MIN.div_euclid(-I256::one());
    }

    #[test]
    #[should_panic]
    fn mod_by_zero() {
        let _ = I256::one() % I256::zero();
    }

    #[test]
    fn remainder() {
        // The only case for overflow.
        assert_eq!(I256::MIN.overflowing_rem(I256::from(-1)), (I256::zero(), true));
        assert_eq!(I256::from(-5) % I256::from(-2), I256::from(-1));
        assert_eq!(I256::from(5) % I256::from(-2), I256::one());
        assert_eq!(I256::from(-5) % I256::from(2), I256::from(-1));
        assert_eq!(I256::from(5) % I256::from(2), I256::one());

        assert_eq!(I256::MIN.checked_rem(I256::from(-1)), None);
        assert_eq!(I256::one().checked_rem(I256::one()), Some(I256::zero()));
    }

    #[test]
    fn exponentiation() {
        assert_eq!(I256::from(1000).saturating_pow(1000), I256::MAX);
        assert_eq!(I256::from(-1000).saturating_pow(1001), I256::MIN);

        assert_eq!(I256::from(2).pow(64), I256::from(1u128 << 64));
        assert_eq!(I256::from(-2).pow(63), I256::from(i64::MIN));

        assert_eq!(I256::zero().pow(42), I256::zero());
        assert_eq!(I256::exp10(18).to_string(), "1000000000000000000");
    }

    #[test]
    fn iterators() {
        assert_eq!((1..=5).map(I256::from).sum::<I256>(), I256::from(15));
        assert_eq!((1..=5).map(I256::from).product::<I256>(), I256::from(120));
    }

    #[test]
    fn tokenization() {
        assert_eq!(json!(I256::from(42)), json!("0x2a"));
        assert_eq!(json!(I256::minus_one()), json!(U256::MAX));

        assert_eq!(I256::from(42).into_token(), 42i32.into_token());
        assert_eq!(I256::minus_one().into_token(), Token::Int(U256::MAX),);

        assert_eq!(I256::from_token(42i32.into_token()).unwrap(), I256::from(42),);
        assert_eq!(I256::from_token(U256::MAX.into_token()).unwrap(), I256::minus_one(),);
    }

    #[test]
    fn twos_complement() {
        macro_rules! assert_twos_complement {
            ($signed:ty, $unsigned:ty) => {
                assert_eq!(
                    I256::from(<$signed>::MAX).twos_complement(),
                    U256::from(<$signed>::MAX)
                );
                assert_eq!(
                    I256::from(<$signed>::MIN).twos_complement(),
                    U256::from(<$signed>::MIN.unsigned_abs())
                );
                assert_eq!(I256::from(0 as $signed).twos_complement(), U256::from(0 as $signed));

                assert_eq!(
                    I256::from(<$unsigned>::MAX).twos_complement(),
                    U256::from(<$unsigned>::MAX)
                );
                assert_eq!(
                    I256::from(0 as $unsigned).twos_complement(),
                    U256::from(0 as $unsigned)
                );
            };
        }

        assert_twos_complement!(i8, u8);
        assert_twos_complement!(i16, u16);
        assert_twos_complement!(i32, u32);
        assert_twos_complement!(i64, u64);
        assert_twos_complement!(i128, u128);
        assert_twos_complement!(isize, usize);
    }
}
