# Changelog

## ethers-core

### Unreleased

* Allow configuring the optimizer & passing arbitrary arguments to solc [#427](https://github.com/gakonst/ethers-rs/pull/427)
* Decimal support for `ethers_core::utils::parse_units` [#463](https://github.com/gakonst/ethers-rs/pull/463)
* Fixed Wei unit calculation in `Units` [#460](https://github.com/gakonst/ethers-rs/pull/460)
* Add `ethers_core::utils::get_create2_address_from_hash` [#444](https://github.com/gakonst/ethers-rs/pull/444)

### 0.5.2
* Correctly RLP Encode transactions as received from the mempool ([#415](https://github.com/gakonst/ethers-rs/pull/415))

## ethers-providers

### Unreleased

* Expose `ens` module [#435](https://github.com/gakonst/ethers-rs/pull/435)
* Add `eth_getProof` [#459](https://github.com/gakonst/ethers-rs/pull/459)


### 0.5.2
* Set resolved ENS name during gas estimation ([1e5a9e](https://github.com/gakonst/ethers-rs/commit/1e5a9efb3c678eecd43d5c341b4932da35445831))

## ethers-signers

### Unreleased

## ethers-contract

### Unreleased
* (De)Tokenize structs and events with only a single field as `Token:Tuple` ([#417](https://github.com/gakonst/ethers-rs/pull/417))

## ethers-middleware

### Unreleased

* Added Time Lagged middleware [#457](https://github.com/gakonst/ethers-rs/pull/457)
