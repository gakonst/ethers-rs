#![cfg(feature = "abigen")]
//! Test cases to validate the `abigen!` macro
use ethers_contract::{abigen, EthCall, EthEvent};
use ethers_core::{
    abi::{AbiDecode, AbiEncode, Address, Tokenizable},
    types::{transaction::eip2718::TypedTransaction, Eip1559TransactionRequest, U256},
    utils::Anvil,
};
use ethers_middleware::SignerMiddleware;
use ethers_providers::{MockProvider, Provider};
use ethers_signers::{LocalWallet, Signer};
use ethers_solc::Solc;
use std::{convert::TryFrom, sync::Arc};

fn assert_codec<T: AbiDecode + AbiEncode>() {}
fn assert_tokenizeable<T: Tokenizable>() {}

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
    assert_eq!("ValueChanged(address,string,string)", ValueChangedFilter::abi_signature());
}

#[test]
fn can_gen_not_human_readable() {
    abigen!(VerifierAbiHardhatContract, "./tests/solidity-contracts/verifier_abi_hardhat.json");
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
    assert_eq!("ValueChanged1(address,string,string)", ValueChanged1Filter::abi_signature());
    assert_eq!("ValueChanged2", ValueChanged2Filter::name());
    assert_eq!("ValueChanged2(address,string,string)", ValueChanged2Filter::abi_signature());
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
    let addr = Addresses {
        addr: vec!["eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap()],
        s: "hello".to_string(),
    };
    let token = addr.clone().into_token();
    assert_eq!(addr, Addresses::from_token(token).unwrap());

    assert_eq!("ValueChanged", ValueChangedFilter::name());
    assert_eq!(
        "ValueChanged((address,string),(address,string),(address[],string))",
        ValueChangedFilter::abi_signature()
    );

    assert_codec::<Value>();
    assert_codec::<Addresses>();
    let encoded = addr.clone().encode();
    let other = Addresses::decode(&encoded).unwrap();
    assert_eq!(addr, other);
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

    assert_codec::<Value>();
    assert_codec::<Addresses>();
}

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

    assert_codec::<VerifyingKey>();
    assert_codec::<G1Point>();
    assert_codec::<G2Point>();
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

    assert_codec::<VerifyingKey>();
    assert_codec::<G1Point>();
    assert_codec::<G2Point>();

    let (provider, _) = Provider::mocked();
    let client = Arc::new(provider);

    let g1 = G1Point { x: U256::zero(), y: U256::zero() };
    let g2 = G2Point { x: [U256::zero(), U256::zero()], y: [U256::zero(), U256::zero()] };
    let vk = VerifyingKey {
        alfa_1: g1.clone(),
        beta_2: g2.clone(),
        gamma_2: g2.clone(),
        delta_2: g2.clone(),
        ic: vec![g1.clone()],
    };
    let proof = Proof { a: g1.clone(), b: g2, c: g1 };

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
        function bar(uint256 x, uint256 y, address addr)
        yeet(uint256,uint256,address)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize)
    );
    assert_tokenizeable::<Foo>();
    assert_codec::<Foo>();

    let (client, _mock) = Provider::mocked();
    let contract = SimpleContract::new(Address::default(), Arc::new(client));
    let f = Foo { x: 100u64.into() };
    let _ = contract.foo(f);

    let call = BarCall { x: 1u64.into(), y: 0u64.into(), addr: Address::random() };
    let encoded_call = contract.encode("bar", (call.x, call.y, call.addr)).unwrap();
    assert_eq!(encoded_call, call.clone().encode().into());
    let decoded_call = BarCall::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(call, decoded_call);

    let contract_call = SimpleContractCalls::Bar(call);
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(contract_call, decoded_enum);
    assert_eq!(encoded_call, contract_call.encode().into());

    let call = YeetCall(1u64.into(), 0u64.into(), Address::zero());
    let encoded_call = contract.encode("yeet", (call.0, call.1, call.2)).unwrap();
    assert_eq!(encoded_call, call.clone().encode().into());
    let decoded_call = YeetCall::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(call, decoded_call);

    let contract_call = SimpleContractCalls::Yeet(call.clone());
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(contract_call, decoded_enum);
    assert_eq!(contract_call, call.into());
    assert_eq!(encoded_call, contract_call.encode().into());
}

#[test]
fn can_handle_overloaded_functions() {
    abigen!(
        SimpleContract,
        r#"[
        getValue() (uint256)
        getValue(uint256 otherValue) (uint256)
        getValue(uint256 otherValue, address addr) (uint256)
        setValue(string, string)
        setValue(string)
    ]"#
    );

    let (provider, _) = Provider::mocked();
    let client = Arc::new(provider);
    let contract = SimpleContract::new(Address::zero(), client);
    // ensure both functions are callable
    let _ = contract.get_value();
    let _ = contract.get_value_with_other_value(1337u64.into());
    let _ = contract.get_value_with_other_value_and_addr(1337u64.into(), Address::zero());

    let call = GetValueCall;

    let encoded_call = contract.encode("getValue", ()).unwrap();
    assert_eq!(encoded_call, call.clone().encode().into());
    let decoded_call = GetValueCall::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(call, decoded_call);

    let contract_call = SimpleContractCalls::GetValue(call);
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(contract_call, decoded_enum);
    assert_eq!(encoded_call, contract_call.encode().into());

    let call = GetValueWithOtherValueCall { other_value: 420u64.into() };

    let encoded_call = contract.encode_with_selector([15, 244, 201, 22], call.other_value).unwrap();
    assert_eq!(encoded_call, call.clone().encode().into());
    let decoded_call = GetValueWithOtherValueCall::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(call, decoded_call);

    let contract_call = SimpleContractCalls::GetValueWithOtherValue(call);
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(contract_call, decoded_enum);
    assert_eq!(encoded_call, contract_call.encode().into());

    let call =
        GetValueWithOtherValueAndAddrCall { other_value: 420u64.into(), addr: Address::random() };

    let encoded_call =
        contract.encode_with_selector([14, 97, 29, 56], (call.other_value, call.addr)).unwrap();
    let decoded_call = GetValueWithOtherValueAndAddrCall::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(call, decoded_call);

    let contract_call = SimpleContractCalls::GetValueWithOtherValueAndAddr(call);
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(contract_call, decoded_enum);
    assert_eq!(encoded_call, contract_call.encode().into());

    let call = SetValue0Call("message".to_string());
    let _contract_call = SimpleContractCalls::SetValue0(call);
    let call = SetValue1Call("message".to_string(), "message".to_string());
    let _contract_call = SimpleContractCalls::SetValue1(call);
}

#[test]
fn can_handle_even_more_overloaded_functions() {
    abigen!(
        ConsoleLog,
        r#"[
            log()
            log(string, string)
            log(string)
    ]"#
    );

    let _call = Log0Call;
    let _contract_call = ConsoleLogCalls::Log0;
    let call = Log1Call("message".to_string());
    let _contract_call = ConsoleLogCalls::Log1(call);
    let call = Log2Call("message".to_string(), "message".to_string());
    let _contract_call = ConsoleLogCalls::Log2(call);
}

#[tokio::test]
async fn can_handle_underscore_functions() {
    abigen!(
        SimpleStorage,
        r#"[
            _hashPuzzle() (uint256)
        ]"#;

        SimpleStorage2,
        "ethers-contract/tests/solidity-contracts/simplestorage_abi.json",
    );

    // launch the network & connect to it
    let anvil = Anvil::new().spawn();
    let from = anvil.addresses()[0];
    let provider = Provider::try_from(anvil.endpoint())
        .unwrap()
        .with_sender(from)
        .interval(std::time::Duration::from_millis(10));
    let client = Arc::new(provider);

    let contract = "SimpleStorage";
    let path = "./tests/solidity-contracts/SimpleStorage.sol";
    let compiled = Solc::default().compile_source(path).unwrap();
    let compiled = compiled.get(path, contract).unwrap();
    let factory = ethers_contract::ContractFactory::new(
        compiled.abi.unwrap().clone(),
        compiled.bytecode().unwrap().clone(),
        client.clone(),
    );
    let addr = factory.deploy("hi".to_string()).unwrap().legacy().send().await.unwrap().address();

    // connect to the contract
    let contract = SimpleStorage::new(addr, client.clone());
    let contract2 = SimpleStorage2::new(addr, client.clone());

    let res = contract.hash_puzzle().call().await.unwrap();
    let res2 = contract2.hash_puzzle().call().await.unwrap();
    let res3 = contract.method::<_, U256>("_hashPuzzle", ()).unwrap().call().await.unwrap();
    let res4 = contract2.method::<_, U256>("_hashPuzzle", ()).unwrap().call().await.unwrap();

    // Manual call construction
    use ethers_providers::Middleware;
    // TODO: How do we handle underscores for calls here?
    let data = simplestorage_mod::HashPuzzleCall.encode();
    let tx = Eip1559TransactionRequest::new().data(data).to(addr);
    let tx = TypedTransaction::Eip1559(tx);
    let res5 = client.call(&tx, None).await.unwrap();
    let res5 = U256::from(res5.as_ref());
    assert_eq!(res, 100.into());
    assert_eq!(res, res2);
    assert_eq!(res, res3);
    assert_eq!(res, res4);
    assert_eq!(res, res5);
}

#[test]
fn can_handle_unique_underscore_functions() {
    abigen!(
        ConsoleLog,
        r#"[
            log(string, string)
            _log(string)
            _log_(string)
            __log__(string)
            __log2__(string)
    ]"#
    );
    let call = LogCall("message".to_string(), "message".to_string());
    let _contract_call = ConsoleLogCalls::Log(call);

    let call = _LogCall("message".to_string());
    let _contract_call = ConsoleLogCalls::_Log(call);

    let call = _Log_Call("message".to_string());
    let _contract_call = ConsoleLogCalls::_Log_(call);

    let call = __Log__Call("message".to_string());
    let _contract_call = ConsoleLogCalls::__Log__(call);

    let call = Log2Call("message".to_string());
    let _contract_call = ConsoleLogCalls::Log2(call);
}

#[test]
fn can_handle_underscore_numeric() {
    abigen!(
        Test,
        r#"[
            _100pct(string)
        ]"#
    );
    let _call = _100PctCall("message".to_string());

    let provider = Arc::new(Provider::new(MockProvider::new()));
    let contract = Test::new(Address::default(), Arc::clone(&provider));
    // NOTE: this seems to be weird behaviour of `Inflector::to_snake_case` which turns "100pct" ->
    // "10_0pct"
    let _call = contract._10_0pct("hello".to_string());
}

#[test]
fn can_handle_duplicates_with_same_name() {
    abigen!(
        ConsoleLog,
        r#"[
            log()
            log(uint p0)
            log(string p0)
    ]"#
    );

    let call = Log0Call;
    let _contract_call = ConsoleLogCalls::Log0(call);

    let call = Log1Call { p_0: 100.into() };
    let _contract_call = ConsoleLogCalls::Log1(call);

    let call = Log2Call { p_0: "message".to_string() };
    let _contract_call = ConsoleLogCalls::Log2(call);
}

#[test]
fn can_abigen_console_sol() {
    abigen!(Console, "ethers-contract/tests/solidity-contracts/console.json",);
}

#[test]
fn can_generate_nested_types() {
    abigen!(
        Test,
        r#"[
        struct Outer {Inner inner; uint256[] arr;}
        struct Inner {uint256 inner;}
        function myfun(Outer calldata a)
    ]"#,
    );

    assert_eq!(MyfunCall::abi_signature(), "myfun(((uint256),uint256[]))");

    let (client, _mock) = Provider::mocked();
    let contract = Test::new(Address::default(), Arc::new(client));

    let inner = Inner { inner: 100u64.into() };
    let a = Outer { inner, arr: vec![101u64.into()] };
    let _ = contract.myfun(a.clone());

    let call = MyfunCall { a: a.clone() };
    let encoded_call = contract.encode("myfun", (a,)).unwrap();
    assert_eq!(encoded_call, call.clone().encode().into());
    let decoded_call = MyfunCall::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(call, decoded_call);
}

#[test]
fn can_handle_different_calls() {
    abigen!(
        Test,
        r#"[
        function fooBar()
        function FOO_BAR()
    ]"#,
    );

    let (client, _mock) = Provider::mocked();
    let contract = Test::new(Address::default(), Arc::new(client));

    let _ = contract.fooBar();
    let _ = contract.FOO_BAR();
}

#[test]
fn can_handle_case_sensitive_calls() {
    abigen!(
        StakedOHM,
        r#"[
        index()
        INDEX()
    ]"#,
    );

    let (client, _mock) = Provider::mocked();
    let contract = StakedOHM::new(Address::default(), Arc::new(client));

    let _ = contract.index();
    let _ = contract.INDEX();
}

#[tokio::test]
async fn can_deploy_greeter() {
    abigen!(Greeter, "ethers-contract/tests/solidity-contracts/greeter.json",);
    let anvil = Anvil::new().spawn();
    let from = anvil.addresses()[0];
    let provider = Provider::try_from(anvil.endpoint())
        .unwrap()
        .with_sender(from)
        .interval(std::time::Duration::from_millis(10));
    let client = Arc::new(provider);

    let greeter_contract =
        Greeter::deploy(client, "Hello World!".to_string()).unwrap().legacy().send().await.unwrap();

    let greeting = greeter_contract.greet().call().await.unwrap();
    assert_eq!("Hello World!", greeting);
}

#[tokio::test]
async fn can_abiencoderv2_output() {
    abigen!(AbiEncoderv2Test, "ethers-contract/tests/solidity-contracts/abiencoderv2test_abi.json",);
    let anvil = Anvil::new().spawn();
    let from = anvil.addresses()[0];
    let provider = Provider::try_from(anvil.endpoint())
        .unwrap()
        .with_sender(from)
        .interval(std::time::Duration::from_millis(10));
    let client = Arc::new(provider);

    let contract = "AbiencoderV2Test";
    let path = "./tests/solidity-contracts/Abiencoderv2Test.sol";
    let compiled = Solc::default().compile_source(path).unwrap();
    let compiled = compiled.get(path, contract).unwrap();
    let factory = ethers_contract::ContractFactory::new(
        compiled.abi.unwrap().clone(),
        compiled.bytecode().unwrap().clone(),
        client.clone(),
    );
    let addr = factory.deploy(()).unwrap().legacy().send().await.unwrap().address();

    let contract = AbiEncoderv2Test::new(addr, client.clone());
    let person = Person { name: "Alice".to_string(), age: 20u64.into() };

    let res = contract.default_person().call().await.unwrap();
    assert_eq!(res, person);
}

#[test]
fn can_gen_multi_etherscan() {
    abigen!(
        MyContract, "etherscan:0xdAC17F958D2ee523a2206206994597C13D831ec7";
        MyContract2, "etherscan:0x8418bb725b3ac45ec8fff3791dd8b4e0480cc2a2";
    );

    let provider = Arc::new(Provider::new(MockProvider::new()));
    let _contract = MyContract::new(Address::default(), Arc::clone(&provider));
    let _contract = MyContract2::new(Address::default(), provider);
}

#[test]
fn can_gen_reserved_word_field_names() {
    abigen!(
        Test,
        r#"[
        struct Foo { uint256 ref; }
    ]"#,
    );

    let _foo = Foo { ref_: U256::default() };
}

#[test]
fn can_handle_overloaded_events() {
    abigen!(
        SimpleContract,
        r#"[
            event ActionPaused(string cToken, string action, bool pauseState)
            event ActionPaused(string action, bool pauseState)
    ]"#
    );

    let _ev1 = ActionPaused1Filter {
        c_token: "ctoken".to_string(),
        action: "action".to_string(),
        pause_state: false,
    };
    let _ev2 = ActionPaused2Filter { action: "action".to_string(), pause_state: false };
}

#[tokio::test]
#[cfg(not(feature = "celo"))]
async fn can_send_struct_param() {
    abigen!(StructContract, "./tests/solidity-contracts/StructContract.json");

    let server = Anvil::new().spawn();
    let wallet: LocalWallet = server.keys()[0].clone().into();
    let provider = Provider::try_from(server.endpoint()).unwrap();
    let client = Arc::new(SignerMiddleware::new(provider, wallet.with_chain_id(1337u64)));

    let contract = StructContract::deploy(client, ()).unwrap().legacy().send().await.unwrap();

    let point = Point { x: 1337u64.into(), y: 0u64.into() };
    let tx = contract.submit_point(point).legacy();
    let tx = tx.send().await.unwrap().await.unwrap().unwrap();
    assert_eq!(tx.logs.len(), 1);

    let logs: Vec<NewPointFilter> = contract.event().from_block(0u64).query().await.unwrap();
    assert_eq!(logs.len(), 1);
}
