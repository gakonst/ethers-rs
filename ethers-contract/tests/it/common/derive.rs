use ethers_contract::{
    abigen, EthAbiCodec, EthAbiType, EthCall, EthDisplay, EthError, EthEvent, EthLogDecode,
};
use ethers_core::{
    abi::{AbiDecode, AbiEncode, RawLog, Tokenizable},
    types::{Address, Bytes, H160, H256, I256, U128, U256},
};

fn assert_tokenizeable<T: Tokenizable>() {}
fn assert_ethcall<T: EthCall>() {}
fn assert_etherror<T: EthError>() {}

#[derive(Debug, Clone, PartialEq, Eq, EthAbiType)]
struct ValueChanged {
    old_author: Address,
    new_author: Address,
    old_value: String,
    new_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, EthAbiType)]
struct ValueChangedWrapper {
    inner: ValueChanged,
    msg: String,
}

#[derive(Debug, Clone, PartialEq, Eq, EthAbiType)]
struct ValueChangedTuple(Address, Address, String, String);

#[derive(Debug, Clone, PartialEq, Eq, EthAbiType)]
struct ValueChangedTupleWrapper(ValueChangedTuple, String);

#[derive(Debug, Clone, PartialEq, Eq, EthAbiType)]
struct ValueChangedVecWrapper {
    inner: Vec<ValueChanged>,
}

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
fn can_derive_abi_type_empty_struct() {
    #[derive(Debug, Clone, PartialEq, Eq, EthAbiType)]
    struct Call();

    #[derive(Debug, Clone, PartialEq, Eq, EthAbiType)]
    struct Call2;

    #[derive(Debug, Clone, PartialEq, Eq, EthAbiType)]
    struct Call3;

    assert_tokenizeable::<Call>();
    assert_tokenizeable::<Call2>();
    assert_tokenizeable::<Call3>();
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
fn can_detokenize_single_field() {
    let value = ValueChangedVecWrapper { inner: vec![] };

    let token = value.clone().into_token();
    assert_eq!(value, ValueChangedVecWrapper::from_token(token).unwrap());

    let value = ValueChangedVecWrapper {
        inner: vec![ValueChanged {
            old_author: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap(),
            new_author: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse().unwrap(),
            old_value: "50".to_string(),
            new_value: "100".to_string(),
        }],
    };

    let token = value.clone().into_token();
    assert_eq!(value, ValueChangedVecWrapper::from_token(token).unwrap());
}

#[test]
fn can_derive_eth_event() {
    #[derive(Debug, Clone, PartialEq, Eq, EthEvent)]
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
    #[derive(Debug, PartialEq, Eq, EthEvent)]
    #[ethevent(name = "MyEvent")]
    struct ValueChangedEvent {
        old_author: Address,
        new_author: Address,
        old_value: String,
        new_value: String,
    }

    assert_eq!("MyEvent", ValueChangedEvent::name());
    assert_eq!("MyEvent(address,address,string,string)", ValueChangedEvent::abi_signature());
}

#[test]
fn can_detect_various_event_abi_types() {
    #[derive(Debug, PartialEq, Eq, EthEvent)]
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
    #[derive(Debug, Clone, PartialEq, Eq, EthAbiType)]
    struct SomeType {
        inner: Address,
        msg: String,
    }

    #[derive(Debug, PartialEq, Eq, EthEvent)]
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

    #[derive(Debug, PartialEq, Eq, EthEvent)]
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
    #[derive(Debug, PartialEq, Eq, EthEvent)]
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

    assert_eq!("Created(address,address,address,address)", CreatedFilter::abi_signature());

    assert_eq!(
        H256([
            37, 155, 48, 202, 57, 136, 92, 109, 128, 26, 11, 93, 188, 152, 134, 64, 243, 194, 94,
            47, 55, 83, 31, 225, 56, 197, 197, 175, 137, 85, 212, 27,
        ]),
        CreatedFilter::signature()
    );
}

#[test]
fn can_decode_event_with_no_topics() {
    #[derive(Debug, PartialEq, Eq, EthEvent)]
    pub struct LiquidateBorrow {
        liquidator: Address,
        borrower: Address,
        repay_amount: U256,
        c_token_collateral: Address,
        seize_tokens: U256,
    }
    // https://etherscan.io/tx/0xb7ba825294f757f8b8b6303b2aef542bcaebc9cc0217ddfaf822200a00594ed9#eventlog index 141
    let log = RawLog {
        topics: vec!["298637f684da70674f26509b10f07ec2fbc77a335ab1e7d6215a4b2484d8bb52"
            .parse()
            .unwrap()],
        data: vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 205, 0, 29, 173, 151, 238, 5, 127, 91, 31,
            197, 154, 221, 40, 175, 143, 32, 26, 201, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 133, 129,
            195, 136, 163, 5, 24, 136, 69, 34, 251, 23, 122, 146, 252, 33, 147, 81, 8, 20, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 18, 195, 162, 210,
            38, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 77, 220, 45, 25, 57, 72, 146, 109, 2,
            249, 177, 254, 158, 29, 170, 7, 24, 39, 14, 213, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 30, 88,
        ],
    };
    let event = <LiquidateBorrow as EthLogDecode>::decode_log(&log).unwrap();
    assert_eq!(event.seize_tokens, 5250648u64.into());
    assert_eq!(event.repay_amount, 653800000000000000u64.into());
}

#[test]
fn can_decode_event_single_param() {
    #[derive(Debug, PartialEq, Eq, EthEvent)]
    pub struct OneParam {
        #[ethevent(indexed)]
        param1: U256,
    }

    let log = RawLog {
        topics: vec![
            "bd9bb67345a2fcc8ef3b0857e7e2901f5a0dcfc7fe5e3c10dc984f02842fb7ba".parse().unwrap(),
            "000000000000000000000000000000000000000000000000000000000000007b".parse().unwrap(),
        ],
        data: vec![],
    };

    let event = <OneParam as EthLogDecode>::decode_log(&log).unwrap();
    assert_eq!(event.param1, 123u64.into());
}

#[test]
fn can_decode_event_tuple_single_param() {
    #[derive(Debug, PartialEq, Eq, EthEvent)]
    struct OneParam(#[ethevent(indexed)] U256);

    let log = RawLog {
        topics: vec![
            "bd9bb67345a2fcc8ef3b0857e7e2901f5a0dcfc7fe5e3c10dc984f02842fb7ba".parse().unwrap(),
            "000000000000000000000000000000000000000000000000000000000000007b".parse().unwrap(),
        ],
        data: vec![],
    };

    let event = <OneParam as EthLogDecode>::decode_log(&log).unwrap();
    assert_eq!(event.0, 123u64.into());
}

#[test]
fn can_decode_event_with_no_params() {
    #[derive(Debug, PartialEq, Eq, EthEvent)]
    pub struct NoParam {}

    let log = RawLog {
        topics: vec!["59a6f900daaeb7581ff830f3a97097fa6372db29b0b50c6d1818ede9d1daaa0c"
            .parse()
            .unwrap()],
        data: vec![],
    };

    let _ = <NoParam as EthLogDecode>::decode_log(&log).unwrap();
}

#[test]
fn eth_display_works() {
    #[derive(Debug, Clone, EthAbiType, EthDisplay)]
    struct MyStruct {
        addr: Address,
        old_value: String,
        new_value: String,
        h: H256,
        i: I256,
        arr_u8: [u8; 32],
        arr_u16: [u16; 32],
        v: Vec<u8>,
    }
    let item = MyStruct {
        addr: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse().unwrap(),
        old_value: "50".to_string(),
        new_value: "100".to_string(),
        h: H256::random(),
        i: I256::zero(),
        arr_u8: [0; 32],
        arr_u16: [1; 32],
        v: vec![0; 32],
    };

    let val = format!(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, 50, 100, 0x{}, {}, 0x{}, {:?}, 0x{}",
        hex::encode(item.h),
        item.i,
        hex::encode(item.arr_u8),
        item.arr_u16,
        hex::encode(&item.v),
    );

    assert_eq!(val, format!("{item}"));
}

#[test]
fn eth_display_works_for_human_readable() {
    ethers_contract::abigen!(
        HevmConsole,
        r#"[
            event log(string)
            event log2(string x)
            ]"#,
    );

    let log = LogFilter("abc".to_string());
    assert_eq!("abc".to_string(), format!("{log}"));
    let log = Log2Filter { x: "abc".to_string() };
    assert_eq!("abc".to_string(), format!("{log}"));
}

#[test]
fn can_derive_ethcall() {
    #[derive(Debug, Clone, EthCall, EthDisplay)]
    struct MyStruct {
        addr: Address,
        old_value: String,
        new_value: String,
        h: H256,
        i: I256,
        arr_u8: [u8; 32],
        arr_u16: [u16; 32],
        nested_arr: [[u8; 32]; 2],
        double_nested: [[[u8; 32]; 2]; 3],
        v: Vec<u8>,
    }

    assert_tokenizeable::<MyStruct>();
    assert_ethcall::<MyStruct>();

    #[derive(Debug, Clone, EthCall, EthDisplay)]
    #[ethcall(name = "my_call")]
    struct MyCall {
        addr: Address,
        old_value: String,
        new_value: String,
    }
    assert_eq!(MyCall::abi_signature().as_ref(), "my_call(address,string,string)");

    assert_tokenizeable::<MyCall>();
    assert_ethcall::<MyCall>();
}

#[test]
fn can_derive_ethcall_with_nested_structs() {
    #[derive(Debug, Clone, PartialEq, Eq, EthAbiType)]
    struct SomeType {
        inner: Address,
        msg: String,
    }

    #[derive(Debug, PartialEq, Eq, EthCall)]
    #[ethcall(name = "foo", abi = "foo(address,(address,string),string)")]
    struct FooCall {
        old_author: Address,
        inner: SomeType,
        new_value: String,
    }

    assert_eq!(FooCall::abi_signature().as_ref(), "foo(address,(address,string),string)");

    assert_tokenizeable::<FooCall>();
    assert_ethcall::<FooCall>();
}

#[test]
fn can_derive_for_enum() {
    #[derive(Debug, Clone, PartialEq, Eq, EthAbiType)]
    enum ActionChoices {
        GoLeft,
        GoRight,
        GoStraight,
        SitStill,
    }
    assert_tokenizeable::<ActionChoices>();

    let token = ActionChoices::GoLeft.into_token();
    assert_eq!(ActionChoices::GoLeft, ActionChoices::from_token(token).unwrap());
}

#[test]
fn can_derive_abi_codec() {
    #[derive(Debug, Clone, PartialEq, Eq, EthAbiType, EthAbiCodec)]
    pub struct SomeType {
        inner: Address,
        msg: String,
    }

    let val = SomeType { inner: Default::default(), msg: "hello".to_string() };

    let encoded = val.clone().encode();
    let other = SomeType::decode(encoded).unwrap();
    assert_eq!(val, other);
}

#[test]
fn can_derive_abi_codec_single_field() {
    #[derive(Debug, Clone, PartialEq, Eq, EthAbiType, EthAbiCodec)]
    pub struct SomeType {
        inner: Vec<U256>,
    }

    let val = SomeType { inner: Default::default() };

    let encoded = val.clone().encode();
    let decoded = SomeType::decode(&encoded).unwrap();
    assert_eq!(val, decoded);

    let encoded_tuple = (Vec::<U256>::default(),).encode();

    assert_eq!(encoded_tuple, encoded);
    let decoded_tuple = SomeType::decode(&encoded_tuple).unwrap();
    assert_eq!(decoded_tuple, decoded);

    let tuple = (val,);
    let encoded = tuple.clone().encode();
    let decoded = <(SomeType,)>::decode(&encoded).unwrap();
    assert_eq!(tuple, decoded);

    let wrapped =
        ethers_core::abi::encode(&ethers_core::abi::Tokenize::into_tokens(tuple.clone())).to_vec();
    assert_eq!(wrapped, encoded);
    let decoded_wrapped = <(SomeType,)>::decode(&wrapped).unwrap();

    assert_eq!(decoded_wrapped, tuple);
}

#[test]
fn can_derive_abi_codec_two_field() {
    #[derive(Debug, Clone, PartialEq, Eq, EthAbiType, EthAbiCodec)]
    pub struct SomeType {
        inner: Vec<U256>,
        addr: Address,
    }

    let val = SomeType { inner: Default::default(), addr: Default::default() };

    let encoded = val.clone().encode();
    let decoded = SomeType::decode(&encoded).unwrap();
    assert_eq!(val, decoded);

    let encoded_tuple = (Vec::<U256>::default(), Address::default()).encode();

    assert_eq!(encoded_tuple, encoded);
    let decoded_tuple = SomeType::decode(&encoded_tuple).unwrap();
    assert_eq!(decoded_tuple, decoded);

    let tuple = (val,);
    let encoded = tuple.clone().encode();
    let decoded = <(SomeType,)>::decode(&encoded).unwrap();
    assert_eq!(tuple, decoded);

    let wrapped =
        ethers_core::abi::encode(&ethers_core::abi::Tokenize::into_tokens(tuple.clone())).to_vec();
    assert_eq!(wrapped, encoded);
    let decoded_wrapped = <(SomeType,)>::decode(&wrapped).unwrap();

    assert_eq!(decoded_wrapped, tuple);
}

#[test]
fn can_derive_ethcall_for_bytes() {
    #[derive(Clone, Debug, Default, Eq, PartialEq, EthCall, EthDisplay)]
    #[ethcall(name = "batch", abi = "batch(bytes[],bool)")]
    pub struct BatchCall {
        pub calls: Vec<Bytes>,
        pub revert_on_fail: bool,
    }

    assert_ethcall::<BatchCall>();
}

#[test]
fn can_derive_array_tuples() {
    #[derive(Clone, Debug, Default, Eq, PartialEq, EthEvent, EthDisplay)]
    #[ethevent(name = "DiamondCut", abi = "DiamondCut((address,uint8,bytes4[])[],address,bytes)")]
    pub struct DiamondCutFilter {
        pub diamond_cut: Vec<(Address, u8, Vec<[u8; 4]>)>,
        pub init: Address,
        pub calldata: Bytes,
    }
}

#[test]
fn can_handle_abigen_tuples() {
    #[derive(Clone, Debug, Default, Eq, PartialEq, EthCall, EthDisplay)]
    #[ethcall(name = "swap", abi = "swap((uint8,uint8)[])")]
    pub struct SwapCall {
        pub pairs_to_swap: ::std::vec::Vec<(u8, u8)>,
    }
}

#[test]
fn eth_display_works_on_ethers_bytes() {
    #[derive(Clone, Debug, Default, Eq, PartialEq, EthCall, EthDisplay)]
    #[ethcall(name = "logBytes", abi = "logBytes(bytes)")]
    pub struct LogBytesCall {
        pub p_0: ethers_core::types::Bytes,
    }
    let call = LogBytesCall { p_0: hex::decode(b"aaaaaa").unwrap().into() };

    let s = format!("{call}");
    assert_eq!(s, "0xaaaaaa");
}

#[test]
fn can_use_result_name() {
    abigen!(
        ResultContract,
        r#"[
           struct Result {uint256 result;}
           result(Result result) (uint256)
        ]"#,
    );

    let _call = ResultCall { result: Result { result: U256::zero() } };
}

#[test]
fn can_derive_etherror() {
    #[derive(Debug, PartialEq, Eq, EthError)]
    #[etherror(name = "MyError", abi = "MyError(address,address,string)")]
    struct MyError {
        old_author: Address,
        addr: Address,
        new_value: String,
    }

    assert_eq!(MyError::abi_signature().as_ref(), "MyError(address,address,string)");

    assert_tokenizeable::<MyError>();
    assert_etherror::<MyError>();
}

#[test]
fn can_use_human_readable_error() {
    abigen!(
        ErrContract,
        r#"[
           error MyError(address,address,string)
        ]"#,
    );

    assert_etherror::<MyError>();
}
