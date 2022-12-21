use ethers_core::{types::U256, utils::format_units};
use std::ops::{Add, Div, Mul, Sub};

fn main() {
    let a = U256::from(100);
    let b = U256::from(10);

    // a + b
    let sum = a.add(b);
    assert!(sum == U256::from(110));

    // a - b
    let diff = a.sub(b);
    assert!(diff == U256::from(90));

    // a / b
    let div = a.div(b);
    assert!(div == U256::from(10));

    // a * b
    let mul = a.mul(b);
    assert!(mul == U256::from(1000));

    // a % b
    let module = a.div_mod(b).1;
    assert!(module == U256::zero());

    // a ^ b
    let base = U256::from(10);
    let expon = U256::from(2);
    let pow = base.pow(expon);
    assert!(pow == U256::from(100));

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
