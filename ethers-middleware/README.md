Ethers uses a middleware-based architecture. You start the middleware stack with
a [`Provider`](ethers_providers::Provider), and wrap it with additional
middleware functionalities that you need.

## Available Middleware

- [`Signer`](crate::SignerMiddleware): Signs transactions locally, with a
  private key or a hardware wallet
- [`Nonce Manager`](crate::NonceManagerMiddleware): Manages nonces locally,
  allowing the rapid broadcast of transactions without having to wait for them
  to be submitted
- [`Gas Escalator`](crate::gas_escalator::GasEscalatorMiddleware): Bumps
  transaction gas prices in the background
- [`Gas Oracle`](crate::gas_oracle): Allows getting your gas price estimates
  from places other than `eth_gasPrice`.
- [`Transformer`](crate::transformer): Allows intercepting and transforming a
  transaction to be broadcasted via a proxy wallet, e.g.
  [`DSProxy`](crate::transformer::DsProxy).

## Example of a middleware stack

```no_run
use ethers_providers::{Provider, Http};
use ethers_signers::{LocalWallet, Signer};
use ethers_middleware::{
    gas_escalator::{GasEscalatorMiddleware, GeometricGasPrice, Frequency},
    gas_oracle::{GasOracleMiddleware, EthGasStation, GasCategory},
    signer::SignerMiddleware,
    nonce_manager::NonceManagerMiddleware,
};
use ethers_core::rand;
use std::convert::TryFrom;

// Start the stack
let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();

// Escalate gas prices
let escalator = GeometricGasPrice::new(1.125, 60u64, None::<u64>);
let provider =
    GasEscalatorMiddleware::new(provider, escalator, Frequency::PerBlock);

// Sign transactions with a private key
let signer = LocalWallet::new(&mut rand::thread_rng());
let address = signer.address();
let provider = SignerMiddleware::new(provider, signer);

// Use EthGasStation as the gas oracle
let gas_oracle = EthGasStation::new(None);
let provider = GasOracleMiddleware::new(provider, gas_oracle);

// Manage nonces locally
let provider = NonceManagerMiddleware::new(provider, address);

// ... do something with the provider
```
