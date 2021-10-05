use std::{convert::TryFrom, sync::Arc, time::Duration};

use ethers::{
    contract::EthAbiType,
    prelude::*,
    types::{transaction::eip712::Eip712, Address, I256, U256},
    utils::{compile_and_launch_ganache, keccak256, Ganache, Solc},
};
use ethers_derive_eip712::*;

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
    // bar: U256,
    // fizz: Vec<u8>,
    // buzz: [u8; 32],
    // far: String,
    // out: Address,
}

abigen!(
    DeriveEip712Test,
    "./examples/derive_eip712_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let contract_name = "DeriveEip712Test".to_string();
    let (compiled, ganache) =
        compile_and_launch_ganache(Solc::new("**/DeriveEip712Test.sol"), Ganache::new()).await?;

    let wallet: LocalWallet = ganache.keys()[0].clone().into();

    let contract = compiled.get(&contract_name).unwrap();

    let provider =
        Provider::<Http>::try_from(ganache.endpoint())?.interval(Duration::from_millis(10u64));

    let client = SignerMiddleware::new(provider, wallet.clone());
    let client = Arc::new(client);

    let factory = ContractFactory::new(
        contract.abi.clone(),
        contract.bytecode.clone(),
        client.clone(),
    );

    let contract = factory.deploy(())?.legacy().send().await?;

    let addr = contract.address();

    let contract = DeriveEip712Test::new(addr, client.clone());

    let foo_bar = FooBar {
        foo: I256::from(10),
        // bar: U256::from(20),
        // fizz: b"fizz".to_vec(),
        // buzz: keccak256("buzz"),
        // far: String::from("space"),
        // out: Address::from([0; 20]),
    };

    let derived_foo_bar = deriveeip712test_mod::FooBar {
        foo: foo_bar.foo.clone(),
        // bar: foo_bar.bar.clone(),
        // fizz: foo_bar.fizz.clone(),
        // buzz: foo_bar.buzz.clone(),
        // far: foo_bar.far.clone(),
        // out: foo_bar.out.clone(),
    };

    let sig = wallet.sign_typed_data(foo_bar.clone()).await?;

    let mut r = [0; 32];
    let mut s = [0; 32];
    let v = u8::try_from(sig.v)?;

    sig.r.to_big_endian(&mut r);
    sig.r.to_big_endian(&mut s);

    let domain_separator = contract.domain_separator().call().await?;
    let type_hash = contract.type_hash().call().await?;
    let struct_hash = contract.struct_hash(derived_foo_bar.clone()).call().await?;
    let encoded = contract
        .encode_eip_712(derived_foo_bar.clone())
        .call()
        .await?;
    let verify = contract
        .verify_foo_bar(wallet.address(), derived_foo_bar, r, s, v)
        .call()
        .await?;

    assert_eq!(
        domain_separator,
        FooBar::domain_separator()?,
        "domain separator does not match contract domain separator!"
    );

    assert_eq!(
        type_hash,
        FooBar::type_hash()?,
        "type hash does not match contract struct type hash!"
    );

    assert_eq!(
        struct_hash,
        foo_bar.clone().struct_hash()?,
        "struct hash does not match contract struct struct hash!"
    );

    assert_eq!(
        encoded,
        foo_bar.encode_eip712()?,
        "Encoded value does not match!"
    );

    assert_eq!(verify, true, "typed data signature failed!");

    Ok(())
}
