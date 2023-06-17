# <h1 align="center"> ethers-rs </h1>

**A complete Ethereum and Celo Rust library**

![Github Actions](https://github.com/gakonst/ethers-rs/workflows/Tests/badge.svg)
[![Telegram Chat](https://img.shields.io/endpoint?color=neon&style=flat-square&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2Fethers_rs)](https://t.me/ethers_rs)
[![Crates.io][crates-badge]][crates-url]

[crates-badge]: https://img.shields.io/crates/v/ethers.svg
[crates-url]: https://crates.io/crates/ethers

## Quickstart

Add this to your Cargo.toml:

```toml
[dependencies]
ethers = "2.0"
```

And this to your code:

```rust
use ethers::prelude::*;
```

## Documentation

View the API reference [here](https://docs.rs/ethers) or the online book [here](https://gakonst.com/ethers-rs).

Examples are organized into individual crates under the `/examples` folder.
You can run any of the examples by executing:

```bash
# cargo run -p <example-crate-name> --example <name>
cargo run -p examples-big-numbers --example math_operations
```

## EVM-compatible chains support

There are many chains live which are Ethereum JSON-RPC & EVM compatible, but do not yet have
support for [EIP-2718](https://eips.ethereum.org/EIPS/eip-2718) Typed Transactions. This means
that transactions submitted to them by default in ethers-rs will have invalid serialization. To
address that, you must use the `legacy` feature flag:

```toml
[dependencies]
ethers = { version = "2.0", features = ["legacy"] }
```

### Polygon support

There is abigen support for Polygon and the Mumbai test network. It is recommended that you set the `POLYGONSCAN_API_KEY` environment variable.
You can get one [here](https://polygonscan.io/apis).

### Avalanche support

There is abigen support for Avalanche and the Fuji test network. It is recommended that you set the `SNOWTRACE_API_KEY` environment variable.
You can get one [here](https://snowtrace.io/apis).

### Optimism support

Optimism is supported via the `optimism` feature flag:

```toml
[dependencies]
ethers = { version = "2.0", features = ["optimism"] }
```

Optimism has a new transaction type: [Deposited Transactions](https://github.com/ethereum-optimism/optimism/blob/develop/specs/deposits.md#the-deposited-transaction-type)
with type ID `0x7E`, which requires 3 new fields:

-   `sourceHash`: The hash which uniquely identifies the origin of the deposit
-   `mint`: The ETH value to mint on L2.
-   `isSystemTx`: True if the tx does not interact with the L2 block gas pool

**Note:** the `optimism` and `celo` features are mutually exclusive.

### Celo Support

[Celo](https://celo.org) support is turned on via the feature-flag `celo`:

```toml
[dependencies]
ethers = { version = "2.0", features = ["celo"] }
```

Celo's transactions differ from Ethereum transactions by including 3 new fields:

-   `fee_currency`: The currency fees are paid in (None for CELO, otherwise it's an Address)
-   `gateway_fee_recipient`: The address of the fee recipient (None for no gateway fee paid)
-   `gateway_fee`: Gateway fee amount (None for no gateway fee paid)

The feature flag enables these additional fields in the transaction request builders and
in the transactions which are fetched over JSON-RPC.

**Note:** the `optimism` and `celo` features are mutually exclusive.

## Features

-   [x] Ethereum JSON-RPC Client
-   [x] Interacting and deploying smart contracts
-   [x] Type safe smart contract bindings code generation
-   [x] Querying past events
-   [x] Event monitoring as `Stream`s
-   [x] ENS as a first class citizen
-   [x] Celo support
-   [x] Polygon support
-   [x] Avalanche support
-   [x] Optimism support
-   [x] Websockets / `eth_subscribe`
-   [x] Hardware Wallet Support
-   [x] Parity APIs (`tracing`, `parity_blockWithReceipts`)
-   [x] Geth TxPool API
-   [ ] WASM Bindings (see note)
-   [ ] FFI Bindings (see note)
-   [ ] CLI for common operations

### Websockets

Websockets support is turned on via the feature-flag `ws`:

```toml
[dependencies]
ethers = { version = "2.0", features = ["ws"] }
```

### Interprocess Communication (IPC)

IPC support is turned on via the feature-flag `ipc`:

```toml
[dependencies]
ethers = { version = "2.0", features = ["ipc"] }
```

### HTTP Secure (HTTPS)

If you are looking to connect to a HTTPS endpoint, then you need to enable the `rustls` or `openssl` feature.
feature-flags.

To enable `rustls`:

```toml
[dependencies]
ethers = { version = "2.0", features = ["rustls"] }
```

To enable `openssl`:

```toml
[dependencies]
ethers = { version = "2.0", features = ["openssl"] }
```

## Note on WASM and FFI bindings

You should be able to build a wasm app that uses ethers-rs (see the [example](./examples/wasm) for reference).
If ethers fails to compile in WASM, please [open an issue][issue].
There is currently no plan to provide an official JS/TS-accessible library
interface, as we believe [viem](https://viem.sh) or [ethers.js](https://docs.ethers.io/v6/)
serve that need very well.

Similarly, you should be able to build FFI bindings to ethers-rs. If ethers
fails to compile in C library formats, please [open an issue][issue].
There is currently no plan to provide official FFI bindings.

[issue]: https://github.com/gakonst/ethers-rs/issues/new/choose

## Getting Help

First, see if the answer to your question can be found in the [API documentation](https://docs.rs/ethers). If the answer
is not there, try opening an [issue](https://github.com/gakonst/ethers-rs/issues/new) with the question.

Join the [ethers-rs telegram](https://t.me/ethers_rs) to chat with the community!

## Contributing

Thanks for your help improving the project! We are so happy to have you! We have
[a contributing guide](./CONTRIBUTING.md) to help you get involved in the ethers-rs project.

If you open a Pull Request, do not forget to add your changes in the [CHANGELOG](./CHANGELOG.md), ensure your code is
properly formatted with `cargo +nightly fmt` and that Clippy is happy `cargo clippy`; you can even try to let clippy fix simple
issues itself: `cargo +nightly clippy --fix`

### Running the tests

Tests require the following installed:

1. [`solc`](https://docs.soliditylang.org/en/latest/installing-solidity.html) (>=0.8.0). We also recommend using [svm](https://github.com/roynalnaruto/svm-rs) for more flexibility.
2. [`anvil`](https://github.com/foundry-rs/foundry/blob/master/anvil/README.md)
3. [`geth`](https://github.com/ethereum/go-ethereum)

Additionally, the `ETHERSCAN_API_KEY` environment variable has to be set to run [`ethers-etherscan`](./ethers-etherscan) tests.
You can get one [here](https://etherscan.io/apis).

## Projects using ethers-rs

-   [Yield Liquidator](https://github.com/yieldprotocol/yield-liquidator/): Liquidator for Yield Protocol
-   [MEV Inspect](https://github.com/flashbots/mev-inspect-rs/): Miner Extractable Value inspector
-   [Ethers CCIP-Read](https://github.com/ensdomains/ethers-ccip-read): Ethers middleware for ENS [CCIP-Read](https://eips.ethereum.org/EIPS/eip-3668) support
-   [Ethers Flashbots](https://github.com/onbjerg/ethers-flashbots): Ethers middleware for [Flashbots](https://docs.flashbots.net)
-   [Ethers Fireblocks](https://github.com/gakonst/ethers-fireblocks): Ethers middleware and signer for [Fireblocks](https://fireblocks.io)' API
-   [Celo Threshold BLS DKG](https://github.com/celo-org/celo-threshold-bls-rs/): CLI for using Celo as a data availability network for the Joint-Feldman BLS DKG
-   [Celo Plumo Prover](https://github.com/celo-org/plumo-prover): Creates Celo's ultralight client proof from on-chain data
-   [Celo SNARK Setup Coordinator](https://github.com/celo-org/snark-setup-operator): Coordinator for executing a pipelined Groth16 SNARK setup
-   [ERC-4337 Bundler](https://github.com/Vid201/aa-bundler/): Account Abstraction (ERC-4337) bundler

## Credits

This library would not have been possible without the great work done in:

-   [`ethers.js`](https://github.com/ethers-io/ethers.js/)
-   [`rust-web3`](https://github.com/tomusdrw/rust-web3/)
-   [`ethcontract-rs`](https://github.com/gnosis/ethcontract-rs/)
-   [`guac_rs`](https://github.com/althea-net/guac_rs/)

A lot of the code was inspired and adapted from them, to a unified and opinionated interface,
built with async/await and std futures from the ground up.
