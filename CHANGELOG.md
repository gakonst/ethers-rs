# Changelog

## ethers-core

### Unreleased

- Pass compilation time as additional argument to `Reporter::on_solc_success` [1098](https://github.com/gakonst/ethers-rs/pull/1098)
- Fix aws signer bug which maps un-normalized signature to error if no normalization occurs (in `aws::utils::decode_signature`)
- Implement signed transaction RLP decoding [#1096](https://github.com/gakonst/ethers-rs/pull/1096)
- `Transaction::from` will default to `Address::zero()`. Add `recover_from` and
  `recover_from_mut` methods for recovering the sender from signature, and also
  setting the same on tx [1075](https://github.com/gakonst/ethers-rs/pull/1075).
- Add Etherscan account API endpoints [939](https://github.com/gakonst/ethers-rs/pull/939)
- Add FTM Mainet and testnet to parse method "try_from" from Chain.rs and add cronos mainet and testnet to "from_str"
- Add FTM mainnet and testnet Multicall addresses [927](https://github.com/gakonst/ethers-rs/pull/927)
- Add Cronos mainnet beta and testnet to the list of known chains
  [926](https://github.com/gakonst/ethers-rs/pull/926)
- `Chain::to_string` will return the same chain name as `Chain::from_str`
- Add `eth_syncing` [848](https://github.com/gakonst/ethers-rs/pull/848)
- Fix overflow and possible divide-by-zero in `estimate_priority_fee`
- Add BSC mainnet and testnet to the list of known chains
  [831](https://github.com/gakonst/ethers-rs/pull/831)
- Returns error on invalid type conversion instead of panicking
  [691](https://github.com/gakonst/ethers-rs/pull/691/files)
- Change types mapping for solidity `bytes` to rust `ethers::core::Bytes` and
  solidity `uint8[]` to rust `Vec<u8>`.
  [613](https://github.com/gakonst/ethers-rs/pull/613)
- Fix `format_units` to return a `String` of representing a decimal point float
  such that the decimal places don't get truncated.
  [597](https://github.com/gakonst/ethers-rs/pull/597)
- Implement hex display format for `ethers::core::Bytes`
  [#624](https://github.com/gakonst/ethers-rs/pull/624).
- Fix `fee_history` to first try with `block_count` encoded as a hex `QUANTITY`.
  [#668](https://github.com/gakonst/ethers-rs/pull/668)
- Fix `fill_transaction` to set nonces in transactions, if the sender is known
  and no nonce is specified
- Move `fill_transaction` implementation to the provider, to allow middleware
  to properly override its behavior.
- Add informational messages to solc installation and compilation.
- Significantly refactor `MultiAbigen` module generation. Now allows for lib
  generation, and does not make unnecessary disk writes.
  [#854](https://github.com/gakonst/ethers-rs/pull/852)
- Refactor `ethers-contract-abigen` to use `eyre` instead of `anyhow` via
  [#858](https://github.com/gakonst/ethers-rs/pull/858)
- Add `Deployer.send_with_receipt -> Result<(Contract, Receipt), Error>`
  so that the receipt can be returned to the called when deploying
  a contract [#865](https://github.com/gakonst/ethers-rs/pull/865)
- Add Arbitrum mainnet and testnet to the list of known chains
- Add ENS avatar and TXT records resolution
  [#889](https://github.com/gakonst/ethers-rs/pull/889)
- Do not override gas limits provided by an outer middleware when including an EIP-2930 access list
  [#901](https://github.com/gakonst/ethers-rs/pull/901)
- Add a getter to `ProjectCompileOutput` that returns a mapping of compiler
  versions to a vector of name + contract struct tuples
  [#908](https://github.com/gakonst/ethers-rs/pull/908)
- Add Yul compilation [994](https://github.com/gakonst/ethers-rs/pull/994)
- Enforce commutativity of ENS reverse resolution
  [#996](https://github.com/gakonst/ethers-rs/pull/996)

## ethers-contract-abigen

### Unreleased

- Generate a deploy function if bytecode is provided in the abigen! input (json artifact)
  [#1030](https://github.com/gakonst/ethers-rs/pull/1030).
- Generate correct bindings of struct's field names that are reserved words
  [#989](https://github.com/gakonst/ethers-rs/pull/989).

### 0.6.0

- Add `MultiAbigen` to generate a series of contract bindings that can be kept in the repo
  [#724](https://github.com/gakonst/ethers-rs/pull/724).
- Add provided `event_derives` to call and event enums as well
  [#721](https://github.com/gakonst/ethers-rs/pull/721).
- Implement snowtrace and polygonscan on par with the etherscan integration
  [#666](https://github.com/gakonst/ethers-rs/pull/666).

## ethers-solc

### Unreleased

- Bundle svm, svm-builds and sha2 dependencies in new `svm-solc` feature
  [#1071](https://github.com/gakonst/ethers-rs/pull/1071)
- Wrap `ethabi::Contract` into new type `LosslessAbi` and `abi: Option<Abi>` with `abi: Option<LosslessAbi>` in `ConfigurableContractArtifact`
  [#952](https://github.com/gakonst/ethers-rs/pull/952)
- Let `Project` take ownership of `ArtifactOutput` and change trait interface
  [#907](https://github.com/gakonst/ethers-rs/pull/907)
- Total revamp of the `Project::compile` pipeline
  [#802](https://github.com/gakonst/ethers-rs/pull/802)
  - Support multiple versions of compiled contracts
  - Breaking: deprecate hardhat cache file compatibility, cache file now tracks artifact paths and their versions
- Fix flatten replacement target location
  [#846](https://github.com/gakonst/ethers-rs/pull/846)
- Fix duplicate files during flattening
  [#813](https://github.com/gakonst/ethers-rs/pull/813)
- Add ability to flatten file imports
  [#774](https://github.com/gakonst/ethers-rs/pull/774)
- Add dependency graph and resolve all imported libraryfiles
  [#750](https://github.com/gakonst/ethers-rs/pull/750)
- `Remapping::find_many` does not return a `Result` anymore
  [#707](https://github.com/gakonst/ethers-rs/pull/707)
- Add support for hardhat artifacts
  [#677](https://github.com/gakonst/ethers-rs/pull/677)
- Add more utility functions to the `Artifact` trait
  [#673](https://github.com/gakonst/ethers-rs/pull/673)
- Return cached artifacts from project `compile` when the cache only contains
  some files
- Add support for library linking and make `Bytecode`'s `object` filed an
  `enum BytecodeObject` [#656](https://github.com/gakonst/ethers-rs/pull/656).

### 0.6.0

- add `EthAbiCodec` proc macro to derive `AbiEncode` `AbiDecode` implementation
  [#704](https://github.com/gakonst/ethers-rs/pull/704)
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

### Unreleased

- Add support for basic and bearer authentication in http and non-wasm websockets.
  [829](https://github.com/gakonst/ethers-rs/pull/829)
- Export `ethers_providers::IpcError` and `ethers_providers::QuorumError`
  [1012](https://github.com/gakonst/ethers-rs/pull/1012)

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
- Add support for `evm_snapshot` and `evm_revert` dev RPC methods.
  [640](https://github.com/gakonst/ethers-rs/pull/640)

### 0.5.3

- Expose `ens` module [#435](https://github.com/gakonst/ethers-rs/pull/435)
- Add `eth_getProof` [#459](https://github.com/gakonst/ethers-rs/pull/459)

### 0.5.2

- Set resolved ENS name during gas estimation
  ([1e5a9e](https://github.com/gakonst/ethers-rs/commit/1e5a9efb3c678eecd43d5c341b4932da35445831))

## ethers-signers

### Unreleased

- `eth-keystore-rs` crate updated. Allow an optional name for the to-be-generated
  keystore file [#910](https://github.com/gakonst/ethers-rs/pull/910)

### 0.6.0

- `LocalWallet::new_keystore` now returns a tuple `(LocalWallet, String)`
  instead of `LocalWallet`, where the string represents the UUID of the newly
  created encrypted JSON keystore. The JSON keystore is stored as a file
  `/dir/uuid`. The issue [#557](https://github.com/gakonst/ethers-rs/issues/557)
  is addressed [#559](https://github.com/gakonst/ethers-rs/pull/559)

## ethers-contract

### Unreleased

- Add `EventStream::select` to combine streams with different event types
  [#725](https://github.com/gakonst/ethers-rs/pull/725)
- Substitute output tuples with rust struct types for function calls
  [#664](https://github.com/gakonst/ethers-rs/pull/664)
- Add AbiType implementation during EthAbiType expansion
  [#647](https://github.com/gakonst/ethers-rs/pull/647)
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
- add initialize_nonce public function to initialize NonceMiddleManager

### 0.5.3

- Added Time Lagged middleware
  [#457](https://github.com/gakonst/ethers-rs/pull/457)
