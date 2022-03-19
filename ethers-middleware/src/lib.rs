#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]

/// The [Gas Escalator middleware](crate::gas_escalator::GasEscalatorMiddleware)
/// is used to re-broadcast transactions with an increasing gas price to guarantee
/// their timely inclusion.
pub mod gas_escalator;

/// The gas oracle middleware is used to get the gas price from a list of gas oracles
/// instead of using eth_gasPrice. For usage examples, refer to the
/// [`GasOracle`](crate::gas_oracle::GasOracle) trait.
pub mod gas_oracle;

/// The [Nonce Manager](crate::NonceManagerMiddleware) is used to locally calculate nonces instead
/// of using eth_getTransactionCount
pub mod nonce_manager;
pub use nonce_manager::NonceManagerMiddleware;

/// The [Transformer](crate::transformer::TransformerMiddleware) is used to intercept transactions
/// and transform them to be sent via various supported transformers, e.g.,
/// [DSProxy](crate::transformer::DsProxy)
pub mod transformer;

/// The [Signer](crate::SignerMiddleware) is used to locally sign transactions and messages
/// instead of using eth_sendTransaction and eth_sign
pub mod signer;
pub use signer::SignerMiddleware;

/// The [Policy](crate::PolicyMiddleware) is used to ensure transactions comply with the rules
/// configured in the `PolicyMiddleware` before sending them.
pub mod policy;
pub use policy::PolicyMiddleware;

/// The [TimeLag](crate::TimeLag) provides safety against reorgs by querying state N blocks
/// before the chain tip
pub mod timelag;
pub use timelag::TimeLag;
