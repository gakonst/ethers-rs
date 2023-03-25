use ethers::{
    contract::{abigen, ContractFactory, Eip712, EthAbiType},
    core::{
        types::{transaction::eip712::Eip712, Address, Bytes, I256, U256},
        utils::{keccak256, Anvil},
    },
    providers::Provider,
    signers::LocalWallet,
    solc::Solc,
};
use std::{path::PathBuf, sync::Arc};

#[tokio::test(flavor = "multi_thread")]
async fn test_derive_eip712() {
    // Generate Contract ABI Bindings
    abigen!(
        DeriveEip712Test,
        "./ethers-contract/tests/solidity-contracts/derive_eip712_abi.json",
        event_derives(serde::Deserialize, serde::Serialize)
    );

    // Create derived structs

    #[derive(Debug, Clone, Eip712, EthAbiType)]
    #[eip712(
        name = "Eip712Test",
        version = "1",
        chain_id = 1,
        verifying_contract = "0x0000000000000000000000000000000000000001",
        salt = "eip712-test-75F0CCte"
    )]
    struct FooBar {
        foo: I256,
        bar: U256,
        fizz: Bytes,
        buzz: [u8; 32],
        far: String,
        out: Address,
    }

    // get ABI and bytecode for the DeriveEip712Test contract
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    Solc::find_or_install_svm_version("0.6.0").unwrap(); // install solc
    let result = Solc::default().compile_source(path).unwrap();
    let (abi, bytecode, _) = result
        .find("DeriveEip712Test")
        .expect("failed to get DeriveEip712Test contract")
        .into_parts_or_default();

    // launch the network & connect to it
    let anvil = Anvil::new().spawn();
    let from = anvil.addresses()[0];
    let provider = Provider::try_from(anvil.endpoint())
        .unwrap()
        .with_sender(from)
        .interval(std::time::Duration::from_millis(10));
    let client = Arc::new(provider);

    let factory = ContractFactory::new(abi.clone(), bytecode.clone(), client.clone());

    let contract = factory
        .deploy(())
        .expect("failed to deploy DeriveEip712Test contract")
        .legacy()
        .send()
        .await
        .expect("failed to instantiate factory for DeriveEip712 contract");

    let addr = contract.address();

    let contract = DeriveEip712Test::new(addr, client.clone());

    let foo_bar = FooBar {
        foo: I256::from(10u64),
        bar: U256::from(20u64),
        fizz: b"fizz".into(),
        buzz: keccak256("buzz"),
        far: String::from("space"),
        out: Address::from([0; 20]),
    };

    let derived_foo_bar = derive_eip_712_test::FooBar {
        foo: foo_bar.foo,
        bar: foo_bar.bar,
        fizz: foo_bar.fizz.clone(),
        buzz: foo_bar.buzz,
        far: foo_bar.far.clone(),
        out: foo_bar.out,
    };

    use ethers::signers::Signer;

    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let sig = wallet.sign_typed_data(&foo_bar).await.expect("failed to sign typed data");

    let r = <[u8; 32]>::try_from(sig.r)
        .expect("failed to parse 'r' value from signature into [u8; 32]");
    let s = <[u8; 32]>::try_from(sig.s)
        .expect("failed to parse 's' value from signature into [u8; 32]");
    let v = u8::try_from(sig.v).expect("failed to parse 'v' value from signature into u8");

    let domain_separator = contract
        .domain_separator()
        .call()
        .await
        .expect("failed to retrieve domain_separator from contract");
    let type_hash =
        contract.type_hash().call().await.expect("failed to retrieve type_hash from contract");
    let struct_hash = contract
        .struct_hash(derived_foo_bar.clone())
        .call()
        .await
        .expect("failed to retrieve struct_hash from contract");
    let encoded = contract
        .encode_eip_712(derived_foo_bar.clone())
        .call()
        .await
        .expect("failed to retrieve eip712 encoded hash from contract");
    let verify = contract
        .verify_foo_bar(wallet.address(), derived_foo_bar, r, s, v)
        .call()
        .await
        .expect("failed to verify signed typed data eip712 payload");

    assert_eq!(
        domain_separator,
        foo_bar
            .domain()
            .expect("failed to return domain_separator from Eip712 implemented struct")
            .separator(),
        "domain separator does not match contract domain separator!"
    );

    assert_eq!(
        type_hash,
        FooBar::type_hash().expect("failed to return type_hash from Eip712 implemented struct"),
        "type hash does not match contract struct type hash!"
    );

    assert_eq!(
        struct_hash,
        foo_bar
            .clone()
            .struct_hash()
            .expect("failed to return struct_hash from Eip712 implemented struct"),
        "struct hash does not match contract struct hash!"
    );

    assert_eq!(
        encoded,
        foo_bar
            .encode_eip712()
            .expect("failed to return domain_separator from Eip712 implemented struct"),
        "Encoded value does not match!"
    );

    assert!(verify, "typed data signature failed!");
}
