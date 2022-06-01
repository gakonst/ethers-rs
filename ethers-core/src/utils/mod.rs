/// Utilities for launching a ganache-cli testnet instance
#[cfg(not(target_arch = "wasm32"))]
mod ganache;
#[cfg(not(target_arch = "wasm32"))]
pub use ganache::{Ganache, GanacheInstance};

/// Utilities for launching a go-ethereum dev-mode instance
#[cfg(not(target_arch = "wasm32"))]
mod geth;
#[cfg(not(target_arch = "wasm32"))]
pub use geth::{Geth, GethInstance};

/// Utilities for launching an anvil instance
#[cfg(not(target_arch = "wasm32"))]
mod anvil;
#[cfg(not(target_arch = "wasm32"))]
pub use anvil::{Anvil, AnvilInstance};

/// Moonbeam utils
pub mod moonbeam;

mod hash;
pub use hash::{hash_message, id, keccak256, serialize};

mod units;
pub use units::Units;

/// Re-export RLP
pub use rlp;

/// Re-export hex
pub use hex;

use crate::types::{Address, Bytes, I256, U256};
use elliptic_curve::sec1::ToEncodedPoint;
use ethabi::ethereum_types::FromDecStrErr;
use k256::{ecdsa::SigningKey, PublicKey as K256PublicKey};
use std::convert::{TryFrom, TryInto};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Unknown units: {0}")]
    UnrecognizedUnits(String),
    #[error("bytes32 strings must not exceed 32 bytes in length")]
    TextTooLong,
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    InvalidFloat(#[from] std::num::ParseFloatError),
    #[error(transparent)]
    FromDecStrError(#[from] FromDecStrErr),
    #[error(transparent)]
    DecimalError(#[from] rust_decimal::Error),
}

/// 1 Ether = 1e18 Wei == 0x0de0b6b3a7640000 Wei
pub const WEI_IN_ETHER: U256 = U256([0x0de0b6b3a7640000, 0x0, 0x0, 0x0]);

/// The number of blocks from the past for which the fee rewards are fetched for fee estimation.
pub const EIP1559_FEE_ESTIMATION_PAST_BLOCKS: u64 = 10;
/// The default percentile of gas premiums that are fetched for fee estimation.
pub const EIP1559_FEE_ESTIMATION_REWARD_PERCENTILE: f64 = 5.0;
/// The default max priority fee per gas, used in case the base fee is within a threshold.
pub const EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE: u64 = 3_000_000_000;
/// The threshold for base fee below which we use the default priority fee, and beyond which we
/// estimate an appropriate value for priority fee.
pub const EIP1559_FEE_ESTIMATION_PRIORITY_FEE_TRIGGER: u64 = 100_000_000_000;
/// The threshold max change/difference (in %) at which we will ignore the fee history values
/// under it.
pub const EIP1559_FEE_ESTIMATION_THRESHOLD_MAX_CHANGE: i64 = 200;

/// Format the output for the user which prefer to see values
/// in ether (instead of wei)
///
/// Divides the input by 1e18
pub fn format_ether<T: Into<U256>>(amount: T) -> U256 {
    amount.into() / WEI_IN_ETHER
}

/// Divides the provided amount with 10^{units} provided.
///
/// ```
/// use ethers_core::{types::U256, utils::format_units};
///
/// let eth = format_units(1395633240123456000_u128, "ether").unwrap();
/// assert_eq!(eth.parse::<f64>().unwrap(), 1.395633240123456);
///
/// let eth = format_units(U256::from_dec_str("1395633240123456000").unwrap(), "ether").unwrap();
/// assert_eq!(eth.parse::<f64>().unwrap(), 1.395633240123456);
///
/// let eth = format_units(U256::from_dec_str("1395633240123456789").unwrap(), "ether").unwrap();
/// assert_eq!(eth, "1.395633240123456789");
/// ```
pub fn format_units<T, K>(amount: T, units: K) -> Result<String, ConversionError>
where
    T: Into<U256>,
    K: TryInto<Units, Error = ConversionError>,
{
    let units = units.try_into()?;
    let amount = amount.into();
    let amount_decimals = amount % U256::from(10_u128.pow(units.as_num()));
    let amount_integer = amount / U256::from(10_u128.pow(units.as_num()));
    Ok(format!(
        "{}.{:0width$}",
        amount_integer,
        amount_decimals.as_u128(),
        width = units.as_num() as usize
    ))
}

/// Converts the input to a U256 and converts from Ether to Wei.
///
/// ```
/// use ethers_core::{types::U256, utils::{parse_ether, WEI_IN_ETHER}};
///
/// let eth = U256::from(WEI_IN_ETHER);
/// assert_eq!(eth, parse_ether(1u8).unwrap());
/// assert_eq!(eth, parse_ether(1usize).unwrap());
/// assert_eq!(eth, parse_ether("1").unwrap());
/// ```
pub fn parse_ether<S>(eth: S) -> Result<U256, ConversionError>
where
    S: ToString,
{
    parse_units(eth, "ether")
}

/// Multiplies the provided amount with 10^{units} provided.
///
/// ```
/// use ethers_core::{types::U256, utils::parse_units};
/// let amount_in_eth = U256::from_dec_str("15230001000000000000").unwrap();
/// let amount_in_gwei = U256::from_dec_str("15230001000").unwrap();
/// let amount_in_wei = U256::from_dec_str("15230001000").unwrap();
/// assert_eq!(amount_in_eth, parse_units("15.230001000000000000", "ether").unwrap());
/// assert_eq!(amount_in_gwei, parse_units("15.230001000000000000", "gwei").unwrap());
/// assert_eq!(amount_in_wei, parse_units("15230001000", "wei").unwrap());
/// ```
/// Example of trying to parse decimal WEI, which should fail, as WEI is the smallest
/// ETH denominator. 1 ETH = 10^18 WEI.
/// ```should_panic
/// use ethers_core::{types::U256, utils::parse_units};
/// let amount_in_wei = U256::from_dec_str("15230001000").unwrap();
/// assert_eq!(amount_in_wei, parse_units("15.230001000000000000", "wei").unwrap());
/// ```
pub fn parse_units<K, S>(amount: S, units: K) -> Result<U256, ConversionError>
where
    S: ToString,
    K: TryInto<Units, Error = ConversionError> + Copy,
{
    use rust_decimal::Decimal;
    let num: Decimal = amount.to_string().parse()?;
    let multiplier: Decimal = 10u64.pow(units.try_into()?.as_num()).into();
    let val =
        num.checked_mul(multiplier).ok_or(rust_decimal::Error::ExceedsMaximumPossibleValue)?;
    let u256_n: U256 = U256::from_dec_str(&val.round().to_string())?;
    Ok(u256_n)
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

/// Returns the CREATE2 address of a smart contract as specified in
/// [EIP1014](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1014.md)
///
/// keccak256( 0xff ++ senderAddress ++ salt ++ keccak256(init_code))[12..]
pub fn get_create2_address(
    from: impl Into<Address>,
    salt: impl Into<Bytes>,
    init_code: impl Into<Bytes>,
) -> Address {
    get_create2_address_from_hash(from, salt, keccak256(init_code.into().as_ref()).to_vec())
}

/// Returns the CREATE2 address of a smart contract as specified in
/// [EIP1014](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1014.md),
/// taking the pre-computed hash of the init code as input.
///
/// keccak256( 0xff ++ senderAddress ++ salt ++ keccak256(init_code))[12..]
///
/// # Example
///
/// Calculate the address of a UniswapV3 pool.
///
/// ```
/// use ethers_core::{
///     abi,
///     abi::Token,
///     types::{Address, Bytes, U256},
///     utils::{get_create2_address_from_hash, keccak256},
/// };
///
/// let UNISWAP_V3_POOL_INIT_CODE_HASH = Bytes::from(
///     hex::decode("e34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54").unwrap(),
/// );
/// let factory: Address = "0x1F98431c8aD98523631AE4a59f267346ea31F984"
///     .parse()
///     .unwrap();
/// let token0: Address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
///     .parse()
///     .unwrap();
/// let token1: Address = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
///     .parse()
///     .unwrap();
/// let fee = 500;
///
/// // abi.encode(token0 as address, token1 as address, fee as uint256)
/// let input = abi::encode(&vec![
///     Token::Address(token0),
///     Token::Address(token1),
///     Token::Uint(U256::from(fee)),
/// ]);
///
/// // keccak256(abi.encode(token0, token1, fee))
/// let salt = keccak256(&input);
/// let pool_address =
///     get_create2_address_from_hash(factory, salt.to_vec(), UNISWAP_V3_POOL_INIT_CODE_HASH);
///
/// assert_eq!(
///     pool_address,
///     "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640" // USDC/ETH pool address
///         .parse()
///         .unwrap()
/// );
/// ```
pub fn get_create2_address_from_hash(
    from: impl Into<Address>,
    salt: impl Into<Bytes>,
    init_code_hash: impl Into<Bytes>,
) -> Address {
    let bytes =
        [&[0xff], from.into().as_bytes(), salt.into().as_ref(), init_code_hash.into().as_ref()]
            .concat();

    let hash = keccak256(&bytes);

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&hash[12..]);
    Address::from(bytes)
}

/// Converts a K256 SigningKey to an Ethereum Address
pub fn secret_key_to_address(secret_key: &SigningKey) -> Address {
    let public_key = K256PublicKey::from(&secret_key.verifying_key());
    let public_key = public_key.to_encoded_point(/* compress = */ false);
    let public_key = public_key.as_bytes();
    debug_assert_eq!(public_key[0], 0x04);
    let hash = keccak256(&public_key[1..]);
    Address::from_slice(&hash[12..])
}

/// Converts an Ethereum address to the checksum encoding
/// Ref: <https://github.com/ethereum/EIPs/blob/master/EIPS/eip-55.md>
pub fn to_checksum(addr: &Address, chain_id: Option<u8>) -> String {
    let prefixed_addr = match chain_id {
        Some(chain_id) => format!("{}0x{:x}", chain_id, addr),
        None => format!("{:x}", addr),
    };
    let hash = hex::encode(keccak256(&prefixed_addr));
    let hash = hash.as_bytes();

    let addr_hex = hex::encode(addr.as_bytes());
    let addr_hex = addr_hex.as_bytes();

    addr_hex.iter().zip(hash).fold("0x".to_owned(), |mut encoded, (addr, hash)| {
        encoded.push(if *hash >= 56 {
            addr.to_ascii_uppercase() as char
        } else {
            addr.to_ascii_lowercase() as char
        });
        encoded
    })
}

/// Returns a bytes32 string representation of text. If the length of text exceeds 32 bytes,
/// an error is returned.
pub fn format_bytes32_string(text: &str) -> Result<[u8; 32], ConversionError> {
    let str_bytes: &[u8] = text.as_bytes();
    if str_bytes.len() > 32 {
        return Err(ConversionError::TextTooLong)
    }

    let mut bytes32: [u8; 32] = [0u8; 32];
    bytes32[..str_bytes.len()].copy_from_slice(str_bytes);

    Ok(bytes32)
}

/// Returns the decoded string represented by the bytes32 encoded data.
pub fn parse_bytes32_string(bytes: &[u8; 32]) -> Result<&str, ConversionError> {
    let mut length = 0;
    while length < 32 && bytes[length] != 0 {
        length += 1;
    }

    Ok(std::str::from_utf8(&bytes[..length])?)
}

/// The default EIP-1559 fee estimator which is based on the work by [MyCrypto](https://github.com/MyCryptoHQ/MyCrypto/blob/master/src/services/ApiService/Gas/eip1559.ts)
pub fn eip1559_default_estimator(base_fee_per_gas: U256, rewards: Vec<Vec<U256>>) -> (U256, U256) {
    let max_priority_fee_per_gas =
        if base_fee_per_gas < U256::from(EIP1559_FEE_ESTIMATION_PRIORITY_FEE_TRIGGER) {
            U256::from(EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE)
        } else {
            std::cmp::max(
                estimate_priority_fee(rewards),
                U256::from(EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE),
            )
        };
    let potential_max_fee = base_fee_surged(base_fee_per_gas);
    let max_fee_per_gas = if max_priority_fee_per_gas > potential_max_fee {
        max_priority_fee_per_gas + potential_max_fee
    } else {
        potential_max_fee
    };
    (max_fee_per_gas, max_priority_fee_per_gas)
}

fn estimate_priority_fee(rewards: Vec<Vec<U256>>) -> U256 {
    let mut rewards: Vec<U256> =
        rewards.iter().map(|r| r[0]).filter(|r| *r > U256::zero()).collect();
    if rewards.is_empty() {
        return U256::zero()
    }
    if rewards.len() == 1 {
        return rewards[0]
    }
    // Sort the rewards as we will eventually take the median.
    rewards.sort();

    // A copy of the same vector is created for convenience to calculate percentage change
    // between subsequent fee values.
    let mut rewards_copy = rewards.clone();
    rewards_copy.rotate_left(1);

    let mut percentage_change: Vec<I256> = rewards
        .iter()
        .zip(rewards_copy.iter())
        .map(|(a, b)| {
            let a = I256::try_from(*a).expect("priority fee overflow");
            let b = I256::try_from(*b).expect("priority fee overflow");
            ((b - a) * 100.into()) / a
        })
        .collect();
    percentage_change.pop();

    // Fetch the max of the percentage change, and that element's index.
    let max_change = percentage_change.iter().max().unwrap();
    let max_change_index = percentage_change.iter().position(|&c| c == *max_change).unwrap();

    // If we encountered a big change in fees at a certain position, then consider only
    // the values >= it.
    let values = if *max_change >= EIP1559_FEE_ESTIMATION_THRESHOLD_MAX_CHANGE.into() &&
        (max_change_index >= (rewards.len() / 2))
    {
        rewards[max_change_index..].to_vec()
    } else {
        rewards
    };

    // Return the median.
    values[values.len() / 2]
}

fn base_fee_surged(base_fee_per_gas: U256) -> U256 {
    if base_fee_per_gas <= U256::from(40_000_000_000u64) {
        base_fee_per_gas * 2
    } else if base_fee_per_gas <= U256::from(100_000_000_000u64) {
        base_fee_per_gas * 16 / 10
    } else if base_fee_per_gas <= U256::from(200_000_000_000u64) {
        base_fee_per_gas * 14 / 10
    } else {
        base_fee_per_gas * 12 / 10
    }
}

/// A bit of hack to find an unused TCP port.
///
/// Does not guarantee that the given port is unused after the function exists, just that it was
/// unused before the function started (i.e., it does not reserve a port).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn unused_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .expect("Failed to create TCP listener to find unused port");

    let local_addr =
        listener.local_addr().expect("Failed to read TCP listener local_addr to find unused port");
    local_addr.port()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn wei_in_ether() {
        assert_eq!(WEI_IN_ETHER.as_u64(), 1e18 as u64);
    }

    #[test]
    fn test_format_units() {
        let gwei_in_ether = format_units(WEI_IN_ETHER, 9).unwrap();
        assert_eq!(gwei_in_ether.parse::<f64>().unwrap() as u64, 1e9 as u64);

        let eth = format_units(WEI_IN_ETHER, "ether").unwrap();
        assert_eq!(eth.parse::<f64>().unwrap() as u64, 1);

        let eth = format_units(1395633240123456000_u128, "ether").unwrap();
        assert_eq!(eth.parse::<f64>().unwrap(), 1.395633240123456);

        let eth =
            format_units(U256::from_dec_str("1395633240123456000").unwrap(), "ether").unwrap();
        assert_eq!(eth.parse::<f64>().unwrap(), 1.395633240123456);

        let eth =
            format_units(U256::from_dec_str("1395633240123456789").unwrap(), "ether").unwrap();
        assert_eq!(eth, "1.395633240123456789");

        let eth =
            format_units(U256::from_dec_str("1005633240123456789").unwrap(), "ether").unwrap();
        assert_eq!(eth, "1.005633240123456789");
    }

    #[test]
    fn test_parse_units() {
        let gwei = parse_units(1.5, 9).unwrap();
        assert_eq!(gwei.as_u64(), 15e8 as u64);

        let token = parse_units(1163.56926418, 8).unwrap();
        assert_eq!(token.as_u64(), 116356926418);

        let eth_dec_float = parse_units(1.39563324, "ether").unwrap();
        assert_eq!(eth_dec_float, U256::from_dec_str("1395633240000000000").unwrap());

        let eth_dec_string = parse_units("1.39563324", "ether").unwrap();
        assert_eq!(eth_dec_string, U256::from_dec_str("1395633240000000000").unwrap());

        let eth = parse_units(1, "ether").unwrap();
        assert_eq!(eth, WEI_IN_ETHER);

        let val = parse_units("2.3", "ether").unwrap();
        assert_eq!(val, U256::from_dec_str("2300000000000000000").unwrap());
    }

    #[test]
    fn addr_checksum() {
        let addr_list = vec![
            // mainnet
            (
                None,
                "27b1fdb04752bbc536007a920d24acb045561c26",
                "0x27b1fdb04752bbc536007a920d24acb045561c26",
            ),
            (
                None,
                "3599689e6292b81b2d85451025146515070129bb",
                "0x3599689E6292b81B2d85451025146515070129Bb",
            ),
            (
                None,
                "42712d45473476b98452f434e72461577d686318",
                "0x42712D45473476b98452f434e72461577D686318",
            ),
            (
                None,
                "52908400098527886e0f7030069857d2e4169ee7",
                "0x52908400098527886E0F7030069857D2E4169EE7",
            ),
            (
                None,
                "5aaeb6053f3e94c9b9a09f33669435e7ef1beaed",
                "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
            ),
            (
                None,
                "6549f4939460de12611948b3f82b88c3c8975323",
                "0x6549f4939460DE12611948b3f82b88C3C8975323",
            ),
            (
                None,
                "66f9664f97f2b50f62d13ea064982f936de76657",
                "0x66f9664f97F2b50F62D13eA064982f936dE76657",
            ),
            (
                None,
                "88021160c5c792225e4e5452585947470010289d",
                "0x88021160C5C792225E4E5452585947470010289D",
            ),
            // rsk mainnet
            (
                Some(30),
                "27b1fdb04752bbc536007a920d24acb045561c26",
                "0x27b1FdB04752BBc536007A920D24ACB045561c26",
            ),
            (
                Some(30),
                "3599689e6292b81b2d85451025146515070129bb",
                "0x3599689E6292B81B2D85451025146515070129Bb",
            ),
            (
                Some(30),
                "42712d45473476b98452f434e72461577d686318",
                "0x42712D45473476B98452f434E72461577d686318",
            ),
            (
                Some(30),
                "52908400098527886e0f7030069857d2e4169ee7",
                "0x52908400098527886E0F7030069857D2E4169ee7",
            ),
            (
                Some(30),
                "5aaeb6053f3e94c9b9a09f33669435e7ef1beaed",
                "0x5aaEB6053f3e94c9b9a09f33669435E7ef1bEAeD",
            ),
            (
                Some(30),
                "6549f4939460de12611948b3f82b88c3c8975323",
                "0x6549F4939460DE12611948B3F82B88C3C8975323",
            ),
            (
                Some(30),
                "66f9664f97f2b50f62d13ea064982f936de76657",
                "0x66F9664f97f2B50F62d13EA064982F936de76657",
            ),
        ];

        for (chain_id, addr, checksummed_addr) in addr_list {
            let addr = addr.parse::<Address>().unwrap();
            assert_eq!(to_checksum(&addr, chain_id), String::from(checksummed_addr));
        }
    }

    #[test]
    fn contract_address() {
        // http://ethereum.stackexchange.com/questions/760/how-is-the-address-of-an-ethereum-contract-computed
        let from = "6ac7ea33f8831ea9dcc53393aaa88b25a785dbf0".parse::<Address>().unwrap();
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
            // get_create2_address()
            let from = from.parse::<Address>().unwrap();
            let salt = hex::decode(salt).unwrap();
            let init_code = hex::decode(init_code).unwrap();
            let expected = expected.parse::<Address>().unwrap();
            assert_eq!(expected, get_create2_address(from, salt.clone(), init_code.clone()));

            // get_create2_address_from_hash()
            let init_code_hash = keccak256(init_code).to_vec();
            assert_eq!(expected, get_create2_address_from_hash(from, salt, init_code_hash))
        }
    }

    #[test]
    fn bytes32_string_parsing() {
        let text_bytes_list = vec![
            ("", hex!("0000000000000000000000000000000000000000000000000000000000000000")),
            ("A", hex!("4100000000000000000000000000000000000000000000000000000000000000")),
            (
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ012345",
                hex!("4142434445464748494a4b4c4d4e4f505152535455565758595a303132333435"),
            ),
            (
                "!@#$%^&*(),./;'[]",
                hex!("21402324255e262a28292c2e2f3b275b5d000000000000000000000000000000"),
            ),
        ];

        for (text, bytes) in text_bytes_list {
            assert_eq!(text, parse_bytes32_string(&bytes).unwrap());
        }
    }

    #[test]
    fn bytes32_string_formatting() {
        let text_bytes_list = vec![
            ("", hex!("0000000000000000000000000000000000000000000000000000000000000000")),
            ("A", hex!("4100000000000000000000000000000000000000000000000000000000000000")),
            (
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ012345",
                hex!("4142434445464748494a4b4c4d4e4f505152535455565758595a303132333435"),
            ),
            (
                "!@#$%^&*(),./;'[]",
                hex!("21402324255e262a28292c2e2f3b275b5d000000000000000000000000000000"),
            ),
        ];

        for (text, bytes) in text_bytes_list {
            assert_eq!(bytes, format_bytes32_string(text).unwrap());
        }
    }

    #[test]
    fn bytes32_string_formatting_too_long() {
        assert!(matches!(
            format_bytes32_string("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456").unwrap_err(),
            ConversionError::TextTooLong
        ));
    }

    #[test]
    fn test_eip1559_default_estimator() {
        // If the base fee is below the triggering base fee, we should get the default priority fee
        // with the base fee surged.
        let base_fee_per_gas = U256::from(EIP1559_FEE_ESTIMATION_PRIORITY_FEE_TRIGGER) - 1;
        let rewards: Vec<Vec<U256>> = vec![vec![]];
        let (base_fee, priority_fee) = eip1559_default_estimator(base_fee_per_gas, rewards);
        assert_eq!(priority_fee, U256::from(EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE));
        assert_eq!(base_fee, base_fee_surged(base_fee_per_gas));

        // If the base fee is above the triggering base fee, we calculate the priority fee using
        // the fee history (rewards).
        let base_fee_per_gas = U256::from(EIP1559_FEE_ESTIMATION_PRIORITY_FEE_TRIGGER) + 1;
        let rewards: Vec<Vec<U256>> = vec![
            vec![100_000_000_000u64.into()],
            vec![105_000_000_000u64.into()],
            vec![102_000_000_000u64.into()],
        ]; // say, last 3 blocks
        let (base_fee, priority_fee) = eip1559_default_estimator(base_fee_per_gas, rewards.clone());
        assert_eq!(base_fee, base_fee_surged(base_fee_per_gas));
        assert_eq!(priority_fee, estimate_priority_fee(rewards.clone()));

        // The median should be taken because none of the changes are big enough to ignore values.
        assert_eq!(estimate_priority_fee(rewards), 102_000_000_000u64.into());

        // Ensure fee estimation doesn't panic when overflowing a u32. This had been a divide by
        // zero.
        let overflow = U256::from(u32::MAX) + 1;
        let rewards_overflow: Vec<Vec<U256>> = vec![vec![overflow], vec![overflow]];
        assert_eq!(estimate_priority_fee(rewards_overflow), overflow);
    }
}
