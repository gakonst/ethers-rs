use ethers_core::types::Address;
use ethers_providers::Middleware;
use ethers_signers::Signer;

use crate::{
    gas_oracle::{GasOracle, GasOracleMiddleware},
    NonceManagerMiddleware, SignerMiddleware,
};

/// A builder struct useful to compose different [`Middleware`](ethers_providers::Middleware) layers
/// and then build a composed [`Provider`](ethers_providers::Provider) architecture.
/// [`Middleware`](ethers_providers::Middleware) composition acts in a wrapping fashion. Adding a
/// new layer results in wrapping its predecessor.
///
/// Builder can be used as follows:
///
/// ```rust
/// use ethers_providers::{Middleware, Provider, Http};
/// use std::sync::Arc;
/// use std::convert::TryFrom;
/// use ethers_signers::{LocalWallet, Signer};
/// use ethers_middleware::{*,gas_escalator::*,gas_oracle::*};
///
/// fn example() {
///     let key = "fdb33e2105f08abe41a8ee3b758726a31abdd57b7a443f470f23efce853af169";
///     let signer = key.parse::<LocalWallet>().unwrap();
///     let address = signer.address();
///     let escalator = GeometricGasPrice::new(1.125, 60_u64, None::<u64>);
///
///     let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
///
///     ProviderBuilder::from(provider)
///         .wrap_into(|p| GasEscalatorMiddleware::new(p, escalator, Frequency::PerBlock))
///         .wrap_into(|p| SignerMiddleware::new(p, signer))
///         .wrap_into(|p| GasOracleMiddleware::new(p, EthGasStation::new(None)))
///         .wrap_into(|p| NonceManagerMiddleware::new(p, address)) // Outermost layer
///         .build();
/// }
/// ```
pub struct ProviderBuilder<M> {
    inner: M,
}

impl<M> ProviderBuilder<M>
where
    M: Middleware,
{
    /// Wraps the current [`Middleware`](ethers_providers::Middleware) inside a
    /// [`SignerMiddleware`](crate::SignerMiddleware).
    ///
    /// [`Signer`] ethers_signers::Signer
    pub fn with_signer<S>(self, signer: S) -> ProviderBuilder<SignerMiddleware<M, S>>
    where
        S: Signer,
    {
        let provider = SignerMiddleware::new(self.inner, signer);
        ProviderBuilder::from(provider)
    }

    /// Wraps the current [`Middleware`](ethers_providers::Middleware) inside a
    /// [`NonceManagerMiddleware`](crate::NonceManagerMiddleware).
    ///
    /// [`Address`] ethers_core::types::Address
    pub fn nonce_manager(self, address: Address) -> ProviderBuilder<NonceManagerMiddleware<M>> {
        let provider = NonceManagerMiddleware::new(self.inner, address);
        ProviderBuilder::from(provider)
    }

    /// Wraps the current [`Middleware`](ethers_providers::Middleware) inside a
    /// [`GasOracleMiddleware`](crate::gas_oracle::GasOracleMiddleware).
    ///
    /// [`Address`] ethers_core::types::Address
    pub fn gas_oracle<G>(self, gas_oracle: G) -> ProviderBuilder<GasOracleMiddleware<M, G>>
    where
        G: GasOracle,
    {
        let provider = GasOracleMiddleware::new(self.inner, gas_oracle);
        ProviderBuilder::from(provider)
    }

    /// Wraps a new [`Middleware`](ethers_providers::Middleware) around the current one.
    ///
    /// `builder_fn` This closure takes the current [`Middleware`](ethers_providers::Middleware) as
    /// an argument. Use this to build a new [`Middleware`](ethers_providers::Middleware) layer
    /// wrapping out the current.
    pub fn wrap_into<F, R>(self, builder_fn: F) -> ProviderBuilder<R>
    where
        F: FnOnce(M) -> R,
        R: Middleware,
    {
        let provider: R = builder_fn(self.inner);
        ProviderBuilder::from(provider)
    }

    /// Returns the overall[`Middleware`](ethers_providers::Middleware) as an owned reference to the
    /// outermost layer
    pub fn build(self) -> M {
        self.inner
    }
}

impl<M: Middleware> From<M> for ProviderBuilder<M> {
    fn from(provider: M) -> Self {
        Self { inner: provider }
    }
}
