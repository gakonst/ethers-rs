//! Spawn an [anvil](https://github.com/foundry-rs/foundry/tree/master/anvil) instance in forking mode

use ethers::utils::Anvil;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // ensure `anvil` is available in $PATH
    let anvil =
        Anvil::new().fork("https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27").spawn();

    println!("Anvil running at `{}`", anvil.endpoint());

    Ok(())
}
