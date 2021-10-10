#![cfg(feature = "abigen")]
//! Test cases to validate the `abigen!` macro
use ethers_contract::{abigen, EthEvent};
use ethers_core::abi::{Address, Tokenizable};
use ethers_core::types::U256;
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
fn can_gen_human_readable_multiple() {
    abigen!(
        SimpleContract1,
        r#"[
        event ValueChanged1(address indexed author, string oldValue, string newValue)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize);

        SimpleContract2,
        r#"[
        event ValueChanged2(address indexed author, string oldValue, string newValue)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize)
    );
    assert_eq!("ValueChanged1", ValueChanged1Filter::name());
    assert_eq!(
        "ValueChanged1(address,string,string)",
        ValueChanged1Filter::abi_signature()
    );
    assert_eq!("ValueChanged2", ValueChanged2Filter::name());
    assert_eq!(
        "ValueChanged2(address,string,string)",
        ValueChanged2Filter::abi_signature()
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
fn can_generate_internal_structs_multiple() {
    // NOTE: nesting here is necessary due to how tests are structured...
    use contract::*;
    mod contract {
        use super::*;
        abigen!(
            VerifierContract,
            "ethers-contract/tests/solidity-contracts/verifier_abi.json",
            event_derives(serde::Deserialize, serde::Serialize);

            MyOtherVerifierContract,
            "ethers-contract/tests/solidity-contracts/verifier_abi.json",
            event_derives(serde::Deserialize, serde::Serialize);
        );
    }
    assert_tokenizeable::<VerifyingKey>();
    assert_tokenizeable::<G1Point>();
    assert_tokenizeable::<G2Point>();

    let (provider, _) = Provider::mocked();
    let client = Arc::new(provider);

    let g1 = G1Point {
        x: U256::zero(),
        y: U256::zero(),
    };
    let g2 = G2Point {
        x: [U256::zero(), U256::zero()],
        y: [U256::zero(), U256::zero()],
    };
    let vk = VerifyingKey {
        alfa_1: g1.clone(),
        beta_2: g2.clone(),
        gamma_2: g2.clone(),
        delta_2: g2.clone(),
        ic: vec![g1.clone()],
    };
    let proof = Proof {
        a: g1.clone(),
        b: g2,
        c: g1,
    };

    // ensure both contracts use the same types
    let contract = VerifierContract::new(Address::zero(), client.clone());
    let _ = contract.verify(vec![], proof.clone(), vk.clone());
    let contract = MyOtherVerifierContract::new(Address::zero(), client);
    let _ = contract.verify(vec![], proof, vk);
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
