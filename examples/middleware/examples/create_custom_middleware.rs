use async_trait::async_trait;
use ethers::{
    core::{
        types::{transaction::eip2718::TypedTransaction, BlockId, TransactionRequest, U256},
        utils::{parse_units, Anvil},
    },
    middleware::MiddlewareBuilder,
    providers::{FromErr, Http, Middleware, PendingTransaction, Provider},
    signers::{LocalWallet, Signer},
};
use thiserror::Error;

/// This example demonstrates the mechanisms for creating custom middlewares in ethers-rs.
/// The example includes explanations of the process and code snippets to illustrate the
/// concepts. It is intended for developers who want to learn how to customize the behavior of
/// ethers-rs providers by creating and using custom middlewares.
///
/// This custom middleware increases the gas value of transactions sent through an ethers-rs
/// provider by a specified percentage and will be called for each transaction before it is sent.
/// This can be useful if you want to ensure that transactions have a higher gas value than the
/// estimated, in order to improve the chances of them not to run out of gas when landing on-chain.
#[derive(Debug)]
struct GasMiddleware<M> {
    inner: M,
    /// This value is used to raise the gas value before sending transactions
    contingency: U256,
}

/// Contingency is expressed with 4 units
/// e.g.
/// 50% => 1 + 0.5  => 15000
/// 20% => 1 + 0.2  => 12000
/// 1%  => 1 + 0.01 => 10100
const CONTINGENCY_UNITS: usize = 4;

impl<M> GasMiddleware<M>
where
    M: Middleware,
{
    /// Creates an instance of GasMiddleware
    /// `ìnner` the inner Middleware
    /// `perc` This is an unsigned integer representing the percentage increase in the amount of gas
    /// to be used for the transaction. The percentage is relative to the gas value specified in the
    /// transaction. Valid contingency values are in range 1..=50. Otherwise a custom middleware
    /// error is raised.
    pub fn new(inner: M, perc: u32) -> Result<Self, GasMiddlewareError<M>> {
        let contingency = match perc {
            0 => Err(GasMiddlewareError::TooLowContingency(perc))?,
            51.. => Err(GasMiddlewareError::TooHighContingency(perc))?,
            1..=50 => {
                let decimals = 2;
                let perc = U256::from(perc) * U256::exp10(decimals); // e.g. 50 => 5000
                let one = parse_units(1, CONTINGENCY_UNITS).unwrap();
                let one = U256::from(one);
                one + perc // e.g. 50% => 1 + 0.5 => 10000 + 5000 => 15000
            }
        };

        Ok(Self { inner, contingency })
    }
}

/// Let's implement the `Middleware` trait for our custom middleware.
/// All trait functions are derived automatically, so we just need to
/// override the needed functions.
#[async_trait]
impl<M> Middleware for GasMiddleware<M>
where
    M: Middleware,
{
    type Error = GasMiddlewareError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    /// In this function we bump the transaction gas value by the specified percentage
    /// This can raise a custom middleware error if a gas amount was not set for
    /// the transaction.
    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        let mut tx: TypedTransaction = tx.into();

        let curr_gas: U256 = match tx.gas() {
            Some(gas) => gas.to_owned(),
            None => Err(GasMiddlewareError::NoGasSetForTransaction)?,
        };

        println!("Original transaction gas: {curr_gas:?} wei");
        let units: U256 = U256::exp10(CONTINGENCY_UNITS.into());
        let raised_gas: U256 = (curr_gas * self.contingency) / units;
        tx.set_gas(raised_gas);
        println!("Raised transaction gas: {raised_gas:?} wei");

        // Dispatch the call to the inner layer
        self.inner().send_transaction(tx, block).await.map_err(FromErr::from)
    }
}

/// This example demonstrates how to handle errors in custom middlewares. It shows how to define
/// custom error types, use them in middleware implementations, and how to propagate the errors
/// through the middleware chain. This is intended for developers who want to create custom
/// middlewares that can handle and propagate errors in a consistent and robust way.
#[derive(Error, Debug)]
pub enum GasMiddlewareError<M: Middleware> {
    /// Thrown when the internal middleware errors
    #[error("{0}")]
    MiddlewareError(M::Error),
    /// Specific errors of this GasMiddleware.
    /// Please refer to the `thiserror` crate for
    /// further docs.
    #[error("{0}")]
    TooHighContingency(u32),
    #[error("{0}")]
    TooLowContingency(u32),
    #[error("Cannot raise gas! Gas value not provided for this transaction.")]
    NoGasSetForTransaction,
}

impl<M: Middleware> FromErr<M::Error> for GasMiddlewareError<M> {
    fn from(src: M::Error) -> Self {
        GasMiddlewareError::MiddlewareError(src)
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let anvil = Anvil::new().spawn();

    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let wallet2: LocalWallet = anvil.keys()[1].clone().into();
    let signer = wallet.with_chain_id(anvil.chain_id());

    let gas_raise_perc = 50; // 50%;
    let provider = Provider::<Http>::try_from(anvil.endpoint())?
        .with_signer(signer)
        .wrap_into(|s| GasMiddleware::new(s, gas_raise_perc).unwrap());

    let gas = 15000;
    let tx = TransactionRequest::new().to(wallet2.address()).value(10000).gas(gas);

    let pending_tx = provider.send_transaction(tx, None).await?;

    let receipt = pending_tx.await?.ok_or_else(|| eyre::format_err!("tx dropped from mempool"))?;
    let tx = provider.get_transaction(receipt.transaction_hash).await?;

    println!("Sent tx: {}\n", serde_json::to_string(&tx)?);
    println!("Tx receipt: {}", serde_json::to_string(&receipt)?);

    Ok(())
}
