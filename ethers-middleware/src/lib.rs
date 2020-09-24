//! Ethers Middleware
//!
//! Ethers uses a middleware architecture. You start the middleware stack with
//! a [`Provider`], and wrap it with additional middleware functionalities that
//! you need.
//!
//! # Middlewares
//!
//! ## Gas Oracle
//!
//! ## Signer
//!
//! ## Nonce Manager
pub mod gas_oracle;
pub use gas_oracle::GasOracleMiddleware;

pub mod client;
pub use client::Client;

mod nonce_manager;
pub use nonce_manager::NonceManager;
