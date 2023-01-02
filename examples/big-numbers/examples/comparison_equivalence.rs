use ethers::types::U256;

/// `U256` implements traits in `std::cmp`, that means `U256` instances
/// can be easily compared using standard Rust operators.
fn main() {
    // a == b
    let a = U256::from(100_u32);
    let b = U256::from(100_u32);
    assert!(a == b);

    // a < b
    let a = U256::from(1_u32);
    let b = U256::from(100_u32);
    assert!(a < b);

    // a <= b
    let a = U256::from(100_u32);
    let b = U256::from(100_u32);
    assert!(a <= b);

    // a > b
    let a = U256::from(100_u32);
    let b = U256::from(1_u32);
    assert!(a > b);

    // a >= b
    let a = U256::from(100_u32);
    let b = U256::from(100_u32);
    assert!(a >= b);

    // a == 0
    let a = U256::zero();
    assert!(a.is_zero());
}
