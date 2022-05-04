use ethers::{core::{abi::Abi,types::{Address, H256}}};
use ethers::middleware::SignerMiddleware;
use ethers::contract::Contract;
use ethers::providers::{Provider, Http};
use ethers::signers::{LocalWallet, Signer};
use std::{convert::TryFrom, sync::Arc};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    
   // this wallet's private key
   let wallet = "<private key>".parse::<LocalWallet>()?;
   
   // (ugly way to write the ABI inline, you can otherwise read it from a file)
   let abi: Abi = serde_json::from_str(r#"<contract's abi>"#)?;

   // contract address
   let address = "<conract address>".parse::<Address>()?;
   
   // connect to the network
   let provider = Provider::<Http>::try_from("<infura api>").unwrap();
   
   // create the contract object at the address
   let contract = Contract::new(address, abi, provider.clone());
   
   // attach chain id
   // you have to convert it in u64 data type
   // rinkeby chain id = 4
   let wallet = wallet.with_chain_id(4u64);
   let client = SignerMiddleware::new(provider, wallet);
   let client = Arc::new(client);
    
    let contract_with_wallet = contract.connect(Arc::clone(&client));
    
    // Calling constant methods is done by calling `call()` on the method builder.
    // (if the function takes no arguments, then you must use `()` as the argument)
    // convert your input in contract data type
    // example of converting a number into uint256 data type
    // let data = U256::from_dec_str("50000000")?;
    let call = contract_with_wallet
        .method::<_, H256>("<function_name>", ())?;

    // Non-constant methods are executed via the `send()` call on the method builder.
    let pending_tx = call.send().await?;
    let receipt = pending_tx.confirmations(6).await?;
    println!("{:?}", receipt);

    Ok(())
}
