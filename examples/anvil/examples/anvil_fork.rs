//! Spawn an [anvil](https://github.com/foundry-rs/foundry/tree/master/anvil) instance in forking mode

use ethers::utils::Anvil;
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // ensure `anvil` is available in $PATH
    let anvil = Anvil::new().fork("https://eth.llamarpc.com").spawn();

    println!("Anvil running at `{}`", anvil.endpoint());

    Ok(())
}
