# ethers.rs

Complete Ethereum wallet implementation and utilities in Rust (with WASM and FFI support).

## Features

- [x] User friendly transaction APIs
- [x] Type-safe EIP-155 transactions
- [x] Querying past events
- [ ] Event Monitoring
- [ ] Deploy and interact with smart contracts
- [ ] Type safe smart contract bindings
- [ ] Hardware wallet support
- [ ] CLI for creating transactions, interacting with contracts, generating bindings from ABIs (abigen equivalent), ...
- [ ] ...

## Directory Structure

## Acknowledgements

This library would not have been possibly without the great work of the creators of [`rust-web3`]() and [`ethcontract-rs`]()

A lot of the code was inspired and adapted from them, to a unified and opinionated interface. 
That said, Rust-web3 is ~9k LoC (tests included) and ethcontract-rs is 11k lines, 
so in total about 20k lines of code with tests. This library is xxx LoC.
