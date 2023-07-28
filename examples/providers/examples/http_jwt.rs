use ethers::prelude::*;

const RPC_URL: &str = "http://localhost:8551";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    connect_jwt().await?;
    Ok(())
}

async fn connect_jwt() -> eyre::Result<()> {
    // An Http provider can be created from an http(s) URI.
    // In case of https you must add the "rustls" or "openssl" feature
    // to the ethers library dependency in `Cargo.toml`.
    let _provider = Provider::<Http>::try_from(RPC_URL)?;

    // Instantiate with auth to append basic authorization headers across requests
    let url = reqwest::Url::parse(RPC_URL)?;

    // Use a JWT signing key to generate a bearer token
    let jwt_secret = &[42; 32];
    let secret = JwtKey::from_slice(jwt_secret).map_err(|err| eyre::eyre!("Invalid key: {err}"))?;
    let jwt_auth = JwtAuth::new(secret, None, None);
    let token = jwt_auth.generate_token()?;

    let auth = Authorization::bearer(token);
    let _provider = Http::new_with_auth(url, auth)?;

    Ok(())
}
