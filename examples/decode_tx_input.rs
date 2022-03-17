use ethers::abi::AbiDecode;
use ethers::prelude::*;
use eyre::Result;
use hex;

// Abigen creates a SwapExactTokensForTokensCall struct that can be used to decode
// the call data for the swapExactTokensForTokens function in the IUniswapV2Router02 contract
abigen!(
    IUniswapV2Router02,
    r#"[
        swapExactTokensForTokens(uint256 amountIn, uint256 amountOutMin, address[] calldata path, address to, uint256 deadline)
    ]"#,
);
fn main() -> Result<()> {
    println!("Decoding https://etherscan.io/tx/0xd1b449d8b1552156957309bffb988924569de34fbf21b51e7af31070cc80fe9a");
    let tx_input = "0x38ed173900000000000000000000000000000000000000000001a717cc0a3e4f84c00000000000000000000000000000000000000000000000000000000000000283568400000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000201f129111c60401630932d9f9811bd5b5fff34e000000000000000000000000000000000000000000000000000000006227723d000000000000000000000000000000000000000000000000000000000000000200000000000000000000000095ad61b0a150d79219dcf64e1e6cc01f0b64c4ce000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec7";
    let calldata = tx_input.strip_prefix("0x").unwrap_or(tx_input);
    let calldata = Bytes::from(hex::decode(calldata).unwrap());
    let decoded = SwapExactTokensForTokensCall::decode(&calldata)?;
    
    let from = decoded.path.into_iter().next().unwrap();
    println!(
        "Swapped {} of token {} for {} of token {}",
        decoded.amount_in, from, decoded.amount_out_min, decoded.to
    );

    Ok(())
}
