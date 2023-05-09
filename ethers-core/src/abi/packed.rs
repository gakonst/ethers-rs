use ethabi::Token;
use thiserror::Error;
use Token::*;

/// An error thrown by [`encode_packed`].
#[derive(Debug, Error)]
pub enum EncodePackedError {
    #[error("This token cannot be encoded in packed mode: {0:?}")]
    InvalidToken(Token),

    #[error("FixedBytes token length > 32")]
    InvalidBytesLength,
}

/// Encodes the given tokens into an ABI compliant vector of bytes.
///
/// This function uses [non-standard packed mode][ref], where:
/// - types shorter than 32 bytes are concatenated directly, without padding or sign extension;
/// - dynamic types are encoded in-place and without the length;
/// - array elements are padded, but still encoded in-place.
///
/// Since this encoding is ambiguous, there is no decoding function.
///
/// Note that this function has the same behaviour as its [Solidity counterpart][ref], and
/// thus structs as well as nested arrays are not supported.
///
/// `Uint` and `Int` tokens will be encoded using the least number of bits, so no padding will be
/// added by default.
///
/// [ref]: https://docs.soliditylang.org/en/latest/abi-spec.html#non-standard-packed-mode
///
/// # Examples
///
/// Calculate the UniswapV2 pair address for two ERC20 tokens:
///
/// ```
/// # use ethers_core::abi::{self, Token};
/// # use ethers_core::types::{Address, H256};
/// # use ethers_core::utils;
/// let factory: Address = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".parse()?;
///
/// let token_a: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;
/// let token_b: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()?;
/// let encoded = abi::encode_packed(&[Token::Address(token_a), Token::Address(token_b)])?;
/// let salt = utils::keccak256(encoded);
///
/// let init_code_hash: H256 = "0x96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f".parse()?;
///
/// let pair = utils::get_create2_address_from_hash(factory, salt, init_code_hash);
/// let weth_usdc = "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc".parse()?;
/// assert_eq!(pair, weth_usdc);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn encode_packed(tokens: &[Token]) -> Result<Vec<u8>, EncodePackedError> {
    // Get vec capacity and find invalid tokens
    let mut max = 0;
    for token in tokens {
        check(token)?;
        max += max_encoded_length(token);
    }

    // Encode the tokens
    let mut bytes = Vec::with_capacity(max);
    for token in tokens {
        encode_token(token, &mut bytes, false);
    }
    Ok(bytes)
}

/// The maximum byte length of the token encoded using packed mode.
fn max_encoded_length(token: &Token) -> usize {
    match token {
        Int(_) | Uint(_) | FixedBytes(_) => 32,
        Address(_) => 20,
        Bool(_) => 1,
        // account for padding
        Array(vec) | FixedArray(vec) | Tuple(vec) => {
            vec.iter().map(|token| max_encoded_length(token).max(32)).sum()
        }
        Bytes(b) => b.len(),
        String(s) => s.len(),
    }
}

/// Tuples and nested arrays are invalid in packed encoding.
fn check(token: &Token) -> Result<(), EncodePackedError> {
    match token {
        FixedBytes(vec) if vec.len() > 32 => Err(EncodePackedError::InvalidBytesLength),

        Tuple(_) => Err(EncodePackedError::InvalidToken(token.clone())),
        Array(vec) | FixedArray(vec) => {
            for t in vec.iter() {
                if t.is_dynamic() || matches!(t, Array(_)) {
                    return Err(EncodePackedError::InvalidToken(token.clone()))
                }
                check(t)?;
            }
            Ok(())
        }

        _ => Ok(()),
    }
}

/// Encodes `token` as bytes into `out`.
fn encode_token(token: &Token, out: &mut Vec<u8>, in_array: bool) {
    match token {
        // Padded to 32 bytes if in_array
        Address(addr) => {
            if in_array {
                out.extend_from_slice(&[0; 12]);
            }
            out.extend_from_slice(&addr.0)
        }
        Int(n) | Uint(n) => {
            let mut buf = [0; 32];
            n.to_big_endian(&mut buf);
            let start = if in_array { 0 } else { 32 - ((n.bits() + 7) / 8) };
            out.extend_from_slice(&buf[start..32]);
        }
        Bool(b) => {
            if in_array {
                out.extend_from_slice(&[0; 31]);
            }
            out.push((*b) as u8);
        }
        FixedBytes(bytes) => {
            out.extend_from_slice(bytes);
            if in_array {
                let mut remaining = vec![0; 32 - bytes.len()];
                out.append(&mut remaining);
            }
        }

        // Encode dynamic types in-place, without their length
        Bytes(bytes) => out.extend_from_slice(bytes),
        String(s) => out.extend_from_slice(s.as_bytes()),
        Array(vec) | FixedArray(vec) => {
            for token in vec {
                encode_token(token, out, true);
            }
        }

        // Should never happen
        token => unreachable!("Uncaught invalid token: {token:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    fn encode(tokens: &[Token]) -> Vec<u8> {
        encode_packed(tokens).unwrap()
    }

    fn string(s: &str) -> Token {
        Token::String(s.into())
    }

    fn bytes(b: &[u8]) -> Token {
        Token::Bytes(b.into())
    }

    #[test]
    fn encode_bytes0() {
        let expected = b"hello world";
        assert_eq!(encode_packed(&[string("hello world")]).unwrap(), expected);
        assert_eq!(encode_packed(&[bytes(b"hello world")]).unwrap(), expected);
        assert_eq!(
            encode_packed(&[string("hello"), string(" "), string("world")]).unwrap(),
            expected
        );
        assert_eq!(
            encode_packed(&[bytes(b"hello"), bytes(b" "), bytes(b"world")]).unwrap(),
            expected
        );
    }

    // modified from: ethabi::encoder::tests
    // https://github.com/rust-ethereum/ethabi/blob/4e14ff83bf27de56555cc2ae0ece9ed2fe28fa0f/ethabi/src/encoder.rs#L181

    #[test]
    fn encode_address() {
        let address = Token::Address([0x11u8; 20].into());
        let encoded = encode(&[address]);
        let expected = hex!("1111111111111111111111111111111111111111");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_dynamic_array_of_addresses() {
        let address1 = Token::Address([0x11u8; 20].into());
        let address2 = Token::Address([0x22u8; 20].into());
        let addresses = Token::Array(vec![address1, address2]);
        let encoded = encode(&[addresses]);
        let expected = hex!(
            "
			0000000000000000000000001111111111111111111111111111111111111111
			0000000000000000000000002222222222222222222222222222222222222222
		"
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_fixed_array_of_addresses() {
        let address1 = Token::Address([0x11u8; 20].into());
        let address2 = Token::Address([0x22u8; 20].into());
        let addresses = Token::FixedArray(vec![address1, address2]);
        let encoded = encode(&[addresses]);
        let expected = hex!(
            "
			0000000000000000000000001111111111111111111111111111111111111111
			0000000000000000000000002222222222222222222222222222222222222222
		"
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_two_addresses() {
        let address1 = Token::Address([0x11u8; 20].into());
        let address2 = Token::Address([0x22u8; 20].into());
        let encoded = encode(&[address1, address2]);
        let expected = hex!(
            "
			1111111111111111111111111111111111111111
			2222222222222222222222222222222222222222
		"
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_empty_array() {
        let encoded = encode(&[Token::Array(vec![]), Token::Array(vec![])]);
        assert_eq!(encoded, b"");
    }

    #[test]
    fn encode_bytes() {
        let bytes = Token::Bytes(hex!("1234").to_vec());
        let encoded = encode(&[bytes]);
        let expected = hex!("1234");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_bytes2() {
        let bytes = Token::Bytes(
            hex!("10000000000000000000000000000000000000000000000000000000000002").to_vec(),
        );
        let encoded = encode(&[bytes]);
        let expected =
            hex!("10000000000000000000000000000000000000000000000000000000000002").to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_bytes3() {
        let bytes = Token::Bytes(
            hex!(
                "
                    1000000000000000000000000000000000000000000000000000000000000000
                    1000000000000000000000000000000000000000000000000000000000000000
		        "
            )
            .to_vec(),
        );
        let encoded = encode(&[bytes]);
        let expected = hex!(
            "
            1000000000000000000000000000000000000000000000000000000000000000
            1000000000000000000000000000000000000000000000000000000000000000
		    "
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_string() {
        let s = Token::String("gavofyork".to_owned());
        let encoded = encode(&[s]);
        assert_eq!(encoded, b"gavofyork");
    }

    #[test]
    fn encode_two_bytes() {
        let bytes1 = Token::Bytes(
            hex!("10000000000000000000000000000000000000000000000000000000000002").to_vec(),
        );
        let bytes2 = Token::Bytes(
            hex!("0010000000000000000000000000000000000000000000000000000000000002").to_vec(),
        );
        let encoded = encode(&[bytes1, bytes2]);
        let expected = hex!(
            "
			10000000000000000000000000000000000000000000000000000000000002
			0010000000000000000000000000000000000000000000000000000000000002
		"
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_fixed_bytes() {
        let bytes = Token::FixedBytes(vec![0x12, 0x34]);
        let encoded = encode(&[bytes]);
        let expected = hex!("1234");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_array_of_fixed_bytes() {
        let array = Token::FixedArray(vec![
            Token::FixedBytes(vec![0x12, 0x34]),
            Token::FixedBytes(vec![0x56, 0x78]),
        ]);
        let encoded = encode(&[array]);
        let expected = hex!(
            "
            1234000000000000000000000000000000000000000000000000000000000000
            5678000000000000000000000000000000000000000000000000000000000000
        "
        );
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_uint() {
        let mut uint = [0u8; 32];
        uint[31] = 4;
        let encoded = encode(&[Token::Uint(uint.into())]);
        let expected = hex!("04");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_int() {
        let mut int = [0u8; 32];
        int[31] = 4;
        let encoded = encode(&[Token::Int(int.into())]);
        let expected = hex!("04");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_bool() {
        let encoded = encode(&[Token::Bool(true)]);
        let expected = hex!("01");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_bool2() {
        let encoded = encode(&[Token::Bool(false)]);
        let expected = hex!("00");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn comprehensive_test() {
        let bytes = hex!(
            "
			131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b
			131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b
		"
        )
        .to_vec();
        let encoded = encode(&[
            Token::Int(5.into()),
            Token::Bytes(bytes.clone()),
            Token::Int(3.into()),
            Token::Bytes(bytes),
        ]);

        let expected = hex!(
            "
			05
            131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b
			131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b
			03
            131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b
			131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b
		"
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn comprehensive_test2() {
        let encoded = encode(&vec![
            Token::Int(1.into()),
            Token::String("gavofyork".to_owned()),
            Token::Int(2.into()),
            Token::Int(3.into()),
            Token::Int(4.into()),
            Token::Array(vec![Token::Int(5.into()), Token::Int(6.into()), Token::Int(7.into())]),
        ]);

        let expected = hex!(
            "
			01
			6761766f66796f726b
			02
			03
			04
			0000000000000000000000000000000000000000000000000000000000000005
			0000000000000000000000000000000000000000000000000000000000000006
			0000000000000000000000000000000000000000000000000000000000000007
		"
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }
}
