use crate::{
    gas_oracle::{GasOracle, GasOracleMiddleware},
    NonceManagerMiddleware, SignerMiddleware,
};
use ethers_core::types::Address;
use ethers_providers::Middleware;
use ethers_signers::Signer;

/// A builder trait to compose different [`Middleware`](ethers_providers::Middleware) layers
/// and then build a composed [`Provider`](ethers_providers::Provider) architecture.
/// [`Middleware`](ethers_providers::Middleware) composition acts in a wrapping fashion. Adding a
/// new layer results in wrapping its predecessor.
///
/// ```rust
/// use ethers_providers::{Middleware, Provider, Http};
/// use std::sync::Arc;
/// use std::convert::TryFrom;
/// use ethers_signers::{LocalWallet, Signer};
/// use ethers_middleware::{*, gas_escalator::*, gas_oracle::*};
///
/// fn builder_example() {
///     let key = "fdb33e2105f08abe41a8ee3b758726a31abdd57b7a443f470f23efce853af169";
///     let signer = key.parse::<LocalWallet>().unwrap();
///     let address = signer.address();
///     let escalator = GeometricGasPrice::new(1.125, 60_u64, None::<u64>);
///     let gas_oracle = GasNow::new();
///
///     let provider = Provider::<Http>::try_from("http://localhost:8545")
///         .unwrap()
///         .wrap_into(|p| GasEscalatorMiddleware::new(p, escalator, Frequency::PerBlock))
///         .gas_oracle(gas_oracle)
///         .with_signer(signer)
///         .nonce_manager(address); // Outermost layer
/// }
///
/// fn builder_example_raw_wrap() {
///     let key = "fdb33e2105f08abe41a8ee3b758726a31abdd57b7a443f470f23efce853af169";
///     let signer = key.parse::<LocalWallet>().unwrap();
///     let address = signer.address();
///     let escalator = GeometricGasPrice::new(1.125, 60_u64, None::<u64>);
///
///     let provider = Provider::<Http>::try_from("http://localhost:8545")
///         .unwrap()
///         .wrap_into(|p| GasEscalatorMiddleware::new(p, escalator, Frequency::PerBlock))
///         .wrap_into(|p| SignerMiddleware::new(p, signer))
///         .wrap_into(|p| GasOracleMiddleware::new(p, GasNow::new()))
///         .wrap_into(|p| NonceManagerMiddleware::new(p, address)); // Outermost layer
/// }
/// ```
pub trait MiddlewareBuilder: Middleware + Sized + 'static {
    /// Wraps `self` inside a new [`Middleware`](ethers_providers::Middleware).
    ///
    /// `f` Consumes `self`. Must be used to return a new
    /// [`Middleware`](ethers_providers::Middleware) wrapping `self`.
    fn wrap_into<F, T>(self, f: F) -> T
    where
        F: FnOnce(Self) -> T,
        T: Middleware,
    {
        f(self)
    }

    /// Wraps `self` inside a [`SignerMiddleware`](crate::SignerMiddleware).
    ///
    /// [`Signer`](ethers_signers::Signer)
    fn with_signer<S>(self, s: S) -> SignerMiddleware<Self, S>
    where
        S: Signer,
    {
        SignerMiddleware::new(self, s)
    }

    /// Wraps `self` inside a [`NonceManagerMiddleware`](crate::NonceManagerMiddleware).
    ///
    /// [`Address`](ethers_core::types::Address)
    fn nonce_manager(self, address: Address) -> NonceManagerMiddleware<Self> {
        NonceManagerMiddleware::new(self, address)
    }

    /// Wraps `self` inside a [`GasOracleMiddleware`](crate::gas_oracle::GasOracleMiddleware).
    ///
    /// [`GasOracle`](crate::gas_oracle::GasOracle)
    fn gas_oracle<G>(self, gas_oracle: G) -> GasOracleMiddleware<Self, G>
    where
        G: GasOracle,
    {
        GasOracleMiddleware::new(self, gas_oracle)
    }
}

impl<M> MiddlewareBuilder for M where M: Middleware + Sized + 'static {}
