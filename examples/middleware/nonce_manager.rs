use ethers::{
    types::{BlockNumber, TransactionRequest},
    utils::Anvil,
};
use ethers_middleware::MiddlewareBuilder;
use ethers_providers::{Http, Middleware, Provider};
use eyre::Result;

/// NonceManagerMiddleware is used for calculating nonces locally, useful for signing multiple
/// consecutive transactions without waiting for them to hit the mempool
#[tokio::main]
async fn main() -> Result<()> {
    let anvil = Anvil::new().spawn();
    let endpoint = anvil.endpoint();

    let provider = Provider::<Http>::try_from(endpoint)?;
    let accounts = provider.get_accounts().await?;
    let account = accounts[0];
    let to = accounts[1];
    let tx = TransactionRequest::new().from(account).to(to).value(1000);

    let nonce_manager = provider.nonce_manager(account);

    let curr_nonce = nonce_manager
        .get_transaction_count(account, Some(BlockNumber::Pending.into()))
        .await?
        .as_u64();

    assert_eq!(curr_nonce, 0);

    nonce_manager.send_transaction(tx, None).await?;
    let next_nonce = nonce_manager.next().as_u64();

    assert_eq!(next_nonce, 1);

    Ok(())
}
