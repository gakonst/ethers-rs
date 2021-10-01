use ethers::{
    abi,
    abi::Token,
    prelude::U256,
    types::Address,
    utils,
    utils::{keccak256, parse_ether},
};

const UNISWAP_V2_USDC_ETH_PAIR: &'static str = "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc";

// Generate the EIP712 permit hash to sign for a Uniswap V2 pair.
// https://eips.ethereum.org/EIPS/eip-712
// https://eips.ethereum.org/EIPS/eip-2612
fn main() {
    // Test data
    let owner: Address = "0x617072Cb2a1897192A9d301AC53fC541d35c4d9D"
        .parse()
        .unwrap();
    let spender: Address = "0x2819c144D5946404C0516B6f817a960dB37D4929"
        .parse()
        .unwrap();
    let value = parse_ether(10).unwrap();
    let nonce = U256::from(1);
    let deadline = U256::from(3133728498 as u32);
    let verifying_contract: Address = UNISWAP_V2_USDC_ETH_PAIR.parse().unwrap();
    let name = "Uniswap V2";
    let version = "1";
    let chainid = 1;

    // Typehash for the permit() function
    let permit_typehash = utils::keccak256(
        "Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)",
    );
    // Typehash for the struct used to generate the domain separator
    let domain_typehash = keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
    );

    // Corresponds to solidity's abi.encode()
    let domain_separator_input = abi::encode(&vec![
        Token::Uint(U256::from(domain_typehash)),
        Token::Uint(U256::from(keccak256(&name))),
        Token::Uint(U256::from(keccak256(&version))),
        Token::Uint(U256::from(chainid)),
        Token::Address(verifying_contract),
    ]);

    // Corresponds to the following solidity:
    // DOMAIN_SEPARATOR = keccak256(
    //     abi.encode(
    //         keccak256('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)'),
    //         keccak256(bytes(name)),
    //         keccak256(bytes('1')),
    //         chainId,
    //         address(this)
    //     )
    // );
    let domain_separator = keccak256(&domain_separator_input);

    // Corresponds to solidity's abi.encode()
    let struct_input = abi::encode(&vec![
        Token::Uint(U256::from(permit_typehash)),
        Token::Address(owner),
        Token::Address(spender),
        Token::Uint(value),
        Token::Uint(nonce),
        Token::Uint(deadline),
    ]);
    let struct_hash = keccak256(&struct_input);

    // Corresponds to solidity's abi.encodePacked()
    let digest_input = [
        &[0x19, 0x01],
        domain_separator.as_ref(),
        struct_hash.as_ref(),
    ]
    .concat();

    // Matches the following solidity:
    // bytes32 digest = keccak256(
    //     abi.encodePacked(
    //         '\x19\x01',
    //         DOMAIN_SEPARATOR,
    //         keccak256(abi.encode(PERMIT_TYPEHASH, owner, spender, value, nonces[owner]++, deadline))
    //     )
    // );
    let permit_hash = keccak256(&digest_input);

    assert_eq!(
        hex::encode(permit_hash),
        "7b90248477de48c0b971e0af8951a55974733455191480e1e117c86cc2a6cd03"
    );
}
