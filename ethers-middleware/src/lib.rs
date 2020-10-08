//! # Ethers Middleware
//!
//! Ethers uses a middleware architecture. You start the middleware stack with
//! a [`Provider`](ethers_providers::Provider), and wrap it with additional
//! middleware functionalities that you need.
//!
//! ## Available Middleware
//! - Signer
//! - Nonce Manager
//! - Gas Escalator
//! - Gas Oracle
//!
//! ## Example of a middleware stack
//!
//! ```no_run
//! use ethers::{
//!     providers::{Provider, Http},
//!     signers::LocalWallet,
//!     middleware::{
//!         gas_escalator::{GasEscalatorMiddleware, GeometricGasPrice, Frequency},
//!         gas_oracle::{GasOracleMiddleware, GasNow, GasCategory},
//!         signer::SignerMiddleware,
//!         nonce_manager::NonceManagerMiddleware,
//!     },
//!     core::rand,
//! };
//! use std::convert::TryFrom;
//!
//! // Start the stack
//! let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
//!
//! // Escalate gas prices
//! let escalator = GeometricGasPrice::new(1.125, 60u64, None::<u64>);
//! let provider =
//!     GasEscalatorMiddleware::new(provider, escalator, Frequency::PerBlock);
//!
//! // Sign transactions with a private key
//! let signer = LocalWallet::new(&mut rand::thread_rng());
//! let address = signer.address();
//! let provider = SignerMiddleware::new(provider, signer);
//!
//! // Use GasNow as the gas oracle
//! let gas_oracle = GasNow::new().category(GasCategory::SafeLow);
//! let provider = GasOracleMiddleware::new(provider, gas_oracle);
//!
//! // Manage nonces locally
//! let provider = NonceManagerMiddleware::new(provider, address);
//!
//! // ... do something with the provider
//! ```

/// The gas escalator middleware is used to re-broadcast transactions with an
/// increasing gas price to guarantee their timely inclusion
pub mod gas_escalator;

/// The gas oracle middleware is used to get the gas price from a list of gas oracles
/// instead of using eth_gasPrice
pub mod gas_oracle;

/// The nonce manager middleware is used to locally calculate nonces instead of
/// using eth_getTransactionCount
pub mod nonce_manager;

/// The signer middleware is used to locally sign transactions and messages
/// instead of using eth_sendTransaction and eth_sign
pub mod signer;
pub use signer::SignerMiddleware;
