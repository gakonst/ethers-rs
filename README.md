# <h1 align="center"> ethers.rs </h1>

**Complete Ethereum and Celo wallet implementation and utilities in Rust**

![Github Actions](https://github.com/gakonst/ethers-rs/workflows/Tests/badge.svg)
[![Telegram Chat](https://img.shields.io/endpoint?color=neon&style=flat-square&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2Fethers_rs)](https://t.me/ethers_rs)
[![Crates.io][crates-badge]][crates-url]

[crates-badge]: https://img.shields.io/crates/v/ethers.svg
[crates-url]: https://crates.io/crates/ethers

## Documentation

Extensive documentation and examples are available [here](https://docs.rs/ethers).

Alternatively, you may clone the repository and run `cd ethers/ && cargo doc --open`

You can also run any of the examples by executing: `cargo run -p ethers --example <name>`

## Add ethers-rs to your repository

```toml
[dependencies]

ethers = "1.0.0"
```

</details>

## Running the tests

Tests require the following installed:

1. [`solc`](https://solidity.readthedocs.io/en/latest/installing-solidity.html) (>=0.8.10). We also recommend using [solc-select](https://github.com/crytic/solc-select) for more flexibility.
2. [`anvil`](https://github.com/foundry-rs/foundry/blob/master/anvil/README.md)
3. [`geth`](https://github.com/ethereum/go-ethereum)

In addition, it is recommended that you set the `ETHERSCAN_API_KEY` environment variable
for [the abigen via Etherscan](https://github.com/gakonst/ethers-rs/blob/master/ethers-contract/tests/abigen.rs) tests.
You can get one [here](https://etherscan.io/apis).

### EVM-compatible chains support

There are many chains live which are Ethereum JSON-RPC & EVM compatible, but do not yet have
support for [EIP-2718](https://eips.ethereum.org/EIPS/eip-2718) Typed Transactions. This means
that transactions submitted to them by default in ethers-rs will have invalid serialization. To
address that, you must use the `legacy` feature flag:

```toml
[dependencies]

ethers = { version = "1.0.0", features = ["legacy"] }
```

### Polygon support

There is abigen support for Polygon and the Mumbai test network. It is recommended that you set the `POLYGONSCAN_API_KEY` environment variable.
You can get one [here](https://polygonscan.io/apis).

### Avalanche support

There is abigen support for Avalanche and the Fuji test network. It is recommended that you set the `SNOWTRACE_API_KEY` environment variable.
You can get one [here](https://snowtrace.io/apis).

### Celo Support

[Celo](http://celo.org/) support is turned on via the feature-flag `celo`:

```toml
[dependencies]

ethers = { version = "1.0.0", features = ["celo"] }
```

Celo's transactions differ from Ethereum transactions by including 3 new fields:

-   `fee_currency`: The currency fees are paid in (None for CELO, otherwise it's an Address)
-   `gateway_fee_recipient`: The address of the fee recipient (None for no gateway fee paid)
-   `gateway_fee`: Gateway fee amount (None for no gateway fee paid)

The feature flag enables these additional fields in the transaction request builders and
in the transactions which are fetched over JSON-RPC.

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

ethers = { version = "1.0.0", features = ["ws"] }
```

### Interprocess Communication (IPC)

IPC support is turned on via the feature-flag `ipc`:

```toml
[dependencies]

ethers = { version = "1.0.0", features = ["ipc"] }
```

### HTTP Secure (HTTPS)

If you are looking to connect to a HTTPS endpoint, then you need to enable the `rustls` or `openssl` feature.
feature-flags.

To enable `rustls`:

```toml
[dependencies]

ethers = { version = "1.0.0", features = ["rustls"] }
```

To enable `openssl`:

```toml
[dependencies]

ethers = { version = "1.0.0", features = ["openssl"] }
```

## Note on WASM and FFI bindings

You should be able to build a wasm app that uses ethers-rs (see the [example](./examples/ethers-wasm) for reference). If ethers fails to
compile in WASM, please
[open an issue](https://github.com/gakonst/ethers-rs/issues/new/choose).
There is currently no plan to provide an official JS/TS-accessible library
interface. we believe [ethers.js](https://docs.ethers.io/v5/) serves that need
very well.

Similarly, you should be able to build FFI bindings to ethers-rs. If ethers
fails to compile in c lib formats, please
[open an issue](https://github.com/gakonst/ethers-rs/issues/new/choose).
There is currently no plan to provide official FFI bindings, and as ethers-rs is
not yet stable 1.0.0, its interface may change significantly between versions.

## Getting Help

First, see if the answer to your question can be found in the [API documentation](https://docs.rs/ethers). If the answer
is not there, try opening an [issue](https://github.com/gakonst/ethers-rs/issues/new) with the question.

Join the [ethers-rs telegram](https://t.me/ethers_rs) to chat with the community!

## Contributing

Thanks for your help improving the project! We are so happy to have you! We have
[a contributing guide](https://github.com/gakonst/ethers-rs/blob/master/CONTRIBUTING.md) to
help you get involved in the ethers-rs project.

If you make a Pull Request, do not forget to add your changes in the [CHANGELOG](CHANGELOG.md) and ensure your code is
properly formatted with `cargo +nightly fmt` and clippy is happy `cargo clippy`, you can even try to let clippy fix simple
issues itself: `cargo +nightly clippy --fix -Z unstable-options`

## Related Projects

This library would not have been possible without the great work done in:

-   [`ethers.js`](https://github.com/ethers-io/ethers.js/)
-   [`rust-web3`](https://github.com/tomusdrw/rust-web3/)
-   [`ethcontract-rs`](https://github.com/gnosis/ethcontract-rs/)
-   [`guac_rs`](https://github.com/althea-net/guac_rs/tree/master/web3/src/jsonrpc)

A lot of the code was inspired and adapted from them, to a unified and opinionated interface,
built with async/await and std futures from the ground up.

## Projects using ethers-rs

-   [Yield Liquidator](https://github.com/yieldprotocol/yield-liquidator/): Liquidator for Yield Protocol
-   [MEV Inspect](https://github.com/flashbots/mev-inspect-rs/): Miner Extractable Value inspector
-   [Ethers Flashbots](https://github.com/onbjerg/ethers-flashbots): Ethers middleware for [Flashbots](https://docs.flashbots.net)
-   [Ethers Fireblocks](https://github.com/gakonst/ethers-fireblocks): Ethers middleware and signer for [Fireblocks](https://fireblocks.io)' API
-   [Celo Threshold BLS DKG](https://github.com/celo-org/celo-threshold-bls-rs/): CLI for using Celo as a data availability network for the Joint-Feldman BLS DKG
-   [Celo Plumo Prover](https://github.com/celo-org/plumo-prover): Creates Celo's ultralight client proof from on-chain data
-   [Celo SNARK Setup Coordinator](https://github.com/celo-org/snark-setup-operator): Coordinator for executing a pipelined Groth16 SNARK setup
