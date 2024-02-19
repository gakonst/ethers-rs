use ethers::{
    core::{types::TransactionRequest, utils::Anvil},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};
use eyre::Result;

/// In Ethereum, transactions must be signed with a private key before they can be broadcast to the
/// network. Ethers-rs provides a way to customize this process by allowing
/// you to define a signer, called to sign transactions before they are sent.
#[tokio::main]
async fn main() -> Result<()> {
    let anvil = Anvil::new().spawn();

    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let wallet2: LocalWallet = anvil.keys()[1].clone().into();

    // connect to the network
    let provider = Provider::<Http>::try_from(anvil.endpoint())?;

    // connect the wallet to the provider
    let client = SignerMiddleware::new(provider, wallet.with_chain_id(anvil.chain_id()));

    // craft the transaction
    let tx = TransactionRequest::new().to(wallet2.address()).value(10000);

    // send it!
    let pending_tx = client.send_transaction(tx, None).await?;

    // get the mined tx
    let receipt = pending_tx.await?.ok_or_else(|| eyre::format_err!("tx dropped from mempool"))?;
    let tx = client.get_transaction(receipt.transaction_hash).await?;

    println!("Sent tx: {}\n", serde_json::to_string(&tx)?);
    println!("Tx receipt: {}", serde_json::to_string(&receipt)?);

    Ok(())
}
