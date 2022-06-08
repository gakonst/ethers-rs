use ethers::{
    prelude::*,
    providers::call_raw::RawCall,
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
    let deploy = Greeter::deploy(client.clone(), "Hi".to_string())?;
    let runtime_bytecode = deploy.call_raw().await?;

    // Instantiate a Greeter, though no bytecode exists at the target address
    let greeter = Greeter::new(target, client.clone());

    // Override the target account's code, simulating a call to Greeter.greet()
    // as if the Greeter contract was deployed at the target address
    let state = call_raw::code(target, runtime_bytecode.clone());
    let res = greeter.greet().call_raw().state(&state).await?;

    // greet() returns the empty string, because the target account's storage is empty
    assert_eq!(&res, "");

    // Encode the greeting string as solidity expects it to be stored
    let greeting = "Hello, world";
    let greet_slot = H256::zero();
    let greet_val = encode_string_for_storage(greeting);

    // Override the target account's code and storage
    let mut state = call_raw::state();
    state.account(target).code(runtime_bytecode.clone()).store(greet_slot, greet_val);
    let res = greeter.greet().call_raw().state(&state).await?;

    // The call returns "Hello, world"
    assert_eq!(&res, greeting);

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
