# <h1 align="center"> ethers.rs </h1>

**Complete Ethereum and Celo wallet implementation and utilities in Rust**

![Github Actions](https://github.com/gakonst/ethers-rs/workflows/Tests/badge.svg)

## Documentation

Extensive documentation and examples are available [here](https://docs.rs/ethers).

Alternatively, you may clone the repository and run `cd ethers/ && cargo doc --open`

## Add ethers-rs to your repository

```toml
[dependencies]

ethers = { git = "https://github.com/gakonst/ethers-rs" }
```

</details>

## Running the tests

Tests require the following installed:
1. [`solc`](https://solidity.readthedocs.io/en/latest/installing-solidity.html). We also recommend using [solc-select](https://github.com/crytic/solc-select) for more flexibility.
2. [`ganache-cli`](https://github.com/trufflesuite/ganache-cli#installation)

In addition, it is recommended that you set the `ETHERSCAN_API_KEY` environment variable 
for [the abigen via Etherscan](https://github.com/gakonst/ethers-rs/blob/master/ethers/tests/major_contracts.rs) tests. 
You can get one [here](https://etherscan.io/apis).

### Celo Support

[Celo](http://celo.org/) support is turned on via the feature-flag `celo`:

```toml
[dependencies]

ethers = { git = "https://github.com/gakonst/ethers-rs", features = ["celo"] }
```

Celo's transactions differ from Ethereum transactions by including 3 new fields:
- `fee_currency`: The currency fees are paid in (None for CELO, otherwise it's an Address)
- `gateway_fee_recipient`: The address of the fee recipient (None for no gateway fee paid)
- `gateway_fee`: Gateway fee amount (None for no gateway fee paid)

The feature flag enables these additional fields in the transaction request builders and
in the transactions which are fetched over JSON-RPC.

## Features

- [x] Ethereum JSON-RPC Client
- [x] Interacting and deploying smart contracts
- [x] Type safe smart contract bindings code generation
- [x] Querying past events
- [x] Event monitoring as `Stream`s
- [x] ENS as a first class citizen
- [x] Celo support
- [x] Websockets / `eth_subscribe`
- [x] Hardware Wallet Support
- [x] Parity APIs (`tracing`, `parity_blockWithReceipts`)
- [x] Geth TxPool API
- [ ] WASM Bindings
- [ ] FFI Bindings
- [ ] CLI for common operations

## Getting Help

First, see if the answer to your question can be found in the [API documentation](https://docs.rs/ethers). If the answer
is not there, try opening an [issue](https://github.com/gakonst/ethers-rs/issues/new) with the question.

Join the [ethers-rs telegram](https://t.me/ethers_rs) to chat with the community!

## Contributing

Thanks for your help improving the project! We are so happy to have you! We have
[a contributing guide](https://github.com/gakonst/ethers-rs/blob/master/CONTRIBUTING.md) to
help you get involved in the ethers-rs project.

## Related Projects

This library would not have been possibly without the great work done in:
- [`ethers.js`](https://github.com/ethers-io/ethers.js/)
- [`rust-web3`](https://github.com/tomusdrw/rust-web3/)
- [`ethcontract-rs`](https://github.com/gnosis/ethcontract-rs/)
- [`guac_rs`](https://github.com/althea-net/guac_rs/tree/master/web3/src/jsonrpc)

A lot of the code was inspired and adapted from them, to a unified and opinionated interface,
built with async/await and std futures from the ground up.

## Projects using ethers-rs

- [Yield Liquidator](https://github.com/yieldprotocol/yield-liquidator/): Liquidator for Yield Protocol
- [MEV Inspect](https://github.com/flashbots/mev-inspect-rs/): Miner Extractable Value inspector
- [Ethers Flashbots](https://github.com/onbjerg/ethers-flashbots): Ethers middleware for [Flashbots](https://docs.flashbots.net)
- [Ethers Fireblocks](https://github.com/gakonst/ethers-fireblocks): Ethers middleware and signer for [Fireblocks](https://fireblocks.io)' API
- [Celo Threshold BLS DKG](https://github.com/celo-org/celo-threshold-bls-rs/): CLI for using Celo as a data availability network for the Joint-Feldman BLS DKG
- [Celo Plumo Prover](https://github.com/celo-org/plumo-prover): Creates Celo's ultralight client proof from on-chain data
- [Celo SNARK Setup Coordinator](https://github.com/celo-org/snark-setup-operator): Coordinator for executing a pipelined Groth16 SNARK setup
