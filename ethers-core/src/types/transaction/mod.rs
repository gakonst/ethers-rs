pub mod request;
pub mod response;

pub mod eip2930;

pub(crate) const BASE_NUM_TX_FIELDS: usize = 9;

// Number of tx fields before signing
#[cfg(not(feature = "celo"))]
pub(crate) const NUM_TX_FIELDS: usize = BASE_NUM_TX_FIELDS;
// Celo has 3 additional fields
#[cfg(feature = "celo")]
pub(crate) const NUM_TX_FIELDS: usize = BASE_NUM_TX_FIELDS + 3;

pub(super) fn rlp_opt<T: rlp::Encodable>(rlp: &mut rlp::RlpStream, opt: Option<T>) {
    if let Some(ref inner) = opt {
        rlp.append(inner);
    } else {
        rlp.append(&"");
    }
}
