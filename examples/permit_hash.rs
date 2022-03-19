use ethers::{
    contract::{Eip712, EthAbiType},
    core::types::transaction::eip712::Eip712,
    types::{Address, U256},
};

// Generate the EIP712 permit hash to sign for a Uniswap V2 pair.
// <https://eips.ethereum.org/EIPS/eip-712>
// <https://eips.ethereum.org/EIPS/eip-2612>
#[derive(Eip712, EthAbiType, Clone)]
#[eip712(
    name = "Uniswap V2",
    version = "1",
    chain_id = 1,
    verifying_contract = "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"
)]
struct Permit {
    owner: Address,
    spender: Address,
    value: U256,
    nonce: U256,
    deadline: U256,
}

fn main() {
    let permit = Permit {
        owner: Address::random(),
        spender: Address::random(),
        value: 100.into(),
        nonce: 0.into(),
        deadline: U256::MAX,
    };
    let permit_hash = permit.encode_eip712().unwrap();
    println!("Permit hash: 0x{}", hex::encode(permit_hash));
}
