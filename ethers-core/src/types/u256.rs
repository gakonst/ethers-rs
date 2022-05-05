use ethabi::ethereum_types::U256;

/// Convert a floating point value to its nearest f64 integer.
///
/// It is saturating, so values $\ge 2^{256}$ will be rounded
/// to [`U256::max_value()`] and values $< 0$ to zero. This includes
/// positive and negative infinity.
///
/// TODO: Move to ethabi::ethereum_types::U256.
/// TODO: Add [`super::I256`] version.
///
/// # Panics
///
/// Panics if `f` is NaN.
pub fn u256_from_f64_saturating(mut f: f64) -> U256 {
    if f.is_nan() {
        panic!("NaN is not a valid value for U256");
    }
    if f < 0.5 {
        return U256::zero()
    }
    if f >= 1.157_920_892_373_162e77_f64 {
        return U256::max_value()
    }
    // All non-normal cases should have been handled above
    assert!(f.is_normal());
    // Turn nearest rounding into truncated rounding
    f += 0.5;

    // Parse IEEE-754 double into U256
    // Sign should be zero, exponent should be >= 0.
    let bits = f.to_bits();
    let sign = bits >> 63;
    assert!(sign == 0);
    let biased_exponent = (bits >> 52) & 0x7ff;
    assert!(biased_exponent >= 1023);
    let exponent = biased_exponent - 1023;
    let fraction = bits & 0xfffffffffffff;
    let mantissa = 0x10000000000000 | fraction;
    if exponent > 255 {
        U256::max_value()
    } else if exponent < 52 {
        // Truncate mantissa
        U256([mantissa, 0, 0, 0]) >> (52 - exponent)
    } else {
        U256([mantissa, 0, 0, 0]) << (exponent - 52)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64;

    #[test]
    fn test_small_integers() {
        for i in 0..=255 {
            let f = i as f64;
            let u = u256_from_f64_saturating(f);
            assert_eq!(u, U256::from(i));
        }
    }

    #[test]
    fn test_small_integers_round_down() {
        for i in 0..=255 {
            let f = (i as f64) + 0.499;
            let u = u256_from_f64_saturating(f);
            assert_eq!(u, U256::from(i));
        }
    }

    #[test]
    fn test_small_integers_round_up() {
        for i in 0..=255 {
            let f = (i as f64) - 0.5;
            let u = u256_from_f64_saturating(f);
            assert_eq!(u, U256::from(i));
        }
    }

    #[test]
    fn test_infinities() {
        assert_eq!(u256_from_f64_saturating(f64::INFINITY), U256::max_value());
        assert_eq!(u256_from_f64_saturating(f64::NEG_INFINITY), U256::zero());
    }

    #[test]
    fn test_saturating() {
        assert_eq!(u256_from_f64_saturating(-1.0), U256::zero());
        assert_eq!(u256_from_f64_saturating(1e90_f64), U256::max_value());
    }

    #[test]
    fn test_large() {
        // Check with e.g. `python3 -c 'print(int(1.0e36))'`
        assert_eq!(
            u256_from_f64_saturating(1.0e36_f64),
            U256::from_dec_str("1000000000000000042420637374017961984").unwrap()
        );
        assert_eq!(
            u256_from_f64_saturating(f64::consts::PI * 2.0e60_f64),
            U256::from_dec_str("6283185307179586084560863929317662625677330590403879287914496")
                .unwrap()
        );
        assert_eq!(
            u256_from_f64_saturating(5.78960446186581e76_f64),
            U256::from_dec_str(
                "57896044618658097711785492504343953926634992332820282019728792003956564819968"
            )
            .unwrap()
        );
        assert_eq!(
            u256_from_f64_saturating(1.157920892373161e77_f64),
            U256::from_dec_str(
                "115792089237316105435040506505232477503392813560534822796089932171514352762880"
            )
            .unwrap()
        );
    }
}
