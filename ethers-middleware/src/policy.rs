use ethers_core::types::{transaction::eip2718::TypedTransaction, BlockId, Bytes, Selector, U256};
use ethers_providers::{FromErr, Middleware, PendingTransaction};

use async_trait::async_trait;
use ethers_core::abi::Address;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use thiserror::Error;

/// Basic trait to ensure that transactions about to be sent follow certain rules.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Policy: Sync + Send + Debug {
    type Error: Sync + Send + Debug;

    /// Evaluates the transactions.
    ///
    /// Returns Ok with the `tx` or an Err otherwise.
    async fn ensure_can_send(&self, tx: TypedTransaction) -> Result<TypedTransaction, Self::Error>;
}

/// Middleware used to enforce certain policies for transactions.
#[derive(Clone, Debug)]
pub struct PolicyMiddleware<M, P> {
    pub(crate) inner: M,
    pub(crate) policy: P,
}

impl<M: Middleware, P: Policy> FromErr<M::Error> for PolicyMiddlewareError<M, P> {
    fn from(src: M::Error) -> PolicyMiddlewareError<M, P> {
        PolicyMiddlewareError::MiddlewareError(src)
    }
}

impl<M, P> PolicyMiddleware<M, P>
where
    M: Middleware,
    P: Policy,
{
    /// Creates a new client from the provider and policy.
    pub fn new(inner: M, policy: P) -> Self {
        Self { inner, policy }
    }

    pub fn policy(&self) -> &P {
        &self.policy
    }

    pub fn policy_mut(&mut self) -> &mut P {
        &mut self.policy
    }
}

#[derive(Error, Debug)]
/// Error thrown when the client interacts with the policy middleware.
pub enum PolicyMiddlewareError<M: Middleware, P: Policy> {
    /// Thrown when the internal policy errors
    #[error("{0:?}")]
    PolicyError(P::Error),
    /// Thrown when an internal middleware errors
    #[error(transparent)]
    MiddlewareError(M::Error),
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<M, P> Middleware for PolicyMiddleware<M, P>
where
    M: Middleware,
    P: Policy,
{
    type Error = PolicyMiddlewareError<M, P>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    /// This ensures the tx complies with the registered policy.
    /// If so then this simply delegates the transaction to the inner middleware
    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        let tx = self
            .policy
            .ensure_can_send(tx.into())
            .await
            .map_err(PolicyMiddlewareError::PolicyError)?;
        self.inner
            .send_transaction(tx, block)
            .await
            .map_err(PolicyMiddlewareError::MiddlewareError)
    }
}

/// A `Policy` that only lives in memory
#[derive(Debug)]
pub struct MemoryPolicy {
    pub rules: Vec<Rule>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Policy for MemoryPolicy {
    type Error = PolicyError;

    async fn ensure_can_send(&self, tx: TypedTransaction) -> Result<TypedTransaction, Self::Error> {
        self.rules
            .iter()
            .find_map(|rule| rule.ensure(&tx).err())
            .map_or(Ok(tx), Err)
    }
}

/// Different rules that can be configured.
#[derive(Debug)]
pub enum Rule {
    ReceiverAllowList(HashSet<Address>),
    ReceiverBlockList(HashSet<Address>),
    SenderAllowList(HashSet<Address>),
    SenderBlockList(HashSet<Address>),
    AllowList(HashSet<(Address, Address)>),
    BlockList(HashSet<(Address, Address)>),
    ValueCap(U256),
    SenderValueCap(HashMap<Address, U256>),
    ReceiverValueCap(HashMap<Address, U256>),
    InvalidSelector(HashSet<Selector>),
    InvalidReceiverSelector(HashMap<Address, Selector>),
    // Other(Box<dyn Fn(&TypedTransaction) -> Result<(), PolicyError> + Send + Sync>)
}

impl Rule {
    fn ensure(&self, tx: &TypedTransaction) -> Result<(), PolicyError> {
        todo!()
    }
}

/// Reasons why a transaction was rejected.
#[derive(Error, Debug)]
pub enum PolicyError {
    #[error("Invalid receiver address: `{0:?}`")]
    InvalidReceiver(Address),
    #[error("Invalid sender address: `{0:?}`")]
    InvalidSender(Address),
    #[error("Attempted to transfer `{value}` ETH, maximum allowed value `{max}`")]
    ValueExceeded {
        /// The requested value, which exceeds `max`
        value: U256,
        /// The maximum allowed value to be transferred
        max: U256,
    },
    /// The first 4 bytes of the hash of the method signature and.
    #[error("Invalid function selector: `0x{0:?}`")]
    InvalidMethod(Selector),
    /// The first 4 bytes of the hash of the method signature and encoded parameters.
    #[error("Invalid function payload")]
    InvalidPayload(Bytes),
}

/// A policy that does not restrict anything.
#[derive(Debug, Clone, Copy)]
pub struct AllowEverything;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Policy for AllowEverything {
    type Error = ();

    async fn ensure_can_send(&self, tx: TypedTransaction) -> Result<TypedTransaction, Self::Error> {
        Ok(tx)
    }
}

/// A policy that rejects all transactions.
#[derive(Debug, Clone, Copy)]
pub struct RejectEverything;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Policy for RejectEverything {
    type Error = ();

    async fn ensure_can_send(&self, _: TypedTransaction) -> Result<TypedTransaction, Self::Error> {
        Err(())
    }
}
