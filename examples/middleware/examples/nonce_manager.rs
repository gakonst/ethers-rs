use ethers::{
    core::{
        types::{BlockNumber, TransactionRequest},
        utils::Anvil,
    },
    middleware::MiddlewareBuilder,
    providers::{Http, Middleware, Provider},
};
use eyre::Result;

/// In Ethereum, the nonce of a transaction is a number that represents the number of transactions
/// that have been sent from a particular account. The nonce is used to ensure that transactions are
/// processed in the order they are intended, and to prevent the same transaction from being
/// processed multiple times.
///
/// The nonce manager in ethers-rs is a middleware that helps you manage the nonce
/// of transactions by keeping track of the current nonce for a given account and automatically
/// incrementing it as needed. This can be useful if you want to ensure that transactions are sent
/// in the correct order, or if you want to avoid having to manually manage the nonce yourself.
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
