/// Utilities for launching a ganache-cli testnet instance
mod ganache;
pub use ganache::{Ganache, GanacheInstance};

/// Solidity compiler bindings
mod solc;
pub use solc::{CompiledContract, Solc};

mod hash;
pub use hash::{hash_message, id, keccak256, serialize};

/// Re-export RLP
pub use rlp;

use crate::types::{Address, Bytes, U256};
use std::convert::TryInto;

/// 1 Ether = 1e18 Wei
pub const WEI_IN_ETHER: usize = 1000000000000000000;

/// Format the output for the user which prefer to see values
/// in ether (instead of wei)
///
/// Divides the input by 1e18
pub fn format_ether<T: Into<U256>>(amount: T) -> U256 {
    amount.into() / WEI_IN_ETHER
}

/// Divides with the number of decimals
pub fn format_units<T: Into<U256>>(amount: T, decimals: usize) -> U256 {
    amount.into() / decimals
}

/// Converts the input to a U256 and converts from Ether to Wei.
///
/// ```
/// use ethers::{types::U256, utils::{parse_ether, WEI_IN_ETHER}};
///
/// let eth = U256::from(WEI_IN_ETHER);
/// assert_eq!(eth, parse_ether(1u8).unwrap());
/// assert_eq!(eth, parse_ether(1usize).unwrap());
/// assert_eq!(eth, parse_ether("1").unwrap());
pub fn parse_ether<S>(eth: S) -> Result<U256, S::Error>
where
    S: TryInto<U256>,
{
    Ok(eth.try_into()? * WEI_IN_ETHER)
}

/// Multiplies with the number of decimals
pub fn parse_units<S>(eth: S, decimals: usize) -> Result<U256, S::Error>
where
    S: TryInto<U256>,
{
    Ok(eth.try_into()? * decimals)
}

/// The address for an Ethereum contract is deterministically computed from the
/// address of its creator (sender) and how many transactions the creator has
/// sent (nonce). The sender and nonce are RLP encoded and then hashed with Keccak-256.
pub fn get_contract_address(sender: impl Into<Address>, nonce: impl Into<U256>) -> Address {
    let mut stream = rlp::RlpStream::new();
    stream.begin_list(2);
    stream.append(&sender.into());
    stream.append(&nonce.into());

    let hash = keccak256(&stream.out());

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&hash[12..]);
    Address::from(bytes)
}

/// Returns the CREATE2 of a smart contract as specified in
/// [EIP1014](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1014.md)
///
/// keccak256( 0xff ++ senderAddress ++ salt ++ keccak256(init_code))[12:]
pub fn get_create2_address(
    from: impl Into<Address>,
    salt: impl Into<Bytes>,
    init_code: impl Into<Bytes>,
) -> Address {
    let bytes = [
        &[0xff],
        from.into().as_bytes(),
        salt.into().as_ref(),
        &keccak256(init_code.into().as_ref()),
    ]
    .concat();

    let hash = keccak256(&bytes);

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&hash[12..]);
    Address::from(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hex::FromHex;

    #[test]
    fn contract_address() {
        // http://ethereum.stackexchange.com/questions/760/how-is-the-address-of-an-ethereum-contract-computed
        let from = "6ac7ea33f8831ea9dcc53393aaa88b25a785dbf0"
            .parse::<Address>()
            .unwrap();
        for (nonce, expected) in [
            "cd234a471b72ba2f1ccf0a70fcaba648a5eecd8d",
            "343c43a37d37dff08ae8c4a11544c718abb4fcf8",
            "f778b86fa74e846c4f0a1fbd1335fe81c00a0c91",
            "fffd933a0bc612844eaf0c6fe3e5b8e9b6c1d19c",
        ]
        .iter()
        .enumerate()
        {
            let address = get_contract_address(from, nonce);
            assert_eq!(address, expected.parse::<Address>().unwrap());
        }
    }

    #[test]
    // Test vectors from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1014.md#examples
    fn create2_address() {
        for (from, salt, init_code, expected) in &[
            (
                "0000000000000000000000000000000000000000",
                "0000000000000000000000000000000000000000000000000000000000000000",
                "00",
                "4D1A2e2bB4F88F0250f26Ffff098B0b30B26BF38",
            ),
            (
                "deadbeef00000000000000000000000000000000",
                "0000000000000000000000000000000000000000000000000000000000000000",
                "00",
                "B928f69Bb1D91Cd65274e3c79d8986362984fDA3",
            ),
            (
                "deadbeef00000000000000000000000000000000",
                "000000000000000000000000feed000000000000000000000000000000000000",
                "00",
                "D04116cDd17beBE565EB2422F2497E06cC1C9833",
            ),
            (
                "0000000000000000000000000000000000000000",
                "0000000000000000000000000000000000000000000000000000000000000000",
                "deadbeef",
                "70f2b2914A2a4b783FaEFb75f459A580616Fcb5e",
            ),
            (
                "00000000000000000000000000000000deadbeef",
                "00000000000000000000000000000000000000000000000000000000cafebabe",
                "deadbeef",
                "60f3f640a8508fC6a86d45DF051962668E1e8AC7",
            ),
            (
                "00000000000000000000000000000000deadbeef",
                "00000000000000000000000000000000000000000000000000000000cafebabe",
                "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
                "1d8bfDC5D46DC4f61D6b6115972536eBE6A8854C",
            ),
            (
                "0000000000000000000000000000000000000000",
                "0000000000000000000000000000000000000000000000000000000000000000",
                "",
                "E33C0C7F7df4809055C3ebA6c09CFe4BaF1BD9e0",
            ),
        ] {
            let from = from.parse::<Address>().unwrap();
            let salt = salt.from_hex::<Vec<u8>>().unwrap();
            let init_code = init_code.from_hex::<Vec<u8>>().unwrap();
            let expected = expected.parse::<Address>().unwrap();
            assert_eq!(expected, get_create2_address(from, salt, init_code))
        }
    }
}
