use ethers::{
    types::{serde_helpers::Numeric, U256},
    utils::{parse_units, ParseUnits},
};

fn main() {
    // From strings
    let a = U256::from_dec_str("42").unwrap();
    assert_eq!(&*format!("{a:?}"), "42");

    let amount = "42";
    let units = 4;
    let pu: ParseUnits = parse_units(amount, units).unwrap();
    let b = U256::from(pu);
    assert_eq!(&*format!("{b:?}"), "420000");

    // From numbers
    let c = U256::from(42_u8);
    assert_eq!(&*format!("{c:?}"), "42");

    let d = U256::from(42_u16);
    assert_eq!(&*format!("{d:?}"), "42");

    let e = U256::from(42_u32);
    assert_eq!(&*format!("{e:?}"), "42");

    let f = U256::from(42_u64);
    assert_eq!(&*format!("{f:?}"), "42");

    let g = U256::from(42_u128);
    assert_eq!(&*format!("{g:?}"), "42");

    let h = U256::from(0x2a);
    assert_eq!(&*format!("{h:?}"), "42");

    let i: U256 = 42.into();
    assert_eq!(&*format!("{i:?}"), "42");

    // From `Numeric`
    let num: Numeric = Numeric::U256(U256::one());
    let l = U256::from(num);
    assert_eq!(&*format!("{l:?}"), "1");

    let num: Numeric = Numeric::Num(42);
    let m = U256::from(num);
    assert_eq!(&*format!("{m:?}"), "42");
}
