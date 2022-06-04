use ethers::{
    abi::{self, AbiEncode, Detokenize},
    prelude::*,
    providers::call_raw,
    utils::{parse_ether, Geth},
};
use std::sync::Arc;

abigen!(Greeter, "ethers-contract/tests/solidity-contracts/greeter.json",);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let geth = Geth::new().spawn();
    let provider = Provider::<Http>::try_from(geth.endpoint()).unwrap();
    let client = Arc::new(provider);

    // Both empty accounts
    let target: Address = "0x6fC21092DA55B392b045eD78F4732bff3C580e2c".parse()?;
    let from: Address = "0x295a70b2de5e3953354a6a8344e616ed314d7251".parse()?;

    // Override the sender's balance for the call
    let pay_amt = parse_ether(1u64)?;
    let tx = TransactionRequest::pay(target, pay_amt).from(from);
    let state = call_raw::balance(from, pay_amt * 2);

    // The call succeeds as if the sender had sufficient balance
    client.call_raw(&tx.into()).state(&state).await.expect("balance override");

    // Get the runtime bytecode for the Greeter contract using eth_call to
    // simulate the deploy transaction
    let tx = Greeter::deploy(client.clone(), "Hello, world".to_string())?.deployer.tx;
    let runtime_bytecode = client.call_raw(&tx.into()).await?;

    // Get the transaction data for a call to Greeter.greet()
    let greeter = Greeter::new(target, client.clone());
    let greet_call = greeter.greet();
    let tx = greet_call.tx.into();

    // Override the target account's code, simulating a call to Greeter.greet()
    // as if the Greeter contract was deployed at the target address
    let state = call_raw::code(target, runtime_bytecode.clone());
    let returndata = client.call_raw(&tx).state(&state).await?;

    // greet() returns the empty string, because the target account's storage is empty
    let decoded = decode_string(returndata)?;
    assert_eq!(&decoded, "");

    // Encode the greeting string as solidity expects it to be stored
    let greeting = "Hello, world";
    let greet_slot = H256::zero();
    let greet_val = encode_string_for_storage(greeting);

    // Override the target account's code and storage
    let mut state = call_raw::state();
    state.account(target).code(runtime_bytecode).store(greet_slot, greet_val);
    let returndata = client.call_raw(&tx).state(&state).await?;

    // The call returns "Hello, world"
    let decoded = decode_string(returndata)?;
    assert_eq!(&decoded, greeting);
    Ok(())
}

// Solidity stores strings shorter than 32 bytes in a single storage slot,
// with the lowest order byte storing 2 * length and the higher order
// bytes storing the string data (left aligned)
fn encode_string_for_storage(s: &str) -> H256 {
    let mut bytes = s.as_bytes().to_vec();
    let len = bytes.len();
    assert!(len < 32, "longer strings aren't stored in a single slot");
    bytes.resize(31, 0);
    bytes.push(len as u8 * 2);
    H256::from_slice(&bytes)
}

// Decodes an abi encoded string
fn decode_string(data: Bytes) -> eyre::Result<String> {
    let tokens = abi::decode(&[abi::param_type::ParamType::String], data.as_ref())?;
    <String as Detokenize>::from_tokens(tokens).map_err(From::from)
}
