use ethers_core::{types::U256, utils::format_units};
use std::ops::{Div, Mul};

fn main() {
    let a = U256::from(10);
    let b = U256::from(2);

    // addition
    let sum = a + b;
    assert_eq!(sum, U256::from(12));

    // subtraction
    let difference = a - b;
    assert_eq!(difference, U256::from(8));

    // multiplication
    let product = a * b;
    assert_eq!(product, U256::from(20));

    // division
    let quotient = a / b;
    assert_eq!(quotient, U256::from(5));

    // modulo
    let remainder = a % b;
    assert_eq!(remainder, U256::zero()); // equivalent to `U256::from(0)`

    // exponentiation
    let power = a.pow(b);
    assert_eq!(power, U256::from(100));
    // powers of 10 can also be expressed like this:
    let power_of_10 = U256::exp10(2);
    assert_eq!(power_of_10, U256::from(100));

    // Multiply two 'ether' numbers:
    // Big numbers are integers, that can represent fixed point numbers.
    // For instance, 1 ether has 18 fixed
    // decimal places (1.000000000000000000), and its big number
    // representation is 10^18 = 1000000000000000000.
    // When we multiply such numbers we are summing up their exponents.
    // So if we multiply 10^18 * 10^18 we get 10^36, that is obviously incorrect.
    // In order to get the correct result we need to divide by 10^18.
    let eth1 = U256::from(10_000000000000000000_u128); // 10 ether
    let eth2 = U256::from(20_000000000000000000_u128); // 20 ether
    let base = U256::from(10).pow(18.into());
    let mul = eth1.mul(eth2).div(base); // We also divide by 10^18
    let s: String = format_units(mul, "ether").unwrap();
    assert_eq!(s, "200.000000000000000000"); // 200
}
