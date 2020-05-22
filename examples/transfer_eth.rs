use ethers::providers::{Provider, ProviderTrait};
use ethers::wallet::Signer;
use std::convert::TryFrom;

#[tokio::main]
async fn main() {
    let provider =
        Provider::try_from("https://mainnet.infura.io/v3/4aebe67796c64b95ab20802677b7bb55")
            .unwrap();

    let num = provider.get_block_number().await.unwrap();
    dbg!(num);

    // let signer = Signer::random().connect(&provider);
}
