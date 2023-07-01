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

/// Utilities for working with a `genesis.json` and other chain config structs.
mod genesis;
pub use genesis::{ChainConfig, CliqueConfig, EthashConfig, Genesis, GenesisAccount};

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
use serde::{Deserialize, Deserializer};
pub use units::Units;

/// Re-export RLP
pub use rlp;

/// Re-export hex
pub use hex;

use crate::types::{Address, Bytes, ParseI256Error, H256, I256, U256};
use ethabi::ethereum_types::FromDecStrErr;
use k256::ecdsa::SigningKey;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    fmt,
};
use thiserror::Error;

/// I256 overflows for numbers wider than 77 units.
const OVERFLOW_I256_UNITS: usize = 77;
/// U256 overflows for numbers wider than 78 units.
const OVERFLOW_U256_UNITS: usize = 78;

// Re-export serde-json for macro usage
#[doc(hidden)]
pub use serde_json as __serde_json;

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
    #[error("Overflow parsing string")]
    ParseOverflow,
    #[error(transparent)]
    ParseI256Error(#[from] ParseI256Error),
    #[error("Invalid address checksum")]
    InvalidAddressChecksum,
    #[error(transparent)]
    FromHexError(<Address as std::str::FromStr>::Err),
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

/// This enum holds the numeric types that a possible to be returned by `parse_units` and
/// that are taken by `format_units`.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ParseUnits {
    U256(U256),
    I256(I256),
}

impl From<ParseUnits> for U256 {
    fn from(n: ParseUnits) -> Self {
        match n {
            ParseUnits::U256(n) => n,
            ParseUnits::I256(n) => n.into_raw(),
        }
    }
}

impl fmt::Display for ParseUnits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseUnits::U256(val) => val.fmt(f),
            ParseUnits::I256(val) => val.fmt(f),
        }
    }
}

macro_rules! construct_format_units_from {
    ($( $t:ty[$convert:ident] ),*) => {
        $(
            impl From<$t> for ParseUnits {
                fn from(num: $t) -> Self {
                    Self::$convert(num.into())
                }
            }
        )*
    }
}

// Generate the From<T> code for the given numeric types below.
construct_format_units_from! {
    u8[U256], u16[U256], u32[U256], u64[U256], u128[U256], U256[U256], usize[U256],
    i8[I256], i16[I256], i32[I256], i64[I256], i128[I256], I256[I256], isize[I256]
}

/// Format the output for the user which prefer to see values
/// in ether (instead of wei)
///
/// Divides the input by 1e18
/// ```
/// use ethers_core::{types::U256, utils::format_ether};
///
/// let eth = format_ether(1395633240123456000_u128);
/// assert_eq!(eth.parse::<f64>().unwrap(), 1.395633240123456);
/// ```
pub fn format_ether<T: Into<ParseUnits>>(amount: T) -> String {
    // format_units returns Err only if units >= 77. Hense, we can safely unwrap here
    format_units(amount, "ether").unwrap()
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
///
/// let eth = format_units(i64::MIN, "gwei").unwrap();
/// assert_eq!(eth, "-9223372036.854775808");
///
/// let eth = format_units(i128::MIN, 36).unwrap();
/// assert_eq!(eth, "-170.141183460469231731687303715884105728");
/// ```
pub fn format_units<T, K>(amount: T, units: K) -> Result<String, ConversionError>
where
    T: Into<ParseUnits>,
    K: TryInto<Units, Error = ConversionError>,
{
    let units: usize = units.try_into()?.into();
    let amount = amount.into();

    match amount {
        // 2**256 ~= 1.16e77
        ParseUnits::U256(_) if units >= OVERFLOW_U256_UNITS => {
            return Err(ConversionError::ParseOverflow)
        }
        // 2**255 ~= 5.79e76
        ParseUnits::I256(_) if units >= OVERFLOW_I256_UNITS => {
            return Err(ConversionError::ParseOverflow)
        }
        _ => {}
    };
    let exp10 = U256::exp10(units);

    // `decimals` are formatted twice because U256 does not support alignment (`:0>width`).
    match amount {
        ParseUnits::U256(amount) => {
            let integer = amount / exp10;
            let decimals = (amount % exp10).to_string();
            Ok(format!("{integer}.{decimals:0>units$}"))
        }
        ParseUnits::I256(amount) => {
            let exp10 = I256::from_raw(exp10);
            let sign = if amount.is_negative() { "-" } else { "" };
            let integer = (amount / exp10).twos_complement();
            let decimals = ((amount % exp10).twos_complement()).to_string();
            Ok(format!("{sign}{integer}.{decimals:0>units$}"))
        }
    }
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
pub fn parse_ether<S: ToString>(eth: S) -> Result<U256, ConversionError> {
    Ok(parse_units(eth, "ether")?.into())
}

/// Multiplies the provided amount with 10^{units} provided.
///
/// ```
/// use ethers_core::{types::U256, utils::parse_units};
/// let amount_in_eth = U256::from_dec_str("15230001000000000000").unwrap();
/// let amount_in_gwei = U256::from_dec_str("15230001000").unwrap();
/// let amount_in_wei = U256::from_dec_str("15230001000").unwrap();
/// assert_eq!(amount_in_eth, parse_units("15.230001000000000000", "ether").unwrap().into());
/// assert_eq!(amount_in_gwei, parse_units("15.230001000000000000", "gwei").unwrap().into());
/// assert_eq!(amount_in_wei, parse_units("15230001000", "wei").unwrap().into());
/// ```
/// Example of trying to parse decimal WEI, which should fail, as WEI is the smallest
/// ETH denominator. 1 ETH = 10^18 WEI.
/// ```should_panic
/// use ethers_core::{types::U256, utils::parse_units};
/// let amount_in_wei = U256::from_dec_str("15230001000").unwrap();
/// assert_eq!(amount_in_wei, parse_units("15.230001000000000000", "wei").unwrap().into());
/// ```
pub fn parse_units<K, S>(amount: S, units: K) -> Result<ParseUnits, ConversionError>
where
    S: ToString,
    K: TryInto<Units, Error = ConversionError> + Copy,
{
    let exponent: u32 = units.try_into()?.as_num();
    let mut amount_str = amount.to_string().replace('_', "");
    let negative = amount_str.chars().next().unwrap_or_default() == '-';
    let dec_len = if let Some(di) = amount_str.find('.') {
        amount_str.remove(di);
        amount_str[di..].len() as u32
    } else {
        0
    };

    if dec_len > exponent {
        // Truncate the decimal part if it is longer than the exponent
        let amount_str = &amount_str[..(amount_str.len() - (dec_len - exponent) as usize)];
        if negative {
            // Edge case: We have removed the entire number and only the negative sign is left.
            //            Return 0 as a I256 given the input was signed.
            if amount_str == "-" {
                Ok(ParseUnits::I256(I256::zero()))
            } else {
                Ok(ParseUnits::I256(I256::from_dec_str(amount_str)?))
            }
        } else {
            Ok(ParseUnits::U256(U256::from_dec_str(amount_str)?))
        }
    } else if negative {
        // Edge case: Only a negative sign was given, return 0 as a I256 given the input was signed.
        if amount_str == "-" {
            Ok(ParseUnits::I256(I256::zero()))
        } else {
            let mut n = I256::from_dec_str(&amount_str)?;
            n *= I256::from(10)
                .checked_pow(exponent - dec_len)
                .ok_or(ConversionError::ParseOverflow)?;
            Ok(ParseUnits::I256(n))
        }
    } else {
        let mut a_uint = U256::from_dec_str(&amount_str)?;
        a_uint *= U256::from(10)
            .checked_pow(U256::from(exponent - dec_len))
            .ok_or(ConversionError::ParseOverflow)?;
        Ok(ParseUnits::U256(a_uint))
    }
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
    salt: impl AsRef<[u8]>,
    init_code: impl AsRef<[u8]>,
) -> Address {
    let init_code_hash = keccak256(init_code.as_ref());
    get_create2_address_from_hash(from, salt, init_code_hash)
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
/// let init_code_hash = hex::decode("e34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54").unwrap();
/// let factory: Address = "0x1F98431c8aD98523631AE4a59f267346ea31F984"
///     .parse()
///     .unwrap();
/// let token0: Address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
///     .parse()
///     .unwrap();
/// let token1: Address = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
///     .parse()
///     .unwrap();
/// let fee = U256::from(500_u64);
///
/// // abi.encode(token0 as address, token1 as address, fee as uint256)
/// let input = abi::encode(&[
///     Token::Address(token0),
///     Token::Address(token1),
///     Token::Uint(fee),
/// ]);
///
/// // keccak256(abi.encode(token0, token1, fee))
/// let salt = keccak256(&input);
/// let pool_address = get_create2_address_from_hash(factory, salt, init_code_hash);
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
    salt: impl AsRef<[u8]>,
    init_code_hash: impl AsRef<[u8]>,
) -> Address {
    let from = from.into();
    let salt = salt.as_ref();
    let init_code_hash = init_code_hash.as_ref();

    let mut bytes = Vec::with_capacity(1 + 20 + salt.len() + init_code_hash.len());
    bytes.push(0xff);
    bytes.extend_from_slice(from.as_bytes());
    bytes.extend_from_slice(salt);
    bytes.extend_from_slice(init_code_hash);

    let hash = keccak256(bytes);

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&hash[12..]);
    Address::from(bytes)
}

/// Converts a K256 SigningKey to an Ethereum Address
pub fn secret_key_to_address(secret_key: &SigningKey) -> Address {
    let public_key = secret_key.verifying_key();
    let public_key = public_key.to_encoded_point(/* compress = */ false);
    let public_key = public_key.as_bytes();
    debug_assert_eq!(public_key[0], 0x04);
    let hash = keccak256(&public_key[1..]);

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&hash[12..]);
    Address::from(bytes)
}

/// Encodes an Ethereum address to its [EIP-55] checksum.
///
/// You can optionally specify an [EIP-155 chain ID] to encode the address using the [EIP-1191]
/// extension.
///
/// [EIP-55]: https://eips.ethereum.org/EIPS/eip-55
/// [EIP-155 chain ID]: https://eips.ethereum.org/EIPS/eip-155
/// [EIP-1191]: https://eips.ethereum.org/EIPS/eip-1191
pub fn to_checksum(addr: &Address, chain_id: Option<u8>) -> String {
    let prefixed_addr = match chain_id {
        Some(chain_id) => format!("{chain_id}0x{addr:x}"),
        None => format!("{addr:x}"),
    };
    let hash = hex::encode(keccak256(prefixed_addr));
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

/// Parses an [EIP-1191](https://eips.ethereum.org/EIPS/eip-1191) checksum address.
///
/// Returns `Ok(address)` if the checksummed address is valid, `Err()` otherwise.
/// If `chain_id` is `None`, falls back to [EIP-55](https://eips.ethereum.org/EIPS/eip-55) address checksum method
pub fn parse_checksummed(addr: &str, chain_id: Option<u8>) -> Result<Address, ConversionError> {
    let addr = addr.strip_prefix("0x").unwrap_or(addr);
    let address: Address = addr.parse().map_err(ConversionError::FromHexError)?;
    let checksum_addr = to_checksum(&address, chain_id);

    if checksum_addr.strip_prefix("0x").unwrap_or(&checksum_addr) == addr {
        Ok(address)
    } else {
        Err(ConversionError::InvalidAddressChecksum)
    }
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

/// Converts a Bytes value into a H256, accepting inputs that are less than 32 bytes long. These
/// inputs will be left padded with zeros.
pub fn from_bytes_to_h256<'de, D>(bytes: Bytes) -> Result<H256, D::Error>
where
    D: Deserializer<'de>,
{
    if bytes.0.len() > 32 {
        return Err(serde::de::Error::custom("input too long to be a H256"))
    }

    // left pad with zeros to 32 bytes
    let mut padded = [0u8; 32];
    padded[32 - bytes.0.len()..].copy_from_slice(&bytes.0);

    // then convert to H256 without a panic
    Ok(H256::from_slice(&padded))
}

/// Deserializes the input into an Option<HashMap<H256, H256>>, using from_unformatted_hex to
/// deserialize the keys and values.
pub fn from_unformatted_hex_map<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<H256, H256>>, D::Error>
where
    D: Deserializer<'de>,
{
    let map = Option::<HashMap<Bytes, Bytes>>::deserialize(deserializer)?;
    match map {
        Some(mut map) => {
            let mut res_map = HashMap::new();
            for (k, v) in map.drain() {
                let k_deserialized = from_bytes_to_h256::<'de, D>(k)?;
                let v_deserialized = from_bytes_to_h256::<'de, D>(v)?;
                res_map.insert(k_deserialized, v_deserialized);
            }
            Ok(Some(res_map))
        }
        None => Ok(None),
    }
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
            ((b - a) * 100) / a
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
    use crate::types::serde_helpers::deserialize_stringified_numeric;
    use hex_literal::hex;

    #[test]
    fn wei_in_ether() {
        assert_eq!(WEI_IN_ETHER.as_u64(), 1e18 as u64);
    }

    #[test]
    fn test_format_ether_unsigned() {
        let eth = format_ether(WEI_IN_ETHER);
        assert_eq!(eth.parse::<f64>().unwrap() as u64, 1);

        let eth = format_ether(1395633240123456000_u128);
        assert_eq!(eth.parse::<f64>().unwrap(), 1.395633240123456);

        let eth = format_ether(U256::from_dec_str("1395633240123456000").unwrap());
        assert_eq!(eth.parse::<f64>().unwrap(), 1.395633240123456);

        let eth = format_ether(U256::from_dec_str("1395633240123456789").unwrap());
        assert_eq!(eth, "1.395633240123456789");

        let eth = format_ether(U256::from_dec_str("1005633240123456789").unwrap());
        assert_eq!(eth, "1.005633240123456789");

        let eth = format_ether(u16::MAX);
        assert_eq!(eth, "0.000000000000065535");

        // Note: This covers usize on 32 bit systems.
        let eth = format_ether(u32::MAX);
        assert_eq!(eth, "0.000000004294967295");

        // Note: This covers usize on 64 bit systems.
        let eth = format_ether(u64::MAX);
        assert_eq!(eth, "18.446744073709551615");
    }

    #[test]
    fn test_format_ether_signed() {
        let eth = format_ether(I256::from_dec_str("-1395633240123456000").unwrap());
        assert_eq!(eth.parse::<f64>().unwrap(), -1.395633240123456);

        let eth = format_ether(I256::from_dec_str("-1395633240123456789").unwrap());
        assert_eq!(eth, "-1.395633240123456789");

        let eth = format_ether(I256::from_dec_str("1005633240123456789").unwrap());
        assert_eq!(eth, "1.005633240123456789");

        let eth = format_ether(i8::MIN);
        assert_eq!(eth, "-0.000000000000000128");

        let eth = format_ether(i8::MAX);
        assert_eq!(eth, "0.000000000000000127");

        let eth = format_ether(i16::MIN);
        assert_eq!(eth, "-0.000000000000032768");

        // Note: This covers isize on 32 bit systems.
        let eth = format_ether(i32::MIN);
        assert_eq!(eth, "-0.000000002147483648");

        // Note: This covers isize on 64 bit systems.
        let eth = format_ether(i64::MIN);
        assert_eq!(eth, "-9.223372036854775808");
    }

    #[test]
    fn test_format_units_unsigned() {
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

        let eth = format_units(u8::MAX, 4).unwrap();
        assert_eq!(eth, "0.0255");

        let eth = format_units(u16::MAX, "ether").unwrap();
        assert_eq!(eth, "0.000000000000065535");

        // Note: This covers usize on 32 bit systems.
        let eth = format_units(u32::MAX, 18).unwrap();
        assert_eq!(eth, "0.000000004294967295");

        // Note: This covers usize on 64 bit systems.
        let eth = format_units(u64::MAX, "gwei").unwrap();
        assert_eq!(eth, "18446744073.709551615");

        let eth = format_units(u128::MAX, 36).unwrap();
        assert_eq!(eth, "340.282366920938463463374607431768211455");

        let eth = format_units(U256::MAX, 77).unwrap();
        assert_eq!(
            eth,
            "1.15792089237316195423570985008687907853269984665640564039457584007913129639935"
        );

        let err = format_units(U256::MAX, 78).unwrap_err();
        assert!(matches!(err, ConversionError::ParseOverflow));
    }

    #[test]
    fn test_format_units_signed() {
        let eth =
            format_units(I256::from_dec_str("-1395633240123456000").unwrap(), "ether").unwrap();
        assert_eq!(eth.parse::<f64>().unwrap(), -1.395633240123456);

        let eth =
            format_units(I256::from_dec_str("-1395633240123456789").unwrap(), "ether").unwrap();
        assert_eq!(eth, "-1.395633240123456789");

        let eth =
            format_units(I256::from_dec_str("1005633240123456789").unwrap(), "ether").unwrap();
        assert_eq!(eth, "1.005633240123456789");

        let eth = format_units(i8::MIN, 4).unwrap();
        assert_eq!(eth, "-0.0128");
        assert_eq!(eth.parse::<f64>().unwrap(), -0.0128_f64);

        let eth = format_units(i8::MAX, 4).unwrap();
        assert_eq!(eth, "0.0127");
        assert_eq!(eth.parse::<f64>().unwrap(), 0.0127);

        let eth = format_units(i16::MIN, "ether").unwrap();
        assert_eq!(eth, "-0.000000000000032768");

        // Note: This covers isize on 32 bit systems.
        let eth = format_units(i32::MIN, 18).unwrap();
        assert_eq!(eth, "-0.000000002147483648");

        // Note: This covers isize on 64 bit systems.
        let eth = format_units(i64::MIN, "gwei").unwrap();
        assert_eq!(eth, "-9223372036.854775808");

        let eth = format_units(i128::MIN, 36).unwrap();
        assert_eq!(eth, "-170.141183460469231731687303715884105728");

        let eth = format_units(I256::MIN, 76).unwrap();
        assert_eq!(
            eth,
            "-5.7896044618658097711785492504343953926634992332820282019728792003956564819968"
        );

        let err = format_units(I256::MIN, 77).unwrap_err();
        assert!(matches!(err, ConversionError::ParseOverflow));
    }

    #[test]
    fn parse_large_units() {
        let decimals = 27u32;
        let val = "10.55";

        let n: U256 = parse_units(val, decimals).unwrap().into();
        assert_eq!(n.to_string(), "10550000000000000000000000000");
    }

    #[test]
    fn test_parse_units() {
        let gwei: U256 = parse_units(1.5, 9).unwrap().into();
        assert_eq!(gwei.as_u64(), 15e8 as u64);

        let token: U256 = parse_units(1163.56926418, 8).unwrap().into();
        assert_eq!(token.as_u64(), 116356926418);

        let eth_dec_float: U256 = parse_units(1.39563324, "ether").unwrap().into();
        assert_eq!(eth_dec_float, U256::from_dec_str("1395633240000000000").unwrap());

        let eth_dec_string: U256 = parse_units("1.39563324", "ether").unwrap().into();
        assert_eq!(eth_dec_string, U256::from_dec_str("1395633240000000000").unwrap());

        let eth: U256 = parse_units(1, "ether").unwrap().into();
        assert_eq!(eth, WEI_IN_ETHER);

        let val: U256 = parse_units("2.3", "ether").unwrap().into();
        assert_eq!(val, U256::from_dec_str("2300000000000000000").unwrap());

        let n: U256 = parse_units(".2", 2).unwrap().into();
        assert_eq!(n, U256::from(20), "leading dot");

        let n: U256 = parse_units("333.21", 2).unwrap().into();
        assert_eq!(n, U256::from(33321), "trailing dot");

        let n: U256 = parse_units("98766", 16).unwrap().into();
        assert_eq!(n, U256::from_dec_str("987660000000000000000").unwrap(), "no dot");

        let n: U256 = parse_units("3_3_0", 3).unwrap().into();
        assert_eq!(n, U256::from(330000), "underscore");

        let n: U256 = parse_units("330", 0).unwrap().into();
        assert_eq!(n, U256::from(330), "zero decimals");

        let n: U256 = parse_units(".1234", 3).unwrap().into();
        assert_eq!(n, U256::from(123), "truncate too many decimals");

        assert!(parse_units("1", 80).is_err(), "overflow");
        assert!(parse_units("1", -1).is_err(), "neg units");

        let two_e30 = U256::from(2) * U256([0x4674edea40000000, 0xc9f2c9cd0, 0x0, 0x0]);
        let n: U256 = parse_units("2", 30).unwrap().into();
        assert_eq!(n, two_e30, "2e30");

        let n: U256 = parse_units(".33_319_2", 0).unwrap().into();
        assert_eq!(n, U256::zero(), "mix");

        let n: U256 = parse_units("", 3).unwrap().into();
        assert_eq!(n, U256::zero(), "empty");
    }

    #[test]
    fn test_signed_parse_units() {
        let gwei: I256 = parse_units(-1.5, 9).unwrap().into();
        assert_eq!(gwei.as_i64(), -15e8 as i64);

        let token: I256 = parse_units(-1163.56926418, 8).unwrap().into();
        assert_eq!(token.as_i64(), -116356926418);

        let eth_dec_float: I256 = parse_units(-1.39563324, "ether").unwrap().into();
        assert_eq!(eth_dec_float, I256::from_dec_str("-1395633240000000000").unwrap());

        let eth_dec_string: I256 = parse_units("-1.39563324", "ether").unwrap().into();
        assert_eq!(eth_dec_string, I256::from_dec_str("-1395633240000000000").unwrap());

        let eth: I256 = parse_units(-1, "ether").unwrap().into();
        assert_eq!(eth, I256::from_raw(WEI_IN_ETHER) * I256::minus_one());

        let val: I256 = parse_units("-2.3", "ether").unwrap().into();
        assert_eq!(val, I256::from_dec_str("-2300000000000000000").unwrap());

        let n: I256 = parse_units("-.2", 2).unwrap().into();
        assert_eq!(n, I256::from(-20), "leading dot");

        let n: I256 = parse_units("-333.21", 2).unwrap().into();
        assert_eq!(n, I256::from(-33321), "trailing dot");

        let n: I256 = parse_units("-98766", 16).unwrap().into();
        assert_eq!(n, I256::from_dec_str("-987660000000000000000").unwrap(), "no dot");

        let n: I256 = parse_units("-3_3_0", 3).unwrap().into();
        assert_eq!(n, I256::from(-330000), "underscore");

        let n: I256 = parse_units("-330", 0).unwrap().into();
        assert_eq!(n, I256::from(-330), "zero decimals");

        let n: I256 = parse_units("-.1234", 3).unwrap().into();
        assert_eq!(n, I256::from(-123), "truncate too many decimals");

        assert!(parse_units("-1", 80).is_err(), "overflow");

        let two_e30 =
            I256::from(-2) * I256::from_raw(U256([0x4674edea40000000, 0xc9f2c9cd0, 0x0, 0x0]));
        let n: I256 = parse_units("-2", 30).unwrap().into();
        assert_eq!(n, two_e30, "-2e30");

        let n: I256 = parse_units("-.33_319_2", 0).unwrap().into();
        assert_eq!(n, I256::zero(), "mix");

        let n: I256 = parse_units("-", 3).unwrap().into();
        assert_eq!(n, I256::zero(), "empty");
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
    fn checksummed_parse() {
        let cases = vec![
            // mainnet
            // wrong case
            (None, "0x27b1fdb04752bbc536007a920d24acb045561c26", true),
            (None, "0x27B1fdb04752bbc536007a920d24acb045561c26", false),
            // no checksummed
            (None, "0x52908400098527886e0f7030069857d2e4169ee7", false),
            // without 0x
            (None, "0x42712D45473476b98452f434e72461577D686318", true),
            (None, "42712D45473476b98452f434e72461577D686318", true),
            // invalid address string
            (None, "0x52908400098527886E0F7030069857D2E4169EE7", true),
            (None, "0x52908400098527886E0F7030069857D2E4169EEX", false),
            (None, "0x52908400098527886E0F7030069857D2E4169EE70", false),
            // mistyped address
            (None, "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed", true),
            (None, "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAe1", false),
            // rsk mainnet
            // wrong case
            (Some(30), "0x27b1FdB04752BBc536007A920D24ACB045561c26", true),
            (Some(30), "0x27b1FdB04752BBc536007A920D24ACB045561C26", false),
            // without 0x
            (Some(30), "0x3599689E6292B81B2D85451025146515070129Bb", true),
            (Some(30), "3599689E6292B81B2D85451025146515070129Bb", true),
            // invalid address string
            (Some(30), "0x42712D45473476B98452f434E72461577d686318", true),
            (Some(30), "0x42712D45473476B98452f434E72461577d686318Z", false),
            // mistyped address
            (Some(30), "0x52908400098527886E0F7030069857D2E4169ee7", true),
            (Some(30), "0x52908400098527886E0F7030069857D2E4169ee9", false),
        ]; // mainnet

        for (chain_id, addr, expected) in cases {
            let result = parse_checksummed(addr, chain_id);
            assert_eq!(
                result.is_ok(),
                expected,
                "chain_id: {:?} addr: {:?} error: {:?}",
                chain_id,
                addr,
                result.err()
            );
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

    #[test]
    fn int_or_hex_combinations() {
        // make sure we can deserialize all combinations of int and hex
        // including large numbers that would overflow u64
        //
        // format: (string, expected value)
        let cases = vec![
            // hex strings
            ("\"0x0\"", U256::from(0)),
            ("\"0x1\"", U256::from(1)),
            ("\"0x10\"", U256::from(16)),
            ("\"0x100000000000000000000000000000000000000000000000000\"", U256::from_dec_str("1606938044258990275541962092341162602522202993782792835301376").unwrap()),
            // small num, both num and str form
            ("10", U256::from(10)),
            ("\"10\"", U256::from(10)),
            // max u256, in both num and str form
            ("115792089237316195423570985008687907853269984665640564039457584007913129639935", U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap()),
            ("\"115792089237316195423570985008687907853269984665640564039457584007913129639935\"", U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap())
        ];

        #[derive(Deserialize)]
        struct TestUint(#[serde(deserialize_with = "deserialize_stringified_numeric")] U256);

        for (string, expected) in cases {
            println!("testing {}", string);
            let test: TestUint = serde_json::from_str(string).unwrap();
            assert_eq!(test.0, expected);
        }
    }
}
