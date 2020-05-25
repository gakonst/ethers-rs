use ethers::{
    types::{Address, Filter},
    Contract, HttpProvider, MainnetWallet,
};
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // connect to the network
    let provider = HttpProvider::try_from("http://localhost:8545")?;

    // create a wallet and connect it to the provider
    let client = "15c42bf2987d5a8a73804a8ea72fb4149f88adf73e98fc3f8a8ce9f24fcb7774"
        .parse::<MainnetWallet>()?
        .connect(&provider);

    // Contract should take both provider or a signer

    let contract = Contract::new(
        "f817796F60D268A36a57b8D2dF1B97B14C0D0E1d".parse::<Address>()?,
        abi,
    );

    Ok(())
}
