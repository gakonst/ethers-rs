# ethers-core

Ethereum data types, cryptography and utilities.

It is recommended to use the `utils`, `types` and `abi` re-exports instead of
the `core` module to simplify your imports.

This library provides type definitions for Ethereum's main datatypes along with
other utilities for interacting with the Ethereum ecosystem

For more information, please refer to the [book](https://gakonst.com/ethers-rs).

## Feature flags

-   `eip712`: Does nothing.

## ABI

This crate re-exports the [`ethabi`](https://docs.rs/ethabi) crate's functions
under the `abi` module, as well as the
[`secp256k1`](https://docs.rs/libsecp256k1) and [`rand`](https://docs.rs/rand)
crates for convenience.

## Utilities

The crate provides utilities for launching local Ethereum testnets by using
`ganache-cli` via the `GanacheBuilder` struct.

## Examples

Calculate the UniswapV2 pair address for two ERC20 tokens:

```rust
# use ethers_core::abi::{self, Token};
# use ethers_core::types::{Address, H256};
# use ethers_core::utils;
let factory: Address = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".parse()?;

let token_a: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;
let token_b: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()?;
let encoded = abi::encode_packed(&[Token::Address(token_a), Token::Address(token_b)])?;
let salt = utils::keccak256(encoded);

let init_code_hash: H256 = "0x96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f".parse()?;

let pair = utils::get_create2_address_from_hash(factory, salt, init_code_hash);
let weth_usdc = "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc".parse()?;
assert_eq!(pair, weth_usdc);
# Ok::<(), Box<dyn std::error::Error>>(())
```
