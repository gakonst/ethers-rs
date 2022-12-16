# Big numbers
Ethereum uses big numbers (also known as "bignums" or "arbitrary-precision integers") to represent certain values in its codebase and in blockchain transactions. This is necessary because Ethereum uses a 256-bit numbering system, which is much larger than the number systems typically used in modern computers (such as base-10 or base-2). Using big numbers allows Ethereum to handle very large numbers, such as those that may be required to represent very large balances or quantities of a particular asset.

It is worth noting that Ethereum is not the only blockchain or cryptocurrency that uses big numbers. Many other blockchains and cryptocurrencies also use big numbers to represent values in their respective systems.

## Comparison and equivalence

```rust
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
```

## Conversion

```rust
use ethers::{types::U256, utils::format_units};

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
    assert_eq!(e, "42".to_string());

    let f: String = format_units(num, 4).unwrap();
    assert_eq!(f, "0.0042".to_string());
}
```

## Create instances
```rust
use ethers::{types::{serde_helpers::Numeric, U256}, utils::{parse_units, ParseUnits}};

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
```

## Math operations
```rust
use std::ops::{Add, Div, Mul, Sub};

use ethers::{types::U256, utils::format_units};

fn main() {
    let a = U256::from(100);
    let b = U256::from(10);

    // a + b
    let sum = a.add(b);
    assert!(sum.eq(&U256::from(110)));

    // a - b
    let diff = a.sub(b);
    assert!(diff.eq(&U256::from(90)));

    // a / b
    let div = a.div(b);
    assert!(div.eq(&U256::from(10)));

    // a * b
    let mul = a.mul(b);
    assert!(mul.eq(&U256::from(1000)));

    // a % b
    let module = a.div_mod(b).1;
    assert!(module.eq(&U256::zero()));

    // a ^ b
    let base = U256::from(10);
    let expon = U256::from(2);
    let pow = base.pow(expon);
    assert!(pow.eq(&U256::from(100)));

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
    assert_eq!(&*s, "200.000000000000000000"); // 200
}
```

## Utilities
In order to create an application, it is often necessary to convert between the representation of values that is easily understood by humans (such as ether) and the machine-readable form that is used by contracts and math functions (such as wei). This is particularly important when working with Ethereum, as certain values, such as balances and gas prices, must be expressed in wei when sending transactions, even if they are displayed to the user in a different format, such as ether or gwei. To help with this conversion, ethers-rs provides two functions, `parse_units` and `format_units`, which allow you to easily convert between human-readable and machine-readable forms of values. `parse_units` can be used to convert a string representing a value in ether, such as "1.1", into a big number in wei, which can be used in contracts and math functions. `format_units` can be used to convert a big number value into a human-readable string, which is useful for displaying values to users.

### `parse_units`
```rust
use ethers::{types::U256, utils::{parse_units, ParseUnits}};

fn main() {
    let pu: ParseUnits = parse_units("1.0", "wei").unwrap();
    let num = U256::from(pu);
    assert!(num.eq(&U256::from(1)));

    let pu: ParseUnits = parse_units("1.0", "kwei").unwrap();
    let num = U256::from(pu);
    assert!(num.eq(&U256::from(1000)));

    let pu: ParseUnits = parse_units("1.0", "mwei").unwrap();
    let num = U256::from(pu);
    assert!(num.eq(&U256::from(1000000)));

    let pu: ParseUnits = parse_units("1.0", "gwei").unwrap();
    let num = U256::from(pu);
    assert!(num.eq(&U256::from(1000000000)));

    let pu: ParseUnits = parse_units("1.0", "szabo").unwrap();
    let num = U256::from(pu);
    assert!(num.eq(&U256::from(1000000000000_u128)));

    let pu: ParseUnits = parse_units("1.0", "finney").unwrap();
    let num = U256::from(pu);
    assert!(num.eq(&U256::from(1000000000000000_u128)));

    let pu: ParseUnits = parse_units("1.0", "ether").unwrap();
    let num = U256::from(pu);
    assert!(num.eq(&U256::from(1000000000000000000_u128)));

    let pu: ParseUnits = parse_units("1.0", 18).unwrap();
    let num = U256::from(pu);
    assert!(num.eq(&U256::from(1000000000000000000_u128)));

}
```

### `format_units`
```rust
use ethers::{types::U256, utils::format_units};

fn format_units_example() {
    // 1 ETHER = 10^18 WEI
    let one_ether = U256::from(1000000000000000000_u128);

    let num: String = format_units(one_ether, "wei").unwrap();
    assert_eq!(&*num, "1000000000000000000.0");

    let num: String = format_units(one_ether, "gwei").unwrap();
    assert_eq!(&*num, "1000000000.000000000");

    let num: String = format_units(one_ether, "ether").unwrap();
    assert_eq!(&*num, "1.000000000000000000");

    // 1 GWEI = 10^9 WEI
    let one_gwei = U256::from(1000000000_u128);

    let num: String = format_units(one_gwei, 0).unwrap();
    assert_eq!(&*num, "1000000000.0");

    let num: String = format_units(one_gwei, "wei").unwrap();
    assert_eq!(&*num, "1000000000.0");

    let num: String = format_units(one_gwei, "kwei").unwrap();
    assert_eq!(&*num, "1000000.000");

    let num: String = format_units(one_gwei, "mwei").unwrap();
    assert_eq!(&*num, "1000.000000");

    let num: String = format_units(one_gwei, "gwei").unwrap();
    assert_eq!(&*num, "1.000000000");

    let num: String = format_units(one_gwei, "szabo").unwrap();
    assert_eq!(&*num, "0.001000000000");

    let num: String = format_units(one_gwei, "finney").unwrap();
    assert_eq!(&*num, "0.000001000000000");

    let num: String = format_units(one_gwei, "ether").unwrap();
    assert_eq!(&*num, "0.000000001000000000");
}
```

