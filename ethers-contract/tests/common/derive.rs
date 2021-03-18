use ethers::core::types::{H160, H256, I256, U128, U256};
use ethers_contract::{abigen, EthAbiType, EthEvent};
use ethers_core::abi::Tokenizable;
use ethers_core::types::Address;

#[derive(Debug, Clone, PartialEq, EthAbiType)]
struct ValueChanged {
    old_author: Address,
    new_author: Address,
    old_value: String,
    new_value: String,
}

#[derive(Debug, Clone, PartialEq, EthAbiType)]
struct ValueChangedWrapper {
    inner: ValueChanged,
    msg: String,
}

#[derive(Debug, Clone, PartialEq, EthAbiType)]
struct ValueChangedTuple(Address, Address, String, String);

#[derive(Debug, Clone, PartialEq, EthAbiType)]
struct ValueChangedTupleWrapper(ValueChangedTuple, String);

#[test]
fn can_detokenize_struct() {
    let value = ValueChanged {
        old_author: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap(),
        new_author: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse().unwrap(),
        old_value: "50".to_string(),
        new_value: "100".to_string(),
    };

    let token = value.clone().into_token();
    assert_eq!(value, ValueChanged::from_token(token).unwrap());
}

#[test]
fn can_detokenize_nested_structs() {
    let value = ValueChangedWrapper {
        inner: ValueChanged {
            old_author: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap(),
            new_author: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse().unwrap(),
            old_value: "50".to_string(),
            new_value: "100".to_string(),
        },
        msg: "hello world".to_string(),
    };

    let token = value.clone().into_token();
    assert_eq!(value, ValueChangedWrapper::from_token(token).unwrap());
}

#[test]
fn can_detokenize_tuple_struct() {
    let value = ValueChangedTuple(
        "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap(),
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse().unwrap(),
        "50".to_string(),
        "100".to_string(),
    );

    let token = value.clone().into_token();
    assert_eq!(value, ValueChangedTuple::from_token(token).unwrap());
}

#[test]
fn can_detokenize_nested_tuple_struct() {
    let value = ValueChangedTupleWrapper(
        ValueChangedTuple(
            "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap(),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse().unwrap(),
            "50".to_string(),
            "100".to_string(),
        ),
        "hello world".to_string(),
    );

    let token = value.clone().into_token();
    assert_eq!(value, ValueChangedTupleWrapper::from_token(token).unwrap());
}

#[test]
fn can_derive_eth_event() {
    #[derive(Debug, Clone, PartialEq, EthEvent)]
    struct ValueChangedEvent {
        old_author: Address,
        new_author: Address,
        old_value: String,
        new_value: String,
    }

    let value = ValueChangedEvent {
        old_author: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap(),
        new_author: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse().unwrap(),
        old_value: "50".to_string(),
        new_value: "100".to_string(),
    };

    assert_eq!("ValueChangedEvent", ValueChangedEvent::name());
    assert_eq!(
        "ValueChangedEvent(address,address,string,string)",
        ValueChangedEvent::abi_signature()
    );

    let token = value.clone().into_token();
    assert_eq!(value, ValueChangedEvent::from_token(token).unwrap());
}

#[test]
fn can_set_eth_event_name_attribute() {
    #[derive(Debug, PartialEq, EthEvent)]
    #[ethevent(name = "MyEvent")]
    struct ValueChangedEvent {
        old_author: Address,
        new_author: Address,
        old_value: String,
        new_value: String,
    }

    assert_eq!("MyEvent", ValueChangedEvent::name());
    assert_eq!(
        "MyEvent(address,address,string,string)",
        ValueChangedEvent::abi_signature()
    );
}

#[test]
fn can_detect_various_event_abi_types() {
    #[derive(Debug, PartialEq, EthEvent)]
    struct ValueChangedEvent {
        old_author: Address,
        s: String,
        h1: H256,
        i256: I256,
        u256: U256,
        b: bool,
        v: Vec<Address>,
        bs: Vec<bool>,
        h160: H160,
        u128: U128,
        int8: i8,
        int16: i16,
        int32: i32,
        int64: i64,
        int128: i128,
        uint8: u8,
        uint16: u16,
        uint32: u32,
        uint64: u64,
        uint128: u128,
    }

    assert_eq!(
        "ValueChangedEvent(address,string,bytes32,int256,uint256,bool,address[],bool[],bytes20,uint128,int8,int16,int32,int64,int128,uint8,uint16,uint32,uint64,uint128)",
        ValueChangedEvent::abi_signature()
    );
}

#[test]
fn can_set_eth_abi_attribute() {
    #[derive(Debug, Clone, PartialEq, EthAbiType)]
    struct SomeType {
        inner: Address,
        msg: String,
    }

    #[derive(Debug, PartialEq, EthEvent)]
    #[ethevent(abi = "ValueChangedEvent(address,(address,string),string)")]
    struct ValueChangedEvent {
        old_author: Address,
        inner: SomeType,
        new_value: String,
    }

    assert_eq!(
        "ValueChangedEvent(address,(address,string),string)",
        ValueChangedEvent::abi_signature()
    );

    #[derive(Debug, PartialEq, EthEvent)]
    #[ethevent(
        name = "ValueChangedEvent",
        abi = "ValueChangedEvent(address,(address,string),string)"
    )]
    struct ValueChangedEvent2 {
        old_author: Address,
        inner: SomeType,
        new_value: String,
    }

    assert_eq!(
        "ValueChangedEvent(address,(address,string),string)",
        ValueChangedEvent2::abi_signature()
    );
}

#[test]
fn can_derive_indexed_and_anonymous_attribute() {
    #[derive(Debug, PartialEq, EthEvent)]
    #[ethevent(anonymous)]
    struct ValueChangedEvent {
        old_author: Address,
        #[ethevent(indexed, name = "newAuthor")]
        new_author: Address,
        old_value: String,
        new_value: String,
    }

    assert_eq!(
        "ValueChangedEvent(address,address,string,string) anonymous",
        ValueChangedEvent::abi_signature()
    );
}

#[test]
fn can_generate_ethevent_from_json() {
    abigen!(DsProxyFactory,
        "ethers-middleware/contracts/DsProxyFactory.json",
        methods {
            build(address) as build_with_owner;
        }
    );

    assert_eq!(
        "Created(address,address,address,address)",
        CreatedFilter::abi_signature()
    );

    assert_eq!(
        H256([
            37, 155, 48, 202, 57, 136, 92, 109, 128, 26, 11, 93, 188, 152, 134, 64, 243, 194, 94,
            47, 55, 83, 31, 225, 56, 197, 197, 175, 137, 85, 212, 27,
        ]),
        CreatedFilter::signature()
    );
}
