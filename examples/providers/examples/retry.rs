use ethers::prelude::*;
use reqwest::Url;
use std::time::Duration;

const RPC_URL: &str = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27";

/// The RetryClient is a type that wraps around a JsonRpcClient and automatically retries failed
/// requests using an exponential backoff and filtering based on a RetryPolicy. It presents as a
/// JsonRpcClient, but with additional functionality for retrying requests.
///
/// The RetryPolicy can be customized for specific applications and endpoints, mainly to handle
/// rate-limiting errors. In addition to the RetryPolicy, errors caused by connectivity issues such
/// as timed out connections or responses in the 5xx range can also be retried separately.
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let provider = Http::new(Url::parse(RPC_URL)?);

    let client = RetryClientBuilder::default()
        .rate_limit_retries(10)
        .timeout_retries(3)
        .initial_backoff(Duration::from_millis(500))
        .build(provider, Box::new(HttpRateLimitRetryPolicy::default()));

    // Send a JSON-RPC request for the latest block
    let block_num = "latest".to_string();
    let txn_details = false;
    let params = (block_num, txn_details);

    let block: Block<H256> =
        JsonRpcClient::request(&client, "eth_getBlockByNumber", params).await?;

    println!("{block:?}");

    Ok(())
}
