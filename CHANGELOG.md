# Changelog

## ethers-core

### Unreleased

- Change types mapping for solidity `bytes` to rust `ethers::core::Bytes` and
  solidity `uint8[]` to rust `Vec<u8>`.
  [613](https://github.com/gakonst/ethers-rs/pull/613)
- Fix `format_units` to return a `String` of representing a decimal point float
  such that the decimal places don't get truncated.
  [597](https://github.com/gakonst/ethers-rs/pull/597)
- Implement hex display format for `ethers::core::Bytes`
  [#624](https://github.com/gakonst/ethers-rs/pull/624).

## ethers-solc

### Unreleased

- Return cached artifacts from project `compile` when the cache only contains
  some files

### 0.6.0

- move `AbiEncode` `AbiDecode` trait to ethers-core and implement for core types
  [#531](https://github.com/gakonst/ethers-rs/pull/531)
- Add EIP-712 `sign_typed_data` signer method; add ethers-core type `Eip712`
  trait and derive macro in ethers-derive-eip712
  [#481](https://github.com/gakonst/ethers-rs/pull/481)

### 0.5.3

- Allow configuring the optimizer & passing arbitrary arguments to solc
  [#427](https://github.com/gakonst/ethers-rs/pull/427)
- Decimal support for `ethers_core::utils::parse_units`
  [#463](https://github.com/gakonst/ethers-rs/pull/463)
- Fixed Wei unit calculation in `Units`
  [#460](https://github.com/gakonst/ethers-rs/pull/460)
- Add `ethers_core::utils::get_create2_address_from_hash`
  [#444](https://github.com/gakonst/ethers-rs/pull/444)
- Bumped ethabi to 0.15.0 and fixing breaking changes
  [#469](https://github.com/gakonst/ethers-rs/pull/469),
  [#448](https://github.com/gakonst/ethers-rs/pull/448),
  [#445](https://github.com/gakonst/ethers-rs/pull/445)

### 0.5.2

- Correctly RLP Encode transactions as received from the mempool
  ([#415](https://github.com/gakonst/ethers-rs/pull/415))

## ethers-providers

### 0.6.0

- re-export error types for `Http` and `Ws` providers in
  [#570](https://github.com/gakonst/ethers-rs/pull/570)
- add a method on the `Middleware` to broadcast a tx with a series of escalating
  gas prices via [#566](https://github.com/gakonst/ethers-rs/pull/566)
- Remove unnecessary `Serialize` constraint to `R` (the Response type) in the
  `request` method of `JsonRpcClient`.
- Fix `http Provider` data race when generating new request `id`s.
- Add support for `net_version` RPC method.
  [595](https://github.com/gakonst/ethers-rs/pull/595)

### Unreleased

### 0.5.3

- Expose `ens` module [#435](https://github.com/gakonst/ethers-rs/pull/435)
- Add `eth_getProof` [#459](https://github.com/gakonst/ethers-rs/pull/459)

### 0.5.2

- Set resolved ENS name during gas estimation
  ([1e5a9e](https://github.com/gakonst/ethers-rs/commit/1e5a9efb3c678eecd43d5c341b4932da35445831))

## ethers-signers

### Unreleased

### 0.6.0

- `LocalWallet::new_keystore` now returns a tuple `(LocalWallet, String)`
  instead of `LocalWallet`, where the string represents the UUID of the newly
  created encrypted JSON keystore. The JSON keystore is stored as a file
  `/dir/uuid`. The issue [#557](https://github.com/gakonst/ethers-rs/issues/557)
  is addressed [#559](https://github.com/gakonst/ethers-rs/pull/559)

## ethers-contract

### Unreleased

- fix Etherscan conditional HTTP support
  [#632](https://github.com/gakonst/ethers-rs/pull/632)
- use `CARGO_MANIFEST_DIR` as root for relative paths in abigen
  [#631](https://github.com/gakonst/ethers-rs/pull/631)

### 0.6.0

- Provide a way to opt out of networking support in abigen proc macro with
  `abigen-offline` feature [#580](https://github.com/gakonst/ethers-rs/pull/580)
- Add `.call()` method to `Deployer` for performing dry runs of contract
  deployments. [#554](https://github.com/gakonst/ethers-rs/pull/554)
- Improve error message from failure in `ethers_contract_abigen::Source::parse`
  [#552](https://github.com/gakonst/ethers-rs/pull/552)
- use enumerated aliases for overloaded functions
  [#545](https://github.com/gakonst/ethers-rs/pull/545)
- add `EthCall` trait and derive macro which generates matching structs for
  contract calls [#517](https://github.com/gakonst/ethers-rs/pull/517)
- Use rust types as contract function inputs for human readable abi
  [#482](https://github.com/gakonst/ethers-rs/pull/482)
- `abigen!` now generates `Display` for all events using the new `EthDisplay`
  macro [#513](https://github.com/gakonst/ethers-rs/pull/513)
- `abigen!` now supports overloaded functions natively
  [#501](https://github.com/gakonst/ethers-rs/pull/501)
- `abigen!` now supports multiple contracts
  [#498](https://github.com/gakonst/ethers-rs/pull/498)

### Unreleased

### 0.5.3

- (De)Tokenize structs and events with only a single field as `Token:Tuple`
  ([#417](https://github.com/gakonst/ethers-rs/pull/417))

## ethers-middleware

### Unreleased

### 0.6.0

- add the missing constructor for `Timelag` middleware via
  [#568](https://github.com/gakonst/ethers-rs/pull/568)
- Removes GasNow as a gas price oracle
  [#508](https://github.com/gakonst/ethers-rs/pull/508)

### 0.5.3

- Added Time Lagged middleware
  [#457](https://github.com/gakonst/ethers-rs/pull/457)
