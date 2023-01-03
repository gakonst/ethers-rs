use ethers::{
    prelude::{abigen, Abigen},
    providers::{Http, Provider},
    types::Address,
};
use eyre::Result;
use std::sync::Arc;

/// Abigen is used to generate Rust code to interact with smart contracts on the blockchain.
/// It provides a way to encode and decode data that is passed to and from smart contracts.
/// The output of abigen is Rust code, that is bound to the contract's interface, allowing
/// developers to call its methods to read/write on-chain state and subscribe to realtime events.
///
/// The abigen tool can be used in two ways, addressing different use-cases scenarios and developer
/// taste:
///
/// 1. **Rust file generation:** takes a smart contract's Application Binary Interface (ABI)
/// file and generates a Rust file to interact with it. This is useful if the smart contract is
/// referenced in different places in a project. File generation from ABI can also be easily
/// included as a build step of your application.
/// 2. **Rust inline generation:** takes a smart contract's solidity definition and generates inline
/// Rust code to interact with it. This is useful for fast prototyping and for tight scoped
/// use-cases of your contracts.
/// 3. **Rust inline generation from ABI:** similar to the previous point but instead of Solidity
/// code takes in input a smart contract's Application Binary Interface (ABI) file.
#[tokio::main]
async fn main() -> Result<()> {
    rust_file_generation()?;
    rust_inline_generation().await?;
    rust_inline_generation_from_abi();
    Ok(())
}

fn rust_file_generation() -> Result<()> {
    let base_dir = "./examples/contracts/examples/abi";
    Abigen::new("IERC20", format!("{base_dir}/IERC20.json"))?
        .generate()?
        .write_to_file(format!("{base_dir}/ierc20.rs"))?;
    Ok(())
}

fn rust_inline_generation_from_abi() {
    abigen!(IERC20, "./examples/contracts/examples/abi/IERC20.json");
}

async fn rust_inline_generation() -> Result<()> {
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
