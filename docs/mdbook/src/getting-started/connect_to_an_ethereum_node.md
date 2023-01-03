# Connect to an Ethereum node
Ethers-rs allows application to connect the blockchain using web3 providers. Providers act as an interface between applications and an Ethereum node, allowing you to send requests and receive responses via JSON-RPC messages.

Some common actions you can perform using a provider include:

* Getting the current block number
* Getting the balance of an Ethereum address
* Sending a transaction to the blockchain
* Calling a smart contract function
* Subscribe logs and smart contract events
* Getting the transaction history of an address

Providers are an important part of web3 libraries because they allow you to easily interact with the Ethereum blockchain without having to manage the underlying connection to the node yourself.

Code below shows a basic setup to connect a provider to a node:
```rust
/// The `prelude` module provides a convenient way to import a number 
/// of common dependencies at once. This can be useful if you are working 
/// with multiple parts of the library and want to avoid having 
/// to import each dependency individually.
use ethers::prelude::*;

const RPC_URL: &str = "https://mainnet.infura.io/v3/your-project-id";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {    
    let provider = Provider::<Http>::try_from(RPC_URL)?;
    let block_number: U64 = provider.get_block_number().await?;
    println!("{block_number}");

    Ok(())
}
```