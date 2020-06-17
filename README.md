# <h1 align="center"> ethers.rs </h1>

**Complete Ethereum and Celo wallet implementation and utilities in Rust**

![Github Actions](https://github.com/gakonst/ethers-rs/workflows/Tests/badge.svg)

## Documentation

Extensive documentation and examples are available [here](https://docs.rs/ethers).

Alternatively, you may clone the repository and run `cd ethers/ && cargo doc --open`

## Add ethers-rs to your repository

```toml
[dependencies]

ethers = { git = "github.com/gakonst/ethers-rs" }
```

</details>

### Celo Support

[Celo](http://celo.org/) support is turned on via the feature-flag `celo`:

```toml
[dependencies]

ethers = { git = "github.com/gakonst/ethers-rs", features = ["celo"] }
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
- [ ] Websockets / `eth_subscribe`
- [ ] Hardware Wallet Support
- [ ] WASM Bindings
- [ ] FFI Bindings
- [ ] CLI for common operations

## Getting Help

First, see if the answer to your question can be found in the [API documentation](https://docs.rs/ethers). If the answer
is not there, try opening an [issue](https://github.com/gakonst/ethers-rs/issues/new) with the question.

## Contributing

Thanks for your help improving the project! We are so happy to have you! We have
[a contributing guide](https://github.com/gakonst/ethers-rs/blob/master/CONTRIBUTING.md) to
help you get involved in the ethers-rs project.

## Related Projects

This library would not have been possibly without the great work done in:
- [`rust-web3`](https://github.com/tomusdrw/rust-web3/)
- [`ethcontract-rs`](https://github.com/gnosis/ethcontract-rs/)

A lot of the code was inspired and adapted from them, to a unified and opinionated interface,
built with async/await and std futures from the ground up.
