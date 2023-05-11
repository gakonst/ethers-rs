#![allow(clippy::extra_unused_type_parameters)]
#![cfg(not(target_arch = "wasm32"))]

use ethers_core::utils::{Anvil, AnvilInstance};
use ethers_providers::{Http, Provider, Ws};
use ethers_signers::{LocalWallet, Signer};
use std::time::Duration;

mod builder;

mod gas_escalator;

mod gas_oracle;

#[cfg(not(feature = "celo"))]
mod signer;

#[cfg(not(feature = "celo"))]
mod nonce_manager;

#[cfg(not(feature = "celo"))]
mod stack;

#[cfg(not(feature = "celo"))]
mod transformer;

/// Spawns Anvil and instantiates an Http provider.
pub fn spawn_anvil() -> (Provider<Http>, AnvilInstance) {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let provider = Provider::<Http>::try_from(anvil.endpoint())
        .unwrap()
        .interval(Duration::from_millis(50u64));
    (provider, anvil)
}

/// Spawns Anvil and instantiates a Ws provider.
pub async fn spawn_anvil_ws() -> (Provider<Ws>, AnvilInstance) {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let provider = Provider::<Ws>::connect(anvil.ws_endpoint())
        .await
        .unwrap()
        .interval(Duration::from_millis(50u64));
    (provider, anvil)
}

/// Gets `idx` wallet from the given anvil instance.
pub fn get_wallet(anvil: &AnvilInstance, idx: usize) -> LocalWallet {
    LocalWallet::from(anvil.keys()[idx].clone()).with_chain_id(anvil.chain_id())
}

// TODO: Move this to `ethers-tests`
#[tokio::test]
async fn test_derive_eip712() {
    use ethers_contract::{Eip712, EthAbiType};
    use ethers_core::{
        types::{transaction::eip712::Eip712, Address, Bytes, I256, U256},
        utils::{keccak256, Anvil},
    };
    use std::sync::Arc;

    // Generate Contract ABI Bindings
    mod contract {
        ethers_contract::abigen!(
            DeriveEip712Test,
            "./ethers-contract/tests/solidity-contracts/DeriveEip712Test.json",
            derives(serde::Deserialize, serde::Serialize)
        );
    }

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

    // launch the network & connect to it
    let anvil = Anvil::new().spawn();
    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let provider = Provider::try_from(anvil.endpoint())
        .unwrap()
        .with_sender(wallet.address())
        .interval(std::time::Duration::from_millis(10));
    let client = Arc::new(provider);

    let contract: contract::DeriveEip712Test<_> =
        contract::DeriveEip712Test::deploy(client.clone(), ()).unwrap().send().await.unwrap();

    let foo_bar = FooBar {
        foo: I256::from(10u64),
        bar: U256::from(20u64),
        fizz: b"fizz".into(),
        buzz: keccak256("buzz"),
        far: String::from("space"),
        out: Address::zero(),
    };

    let derived_foo_bar = contract::FooBar {
        foo: foo_bar.foo,
        bar: foo_bar.bar,
        fizz: foo_bar.fizz.clone(),
        buzz: foo_bar.buzz,
        far: foo_bar.far.clone(),
        out: foo_bar.out,
    };

    let sig = wallet.sign_typed_data(&foo_bar).await.expect("failed to sign typed data");

    let mut r = [0; 32];
    sig.r.to_big_endian(&mut r);
    let mut s = [0; 32];
    sig.s.to_big_endian(&mut s);
    let v = sig.v as u8;

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
