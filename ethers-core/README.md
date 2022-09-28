# Ethereum types, cryptography and utilities.

It is recommended to use the `utils`, `types` and `abi` re-exports instead of
the `core` module to simplify your imports.

This library provides type definitions for Ethereum's main datatypes along with
other utilities for interacting with the Ethereum ecosystem

## Signing an ethereum-prefixed message

Signing in Ethereum is done by first prefixing the message with
`"\x19Ethereum Signed Message:\n" + message.length`, and then signing the hash
of the result.

```rust,ignore
# async fn foo() -> Result<(), Box<dyn std::error::Error>> {
use ethers::signers::{Signer, LocalWallet};

let message = "Some data";
let wallet = LocalWallet::new(&mut rand::thread_rng());

// Sign the message
let signature = wallet.sign_message(message).await?;

// Recover the signer from the message
let recovered = signature.recover(message)?;

assert_eq!(recovered, wallet.address());
# Ok(())
# }
```

## Utilities

The crate provides utilities for launching local Ethereum testnets by using
`ganache-cli` via the `GanacheBuilder` struct.

# Features

-   ["eip712"] | Provides Eip712 trait for EIP-712 encoding of typed data for
    derived structs

# ABI Encoding and Decoding

This crate re-exports the [`ethabi`](https://docs.rs/ethabi) crate's functions
under the `abi` module, as well as the
[`secp256k1`](https://docs.rs/libsecp256k1) and [`rand`](https://docs.rs/rand)
crates for convenience.
