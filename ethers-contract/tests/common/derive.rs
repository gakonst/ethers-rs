use ethers_contract::EthAbiType;
use ethers_core::abi::Tokenizable;
use ethers_core::types::Address;

#[derive(Debug, Clone, PartialEq, EthAbiType)]
pub struct ValueChanged {
    pub old_author: Address,
    pub new_author: Address,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Debug, Clone, PartialEq, EthAbiType)]
pub struct ValueChangedWrapper {
    pub inner: ValueChanged,
    pub msg: String,
}

#[derive(Debug, Clone, PartialEq, EthAbiType)]
pub struct ValueChangedTuple(Address, Address, String, String);

#[derive(Debug, Clone, PartialEq, EthAbiType)]
pub struct ValueChangedTupleWrapper(ValueChangedTuple, String);

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
