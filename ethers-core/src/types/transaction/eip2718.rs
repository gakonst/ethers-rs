use super::{
    eip1559::Eip1559TransactionRequest,
    eip2930::{AccessList, Eip2930TransactionRequest},
};
use crate::{
    types::{Address, Bytes, NameOrAddress, Signature, TransactionRequest, H256, U256, U64},
    utils::keccak256,
};
use serde::{Deserialize, Serialize};

/// The TypedTransaction enum represents all Ethereum transaction types.
///
/// Its variants correspond to specific allowed transactions:
/// 1. Legacy (pre-EIP2718) [`TransactionRequest`]
/// 2. EIP2930 (state access lists) [`Eip2930TransactionRequest`]
/// 3. EIP1559 [`Eip1559TransactionRequest`]
///
/// To support Kovan and other non-London-compatbile networks, please enable
/// the `legacy` crate feature. This will disable the `type` flag in the
/// serialized transaction, and cause contract calls and other common actions
/// to default to using the legacy transaction type.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[cfg_attr(not(feature = "legacy"), serde(tag = "type"))]
#[cfg_attr(feature = "legacy", serde(untagged))]
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

use TypedTransaction::*;

impl TypedTransaction {
    pub fn from(&self) -> Option<&Address> {
        match self {
            Legacy(inner) => inner.from.as_ref(),
            Eip2930(inner) => inner.tx.from.as_ref(),
            Eip1559(inner) => inner.from.as_ref(),
        }
    }

    pub fn set_from(&mut self, from: Address) {
        match self {
            Legacy(inner) => inner.from = Some(from),
            Eip2930(inner) => inner.tx.from = Some(from),
            Eip1559(inner) => inner.from = Some(from),
        };
    }

    pub fn to(&self) -> Option<&NameOrAddress> {
        match self {
            Legacy(inner) => inner.to.as_ref(),
            Eip2930(inner) => inner.tx.to.as_ref(),
            Eip1559(inner) => inner.to.as_ref(),
        }
    }

    pub fn set_to<T: Into<NameOrAddress>>(&mut self, to: T) {
        let to = to.into();
        match self {
            Legacy(inner) => inner.to = Some(to),
            Eip2930(inner) => inner.tx.to = Some(to),
            Eip1559(inner) => inner.to = Some(to),
        };
    }

    pub fn nonce(&self) -> Option<&U256> {
        match self {
            Legacy(inner) => inner.nonce.as_ref(),
            Eip2930(inner) => inner.tx.nonce.as_ref(),
            Eip1559(inner) => inner.nonce.as_ref(),
        }
    }

    pub fn set_nonce<T: Into<U256>>(&mut self, nonce: T) {
        let nonce = nonce.into();
        match self {
            Legacy(inner) => inner.nonce = Some(nonce),
            Eip2930(inner) => inner.tx.nonce = Some(nonce),
            Eip1559(inner) => inner.nonce = Some(nonce),
        };
    }

    pub fn value(&self) -> Option<&U256> {
        match self {
            Legacy(inner) => inner.value.as_ref(),
            Eip2930(inner) => inner.tx.value.as_ref(),
            Eip1559(inner) => inner.value.as_ref(),
        }
    }

    pub fn set_value<T: Into<U256>>(&mut self, value: T) {
        let value = value.into();
        match self {
            Legacy(inner) => inner.value = Some(value),
            Eip2930(inner) => inner.tx.value = Some(value),
            Eip1559(inner) => inner.value = Some(value),
        };
    }

    pub fn gas(&self) -> Option<&U256> {
        match self {
            Legacy(inner) => inner.gas.as_ref(),
            Eip2930(inner) => inner.tx.gas.as_ref(),
            Eip1559(inner) => inner.gas.as_ref(),
        }
    }

    pub fn set_gas<T: Into<U256>>(&mut self, gas: T) {
        let gas = gas.into();
        match self {
            Legacy(inner) => inner.gas = Some(gas),
            Eip2930(inner) => inner.tx.gas = Some(gas),
            Eip1559(inner) => inner.gas = Some(gas),
        };
    }

    pub fn gas_price(&self) -> Option<U256> {
        match self {
            Legacy(inner) => inner.gas_price,
            Eip2930(inner) => inner.tx.gas_price,
            Eip1559(inner) => {
                match (inner.max_fee_per_gas, inner.max_priority_fee_per_gas) {
                    (Some(basefee), Some(prio_fee)) => Some(basefee + prio_fee),
                    // this also covers the None, None case
                    (None, prio_fee) => prio_fee,
                    (basefee, None) => basefee,
                }
            }
        }
    }

    pub fn set_gas_price<T: Into<U256>>(&mut self, gas_price: T) {
        let gas_price = gas_price.into();
        match self {
            Legacy(inner) => inner.gas_price = Some(gas_price),
            Eip2930(inner) => inner.tx.gas_price = Some(gas_price),
            Eip1559(inner) => {
                inner.max_fee_per_gas = Some(gas_price);
                inner.max_priority_fee_per_gas = Some(gas_price);
            }
        };
    }

    pub fn data(&self) -> Option<&Bytes> {
        match self {
            Legacy(inner) => inner.data.as_ref(),
            Eip2930(inner) => inner.tx.data.as_ref(),
            Eip1559(inner) => inner.data.as_ref(),
        }
    }

    pub fn access_list(&self) -> Option<&AccessList> {
        match self {
            Legacy(_) => None,
            Eip2930(inner) => Some(&inner.access_list),
            Eip1559(inner) => Some(&inner.access_list),
        }
    }

    pub fn set_access_list(&mut self, access_list: AccessList) {
        match self {
            Legacy(_) => {}
            Eip2930(inner) => inner.access_list = access_list,
            Eip1559(inner) => inner.access_list = access_list,
        };
    }

    pub fn set_data(&mut self, data: Bytes) {
        match self {
            Legacy(inner) => inner.data = Some(data),
            Eip2930(inner) => inner.tx.data = Some(data),
            Eip1559(inner) => inner.data = Some(data),
        };
    }

    pub fn rlp_signed<T: Into<U64>>(&self, chain_id: T, signature: &Signature) -> Bytes {
        let mut encoded = vec![];
        match self {
            Legacy(ref tx) => {
                encoded.extend_from_slice(tx.rlp_signed(signature).as_ref());
            }
            Eip2930(inner) => {
                encoded.extend_from_slice(&[0x1]);
                encoded.extend_from_slice(inner.rlp_signed(chain_id, signature).as_ref());
            }
            Eip1559(inner) => {
                encoded.extend_from_slice(&[0x2]);
                encoded.extend_from_slice(inner.rlp_signed(chain_id, signature).as_ref());
            }
        };
        encoded.into()
    }

    pub fn rlp<T: Into<U64>>(&self, chain_id: T) -> Bytes {
        let chain_id = chain_id.into();
        let mut encoded = vec![];
        match self {
            Legacy(inner) => {
                encoded.extend_from_slice(inner.rlp(chain_id).as_ref());
            }
            Eip2930(inner) => {
                encoded.extend_from_slice(&[0x1]);
                encoded.extend_from_slice(inner.rlp(chain_id).as_ref());
            }
            Eip1559(inner) => {
                encoded.extend_from_slice(&[0x2]);
                encoded.extend_from_slice(inner.rlp(chain_id).as_ref());
            }
        };

        encoded.into()
    }

    /// Hashes the transaction's data with the provided chain id
    /// Does not double-RLP encode
    pub fn sighash<T: Into<U64>>(&self, chain_id: T) -> H256 {
        let encoded = self.rlp(chain_id);
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
        let tx = TransactionRequest::new().to(Address::zero()).value(U256::from(100));
        let tx: TypedTransaction = tx.into();
        let serialized = serde_json::to_string(&tx).unwrap();

        // deserializes to either the envelope type or the inner type
        let de: TypedTransaction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tx, de);

        let de: TransactionRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tx, TypedTransaction::Legacy(de));
    }
}
