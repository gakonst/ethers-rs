use ethers::{abi::AbiDecode, prelude::*, utils::keccak256};
use eyre::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let client =
        Provider::<Ws>::connect("wss://mainnet.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27")
            .await?;
    let client = Arc::new(client);

    let last_block = client.get_block(BlockNumber::Latest).await?.unwrap().number.unwrap();
    println!("last_block: {}", last_block);

    let erc20_transfer_filter = Filter::new()
        .from_block(last_block - 25)
        .topic0(ValueOrArray::Value(H256::from(keccak256("Transfer(address,address,uint256)"))));

    let mut stream = client.subscribe_logs(&erc20_transfer_filter).await?;

    while let Some(log) = stream.next().await {
        println!(
            "block: {:?}, tx: {:?}, token: {:?}, from: {:?}, to: {:?}, amount: {:?}",
            log.block_number,
            log.transaction_hash,
            log.address,
            Address::from(log.topics[1]),
            Address::from(log.topics[2]),
            U256::decode(log.data)
        );
    }

    Ok(())
}
