pub mod request;
pub mod response;

pub mod eip1559;
pub mod eip2718;
pub mod eip2930;

#[cfg(feature = "optimism")]
pub mod optimism_deposited;

pub mod eip712;

pub(crate) const BASE_NUM_TX_FIELDS: usize = 9;

// Number of tx fields before signing
#[cfg(not(feature = "celo"))]
pub(crate) const NUM_TX_FIELDS: usize = BASE_NUM_TX_FIELDS;
// Celo has 3 additional fields
#[cfg(feature = "celo")]
pub(crate) const NUM_TX_FIELDS: usize = BASE_NUM_TX_FIELDS + 3;

pub(super) fn rlp_opt<T: rlp::Encodable>(rlp: &mut rlp::RlpStream, opt: &Option<T>) {
    if let Some(inner) = opt {
        rlp.append(inner);
    } else {
        rlp.append(&"");
    }
}

pub(super) fn rlp_opt_list<T: rlp::Encodable>(rlp: &mut rlp::RlpStream, opt: &Option<T>) {
    if let Some(inner) = opt {
        rlp.append(inner);
    } else {
        // Choice of `u8` type here is arbitrary as all empty lists are encoded the same.
        rlp.append_list::<u8, u8>(&[]);
    }
}

/// normalizes the signature back to 0/1
pub(crate) fn normalize_v(v: u64, chain_id: crate::types::U64) -> u64 {
    if v > 1 {
        v - chain_id.as_u64() * 2 - 35
    } else {
        v
    }
}

/// extracts the chainid from the signature v value based on EIP-155
pub(crate) fn extract_chain_id(v: u64) -> Option<crate::types::U64> {
    // https://eips.ethereum.org/EIPS/eip-155
    // if chainid is available, v = {0, 1} + CHAIN_ID * 2 + 35
    if v >= 35 {
        return Some(crate::types::U64::from((v - 35) >> 1))
    }
    None
}

/// Decodes the signature portion of the RLP encoding based on the RLP offset passed.
/// Increments the offset for each element parsed.
#[inline]
fn decode_signature(
    rlp: &rlp::Rlp,
    offset: &mut usize,
) -> Result<super::Signature, rlp::DecoderError> {
    let sig = super::Signature {
        v: rlp.val_at(*offset)?,
        r: rlp.val_at(*offset + 1)?,
        s: rlp.val_at(*offset + 2)?,
    };
    *offset += 3;
    Ok(sig)
}

/// Decodes the `to` field of the RLP encoding based on the RLP offset passed. Increments the offset
/// by one.
#[inline]
fn decode_to(
    rlp: &rlp::Rlp,
    offset: &mut usize,
) -> Result<Option<super::Address>, rlp::DecoderError> {
    let to = {
        let to = rlp.at(*offset)?;
        if to.is_empty() {
            if to.is_data() {
                None
            } else {
                return Err(rlp::DecoderError::RlpExpectedToBeData)
            }
        } else {
            Some(to.as_val()?)
        }
    };
    *offset += 1;

    Ok(to)
}

#[cfg(test)]
mod tests {
    use crate::types::{transaction::rlp_opt, U64};
    use rlp::RlpStream;

    #[test]
    fn test_rlp_opt_none() {
        let mut stream = RlpStream::new_list(1);
        let empty_chainid: Option<U64> = None;
        rlp_opt(&mut stream, &empty_chainid);
        let out = stream.out();
        assert_eq!(out, vec![0xc1, 0x80]);
    }
}
