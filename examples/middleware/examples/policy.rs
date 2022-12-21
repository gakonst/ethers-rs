use ethers_core::{types::TransactionRequest, utils::Anvil};
use ethers_middleware::{
    policy::{PolicyMiddlewareError, RejectEverything},
    MiddlewareBuilder, PolicyMiddleware,
};
use ethers_providers::{Http, Middleware, Provider};
use eyre::Result;

/// Policy middleware is a way to inject custom logic into the process of sending transactions and
/// interacting with contracts on the Ethereum blockchain. It allows you to define rules or policies
/// that should be followed when performing these actions, and to customize the behavior of the
/// library based on these policies.
#[tokio::main]
async fn main() -> Result<()> {
    let anvil = Anvil::new().spawn();
    let endpoint = anvil.endpoint();

    let provider = Provider::<Http>::try_from(endpoint)?;

    let accounts = provider.get_accounts().await?;
    let account = accounts[0];
    let to = accounts[1];
    let tx = TransactionRequest::new().from(account).to(to).value(1000);

    let policy = RejectEverything;
    let policy_middleware = provider.wrap_into(|p| PolicyMiddleware::new(p, policy));

    match policy_middleware.send_transaction(tx, None).await {
        Err(e) => {
            // Given the RejectEverything policy, we expect to execute this branch
            assert!(matches!(e, PolicyMiddlewareError::PolicyError(())))
        }
        _ => panic!("We don't expect this to happen!"),
    }

    Ok(())
}
