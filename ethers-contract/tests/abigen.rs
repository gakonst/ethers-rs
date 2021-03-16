//! Test cases to validate the `abigen!` macro
use ethers_contract::{abigen, EthEvent};
use ethers_core::abi::Tokenizable;

#[test]
fn can_gen_human_readable() {
    abigen!(
        SimpleContract,
        r#"[
        event ValueChanged(address indexed author, string oldValue, string newValue)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize)
    );
    assert_eq!("ValueChanged", ValueChangedFilter::name());
    assert_eq!(
        "ValueChanged(address,string,string)",
        ValueChangedFilter::abi_signature()
    );
}

#[test]
fn can_gen_structs_readable() {
    abigen!(
        SimpleContract,
        r#"[
        struct Value {address addr; string value;}
        struct Addresses {address[] addr; string s;}
        event ValueChanged(Value indexed old, Value newValue, Addresses _a)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize)
    );
    let value = Addresses {
        addr: vec!["eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap()],
        s: "hello".to_string(),
    };
    let token = value.clone().into_token();
    assert_eq!(value, Addresses::from_token(token).unwrap());

    assert_eq!("ValueChanged", ValueChangedFilter::name());
    assert_eq!(
        "ValueChanged((address,string),(address,string),(address[],string))",
        ValueChangedFilter::abi_signature()
    );
}

// NOTE(mattsse): There is currently a limitation with the `ethabi` crate's `Reader`
//  that doesn't support arrays of tuples; https://github.com/gakonst/ethabi/pull/1 should fix this
// See also https://github.com/rust-ethereum/ethabi/issues/178 and
// https://github.com/rust-ethereum/ethabi/pull/186

// #[test]
// fn can_gen_structs_with_arrays_readable() {
//     abigen!(
//         SimpleContract,
//         r#"[
//         struct Value {address addr; string value;}
//         struct Addresses {address[] addr; string s;}
//         event ValueChanged(Value indexed old, Value newValue, Addresses[] _a)
//     ]"#,
//         event_derives(serde::Deserialize, serde::Serialize)
//     );
//     assert_eq!(
//         "ValueChanged((address,string),(address,string),(address[],string)[])",
//         ValueChangedFilter::abi_signature()
//     );
// }
