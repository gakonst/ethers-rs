# ethers.rs

Complete Ethereum wallet implementation and utilities in Rust (with WASM and FFI support).

## Features

- [x] User friendly transaction APIs
- [x] Type-safe EIP-155 transactions
- [ ] Event Monitoring
- [ ] Deploy and interact with smart contracts
- [ ] Type safe smart contract bindings
- [ ] Hardware wallet support
- [ ] ...

## Examples

### Sending a transaction with an offline key

```rust
use ethers::{types::TransactionRequest, HttpProvider, MainnetWallet};
use std::convert::TryFrom;

// connect to the network
let provider = HttpProvider::try_from("http://localhost:8545")?;

// create a wallet and connect it to the provider
let client = "15c42bf2987d5a8a73804a8ea72fb4149f88adf73e98fc3f8a8ce9f24fcb7774"
    .parse::<MainnetWallet>()?
    .connect(&provider);

// craft the transaction using the builder pattern
let tx = TransactionRequest::new()
    .send_to_str("986eE0C8B91A58e490Ee59718Cca41056Cf55f24")?
    .value(10000);

// send it!
let tx = client.sign_and_send_transaction(tx, None).await?;

// get the mined tx
let tx = client.get_transaction(tx.hash).await?;

println!("{}", serde_json::to_string(&tx)?);
```
