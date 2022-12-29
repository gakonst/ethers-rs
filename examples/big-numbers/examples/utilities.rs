use ethers::{
    types::U256,
    utils::{format_units, parse_units, ParseUnits},
};

fn main() {
    parse_units_example();
    format_units_example();
}

/// DApps business logics handles big numbers in 'wei' units (i.e. sending transactions, on-chain
/// math, etc.). We provide convenient methods to map user inputs (usually in 'ether' or 'gwei')
/// into 'wei' format.
fn parse_units_example() {
    let pu: ParseUnits = parse_units("1.0", "wei").unwrap();
    let num = U256::from(pu);
    assert_eq!(num, U256::one());

    let pu: ParseUnits = parse_units("1.0", "kwei").unwrap();
    let num = U256::from(pu);
    assert_eq!(num, U256::from(1000));

    let pu: ParseUnits = parse_units("1.0", "mwei").unwrap();
    let num = U256::from(pu);
    assert_eq!(num, U256::from(1000000));

    let pu: ParseUnits = parse_units("1.0", "gwei").unwrap();
    let num = U256::from(pu);
    assert_eq!(num, U256::from(1000000000));

    let pu: ParseUnits = parse_units("1.0", "szabo").unwrap();
    let num = U256::from(pu);
    assert_eq!(num, U256::from(1000000000000_u128));

    let pu: ParseUnits = parse_units("1.0", "finney").unwrap();
    let num = U256::from(pu);
    assert_eq!(num, U256::from(1000000000000000_u128));

    let pu: ParseUnits = parse_units("1.0", "ether").unwrap();
    let num = U256::from(pu);
    assert_eq!(num, U256::from(1000000000000000000_u128));

    let pu: ParseUnits = parse_units("1.0", 18).unwrap();
    let num = U256::from(pu);
    assert_eq!(num, U256::from(1000000000000000000_u128));
}

/// DApps business logics handles big numbers in 'wei' units (i.e. sending transactions, on-chain
/// math, etc.). On the other hand it is useful to convert big numbers into user readable formats
/// when displaying on a UI. Generally dApps display numbers in 'ether' and 'gwei' units,
/// respectively for displaying amounts and gas. The `format_units` function will format a big
/// number into a user readable string.
fn format_units_example() {
    // 1 ETHER = 10^18 WEI
    let one_ether = U256::from(1000000000000000000_u128);

    let num: String = format_units(one_ether, "wei").unwrap();
    assert_eq!(num, "1000000000000000000.0");

    let num: String = format_units(one_ether, "gwei").unwrap();
    assert_eq!(num, "1000000000.000000000");

    let num: String = format_units(one_ether, "ether").unwrap();
    assert_eq!(num, "1.000000000000000000");

    // 1 GWEI = 10^9 WEI
    let one_gwei = U256::from(1000000000_u128);

    let num: String = format_units(one_gwei, 0).unwrap();
    assert_eq!(num, "1000000000.0");

    let num: String = format_units(one_gwei, "wei").unwrap();
    assert_eq!(num, "1000000000.0");

    let num: String = format_units(one_gwei, "kwei").unwrap();
    assert_eq!(num, "1000000.000");

    let num: String = format_units(one_gwei, "mwei").unwrap();
    assert_eq!(num, "1000.000000");

    let num: String = format_units(one_gwei, "gwei").unwrap();
    assert_eq!(num, "1.000000000");

    let num: String = format_units(one_gwei, "szabo").unwrap();
    assert_eq!(num, "0.001000000000");

    let num: String = format_units(one_gwei, "finney").unwrap();
    assert_eq!(num, "0.000001000000000");

    let num: String = format_units(one_gwei, "ether").unwrap();
    assert_eq!(num, "0.000000001000000000");
}
