use ethers::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use std::sync::Arc;

const RPC_URL: &str = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27";

/// The Http transport is used to send JSON-RPC requests over Http to an
/// Ethereum node. It allows you to perform various actions on the Ethereum blockchain, such as
/// reading and writing data, sending transactions, and more. To use the Http transport, you will
/// need to create a new `Provider` instance as described in this example.
#[tokio::main]
async fn main() -> eyre::Result<()> {
    create_instance().await?;
    share_providers_across_tasks().await?;
    Ok(())
}

async fn create_instance() -> eyre::Result<()> {
    // An Http provider can be created from an http(s) URI.
    // In case of https you must add the "rustls" or "openssl" feature
    // to the ethers library dependency in `Cargo.toml`.
    let _provider = Provider::<Http>::try_from(RPC_URL)?;

    // Instantiate with auth to append basic authorization headers across requests
    let url = reqwest::Url::parse(RPC_URL)?;
    let auth = Authorization::basic("username", "password");
    let _provider = Http::new_with_auth(url, auth)?;

    // Instantiate from custom Http Client if you need
    // finer control over the Http client configuration
    // (TLS, Proxy, Cookies, Headers, etc.)
    let url = reqwest::Url::parse(RPC_URL)?;

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("Bearer my token"));
    headers.insert("X-MY-HEADERS", HeaderValue::from_static("Some value"));

    let http_client = reqwest::Client::builder()
        .default_headers(headers)
        .proxy(reqwest::Proxy::all("http://proxy.example.com:8080")?)
        .build()?;

    let _provider = Http::new_with_client(url, http_client);

    Ok(())
}

/// Providers can be easily shared across tasks using `Arc` smart pointers
async fn share_providers_across_tasks() -> eyre::Result<()> {
    let provider: Provider<Http> = Provider::<Http>::try_from(RPC_URL)?;

    let client_1 = Arc::new(provider);
    let client_2 = Arc::clone(&client_1);

    let handle1 =
        tokio::spawn(async move { client_1.get_block(BlockNumber::Latest).await.unwrap_or(None) });

    let handle2 =
        tokio::spawn(async move { client_2.get_block(BlockNumber::Latest).await.unwrap_or(None) });

    let block1: Option<Block<H256>> = handle1.await?;
    let block2: Option<Block<H256>> = handle2.await?;

    println!("{block1:?} {block2:?}");

    Ok(())
}
