use ethers_core::{types::U256, utils::format_units};

fn main() {
    let num = U256::from(42_u8);

    let a: u128 = num.as_u128();
    assert_eq!(a, 42);

    let b: u64 = num.as_u64();
    assert_eq!(b, 42);

    let c: u32 = num.as_u32();
    assert_eq!(c, 42);

    let d: usize = num.as_usize();
    assert_eq!(d, 42);

    let e: String = num.to_string();
    assert_eq!(e, "42");

    let f: String = format_units(num, 4).unwrap();
    assert_eq!(f, "0.0042");
}
