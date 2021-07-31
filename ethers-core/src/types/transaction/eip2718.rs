use super::{eip1559::Eip1559TransactionRequest, eip2930::Eip2930TransactionRequest};
use crate::{
    types::{TransactionRequest, H256, U64},
    utils::keccak256,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum TypedTransaction {
    // 0x00
    #[serde(rename = "0x00")]
    Legacy(TransactionRequest),
    // 0x01
    #[serde(rename = "0x01")]
    Eip2930(Eip2930TransactionRequest),
    // 0x02
    #[serde(rename = "0x02")]
    Eip1559(Eip1559TransactionRequest),
}

impl TypedTransaction {
    /// Hashes the transaction's data with the provided chain id
    pub fn sighash<T: Into<U64>>(&self, chain_id: T) -> H256 {
        let encoded = match self {
            TypedTransaction::Legacy(ref tx) => {
                let mut encoded = vec![0];
                encoded.extend_from_slice(tx.rlp(chain_id).as_ref());
                encoded
            }
            TypedTransaction::Eip2930(ref tx) => {
                let mut encoded = vec![1];
                encoded.extend_from_slice(tx.rlp(chain_id).as_ref());
                encoded
            }
            TypedTransaction::Eip1559(ref tx) => {
                let mut encoded = vec![2];
                encoded.extend_from_slice(tx.rlp(chain_id).as_ref());
                encoded
            }
        };
        keccak256(encoded).into()
    }
}

impl From<TransactionRequest> for TypedTransaction {
    fn from(src: TransactionRequest) -> TypedTransaction {
        TypedTransaction::Legacy(src)
    }
}

impl From<Eip2930TransactionRequest> for TypedTransaction {
    fn from(src: Eip2930TransactionRequest) -> TypedTransaction {
        TypedTransaction::Eip2930(src)
    }
}

impl From<Eip1559TransactionRequest> for TypedTransaction {
    fn from(src: Eip1559TransactionRequest) -> TypedTransaction {
        TypedTransaction::Eip1559(src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, U256};

    #[test]
    fn serde_legacy_tx() {
        let tx = TransactionRequest::new()
            .to(Address::zero())
            .value(U256::from(100));
        let tx = TypedTransaction::from(tx);
        let serialized = serde_json::to_string(&tx).unwrap();

        // deserializes to either the envelope type or the inner type
        let de: TypedTransaction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tx, de);

        let de: TransactionRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tx, TypedTransaction::Legacy(de));
    }
}
