use ethers::types::U256;

fn main() {
    // a == b
    let a = U256::from(100_u32);
    let b = U256::from(100_u32);
    assert!(a.eq(&b));

    // a < b
    let a = U256::from(1_u32);
    let b = U256::from(100_u32);
    assert!(a.lt(&b));

    // a <= b
    let a = U256::from(100_u32);
    let b = U256::from(100_u32);
    assert!(a.le(&b));

    // a > b
    let a = U256::from(100_u32);
    let b = U256::from(1_u32);
    assert!(a.gt(&b));

    // a >= b
    let a = U256::from(100_u32);
    let b = U256::from(100_u32);
    assert!(a.ge(&b));

    // a == 0
    let a = U256::zero();
    assert!(a.is_zero());
}
