use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{transaction::conditional::ConditionalOptions, BlockNumber, TransactionRequest},
};
use eyre::Result;

/// Use 'eth_sendRawTransactionConditional' to send a transaction with a conditional options
/// requires, a valid endpoint in `RPC_URL` env var that supports
/// `eth_sendRawTransactionConditional`
#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(url) = std::env::var("RPC_URL") {
        let provider = Provider::<Http>::try_from(url)?;
        let chain_id = provider.get_chainid().await?;
        let wallet: LocalWallet =
            "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc".parse()?;
        let from = wallet.address();

        let client = SignerMiddleware::new(provider, wallet.with_chain_id(chain_id.as_u64()));

        let mut tx = TransactionRequest::default()
            .from(from)
            .to("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
            .value(100)
            .into();

        client.fill_transaction(&mut tx, None).await.unwrap();

        let signed_tx = client.sign_transaction(tx).await.unwrap();
        let pending_tx = client
            .send_raw_transaction_conditional(
                signed_tx,
                ConditionalOptions {
                    block_number_min: Some(BlockNumber::from(33285900)),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let receipt = pending_tx.await?.ok_or_else(|| eyre::eyre!("tx not included"))?;
        let tx = client.get_transaction(receipt.transaction_hash).await?;

        println!("Sent transaction: {}\n", serde_json::to_string(&tx)?);
        println!("Receipt: {}\n", serde_json::to_string(&receipt)?);
    }

    Ok(())
}
