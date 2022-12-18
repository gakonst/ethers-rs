use ethers::types::Address;
use ethers_contract::{abigen, Abigen};
use ethers_providers::{Http, Provider};
use eyre::Result;
use std::sync::Arc;

/// `abigen` is used to generate Rust code for interacting with smart contracts on the blockchain.
/// It provides a way to encode and decode data that is passed to and from smart contracts.
///
/// The abigen tool can be used in two ways:
/// 1. From ABI: takes a smart contract's Application Binary Interface (ABI) and generates Rust
/// code to interact with it.
/// 2. Human readable: takes a smart contract's solidity definition and generates inline Rust
/// code to interact with it.
#[tokio::main]
async fn main() -> Result<()> {
    human_readable().await?;
    from_abi()?;
    Ok(())
}

async fn human_readable() -> Result<()> {
    // The abigen! macro expands the contract's code in the current scope
    // so that you can interface your Rust program with the blockchain
    // counterpart of the contract.
    abigen!(
        IERC20,
        r#"[
            function totalSupply() external view returns (uint256)
            function balanceOf(address account) external view returns (uint256)
            function transfer(address recipient, uint256 amount) external returns (bool)
            function allowance(address owner, address spender) external view returns (uint256)
            function approve(address spender, uint256 amount) external returns (bool)
            function transferFrom( address sender, address recipient, uint256 amount) external returns (bool)
            event Transfer(address indexed from, address indexed to, uint256 value)
            event Approval(address indexed owner, address indexed spender, uint256 value)
        ]"#,
    );

    const RPC_URL: &str = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27";
    const WETH_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

    let provider = Provider::<Http>::try_from(RPC_URL)?;
    let client = Arc::new(provider);
    let address: Address = WETH_ADDRESS.parse()?;
    let contract = IERC20::new(address, client);

    if let Ok(total_supply) = contract.total_supply().call().await {
        println!("WETH total supply is {total_supply:?}");
    }

    Ok(())
}

fn from_abi() -> Result<()> {
    // Abigen allows to generate a Rust file with contract bindings directly from an ABI json file.
    // This is useful if you need to use the contract in different places of your project.
    // You can include abigen generation as a build step of your application.
    let base_dir = "./examples/contracts";
    Abigen::new("IERC20", format!("{base_dir}/IERC20.json"))?
        .generate()?
        .write_to_file(format!("{base_dir}/ierc20.rs"))?;
    Ok(())
}
