Provider model

should be usable both for tests and for normal actions


Features
- [ ] Keep your private keys in your client, safe and sound
- [ ] Import and export JSON wallets (Geth, Parity and crowdsale)
- [ ] Import and export BIP 39 mnemonic phrases (12 word backup phrases) and HD Wallets (English, Italian, Japanese, Korean, Simplified Chinese, Traditional Chinese; more coming soon)
- [ ] Meta-classes create JavaScript objects from any contract ABI, including ABIv2 and Human-Readable ABI
- [ ] Connect to Ethereum nodes over JSON-RPC, INFURA, Etherscan, Alchemy, Cloudflare or MetaMask.
- [ ] ENS names are first-class citizens; they can be used anywhere an Ethereum addresses can be used
- [ ] Tiny (~88kb compressed; 284kb uncompressed)
- [ ] Complete functionality for all your Ethereum needs
- [ ] Extensive documentation
- [ ] Large collection of test cases which are maintained and added to
- [ ] Fully TypeScript ready, with definition files and full TypeScript source
- [ ] MIT License (including ALL dependencies); completely open source to do with as you please
- [ ] Compatible with ethers.js and Metamask web3 providers via WASM
- Calls by default are made async -> provide a synchronous API


- Provider
- Signer
- Contract
- Choice of BigNumber library? Probably the ones in ethabi
- Human readable ABI very important
- https://docs-beta.ethers.io/getting-started/
- Supports IN-EVM Testing -> SUPER fast tests which are ALSO typesafe
- build.rs type safe methods

This library is inspired by the APIs of Riemann Ether and Ethers https://github.com/summa-tx/riemann-ether#development

- Listening to events via HTTP polls, while WS push/pulls -> look at rust-web3

- Async std futures 1.0 RPC calls for everything  `rpc_impl!` using reqwest and mockito

https://github.com/ethers-io/ethers.js/blob/ethers-v5-beta/packages/contracts/src.ts/index.ts#L721

- Wallet

ethers::wallet::new().connect(provider)
::get_default_provider()

