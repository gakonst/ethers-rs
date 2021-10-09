#![cfg(feature = "abigen")]
//! Test cases to validate the `abigen!` macro
use ethers_contract::{abigen, EthEvent};
use ethers_core::abi::{Address, Tokenizable};
use ethers_providers::Provider;
use std::sync::Arc;

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

#[test]
fn can_gen_structs_with_arrays_readable() {
    abigen!(
        SimpleContract,
        r#"[
        struct Value {address addr; string value;}
        struct Addresses {address[] addr; string s;}
        event ValueChanged(Value indexed old, Value newValue, Addresses[] _a)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize)
    );
    assert_eq!(
        "ValueChanged((address,string),(address,string),(address[],string)[])",
        ValueChangedFilter::abi_signature()
    );
}

fn assert_tokenizeable<T: Tokenizable>() {}

#[test]
fn can_generate_internal_structs() {
    abigen!(
        VerifierContract,
        "ethers-contract/tests/solidity-contracts/verifier_abi.json",
        event_derives(serde::Deserialize, serde::Serialize)
    );
    assert_tokenizeable::<VerifyingKey>();
    assert_tokenizeable::<G1Point>();
    assert_tokenizeable::<G2Point>();
}

#[test]
fn can_gen_human_readable_with_structs() {
    abigen!(
        SimpleContract,
        r#"[
        struct Foo { uint256 x; }
        function foo(Foo memory x)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize)
    );
    assert_tokenizeable::<Foo>();

    let (client, _mock) = Provider::mocked();
    let contract = SimpleContract::new(Address::default(), Arc::new(client));
    let f = Foo { x: 100u64.into() };
    let _ = contract.foo(f);
}
