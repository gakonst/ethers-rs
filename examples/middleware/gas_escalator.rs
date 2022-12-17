use ethers::{types::TransactionRequest, utils::Anvil};
use ethers_middleware::gas_escalator::*;
use ethers_providers::{Http, Middleware, Provider};
use eyre::Result;

/// A Gas escalator allows bumping transactions' gas price to avoid getting them
/// stuck in the memory pool.
#[tokio::main]
#[allow(unused_must_use)]
async fn main() {
    let every_secs: u64 = 60;
    let max_price: Option<i32> = None;

    // Linearly increase gas price:
    // Start with `initial_price`, then increase it by fixed amount `increase_by` every `every_secs`
    // seconds until the transaction gets confirmed. There is an optional upper limit.
    let increase_by: i32 = 100;
    let linear_escalator = LinearGasPrice::new(increase_by, every_secs, max_price);
    send_escalating_transaction(linear_escalator).await;

    // Geometrically increase gas price:
    // Start with `initial_price`, then increase it every 'every_secs' seconds by a fixed
    // coefficient. Coefficient defaults to 1.125 (12.5%), the minimum increase for Parity to
    // replace a transaction. Coefficient can be adjusted, and there is an optional upper limit.
    let coefficient: f64 = 1.125;
    let geometric_escalator = GeometricGasPrice::new(coefficient, every_secs, max_price);
    send_escalating_transaction(geometric_escalator).await;
}

async fn send_escalating_transaction<E>(escalator: E) -> Result<()>
where
    E: GasEscalator + Clone + 'static,
{
    // Spawn local node
    let anvil = Anvil::new().spawn();
    let endpoint = anvil.endpoint();

    // Connect to the node
    let provider = Provider::<Http>::try_from(endpoint)?;
    let provider = GasEscalatorMiddleware::new(provider, escalator, Frequency::PerBlock);

    let accounts = provider.get_accounts().await?;
    let from = accounts[0];
    let to = accounts[1];
    let tx = TransactionRequest::new().from(from).to(to).value(1000);

    // Bumps the gas price until transaction gets mined
    let pending_tx = provider.send_transaction(tx, None).await?;
    let receipt = pending_tx.await?;

    println!("{receipt:?}");

    Ok(())
}
