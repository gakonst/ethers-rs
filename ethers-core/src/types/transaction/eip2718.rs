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

#[cfg(feature = "legacy")]
impl Default for TypedTransaction {
    fn default() -> Self {
        TypedTransaction::Legacy(Default::default())
    }
}

#[cfg(not(feature = "legacy"))]
impl Default for TypedTransaction {
    fn default() -> Self {
        TypedTransaction::Eip1559(Default::default())
    }
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

    pub fn chain_id(&self) -> Option<U64> {
        match self {
            Legacy(inner) => inner.chain_id,
            Eip2930(inner) => inner.tx.chain_id,
            Eip1559(inner) => inner.chain_id,
        }
    }

    pub fn set_chain_id<T: Into<U64>>(&mut self, chain_id: T) {
        let chain_id = chain_id.into();
        match self {
            Legacy(inner) => inner.chain_id = Some(chain_id),
            Eip2930(inner) => inner.tx.chain_id = Some(chain_id),
            Eip1559(inner) => inner.chain_id = Some(chain_id),
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

    pub fn rlp_signed(&self, signature: &Signature) -> Bytes {
        let mut encoded = vec![];
        match self {
            Legacy(ref tx) => {
                encoded.extend_from_slice(tx.rlp_signed(signature).as_ref());
            }
            Eip2930(inner) => {
                encoded.extend_from_slice(&[0x1]);
                encoded.extend_from_slice(inner.rlp_signed(signature).as_ref());
            }
            Eip1559(inner) => {
                encoded.extend_from_slice(&[0x2]);
                encoded.extend_from_slice(inner.rlp_signed(signature).as_ref());
            }
        };
        encoded.into()
    }

    pub fn rlp(&self) -> Bytes {
        let mut encoded = vec![];
        match self {
            Legacy(inner) => {
                encoded.extend_from_slice(inner.rlp().as_ref());
            }
            Eip2930(inner) => {
                encoded.extend_from_slice(&[0x1]);
                encoded.extend_from_slice(inner.rlp().as_ref());
            }
            Eip1559(inner) => {
                encoded.extend_from_slice(&[0x2]);
                encoded.extend_from_slice(inner.rlp().as_ref());
            }
        };

        encoded.into()
    }

    /// Hashes the transaction's data. Does not double-RLP encode
    pub fn sighash(&self) -> H256 {
        let encoded = self.rlp();
        keccak256(encoded).into()
    }
}

/// Get a TypedTransaction directly from an rlp encoded byte stream
impl rlp::Decodable for TypedTransaction {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let tx_type: Option<U64> = match rlp.is_data() {
            true => Some(rlp.data().unwrap().into()),
            false => None,
        };
        let rest = rlp::Rlp::new(
            rlp.as_raw().get(1..).ok_or(rlp::DecoderError::Custom("no transaction payload"))?,
        );

        match tx_type {
            Some(x) if x == U64::from(1) => {
                // EIP-2930 (0x01)
                Ok(Self::Eip2930(Eip2930TransactionRequest::decode(&rest)?))
            }
            Some(x) if x == U64::from(2) => {
                // EIP-1559 (0x02)
                Ok(Self::Eip1559(Eip1559TransactionRequest::decode(&rest)?))
            }
            _ => {
                // Legacy (0x00)
                // use the original rlp
                Ok(Self::Legacy(TransactionRequest::decode(rlp)?))
            }
        }
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
    use rlp::Decodable;

    use super::*;
    use crate::types::{Address, U256};
    use std::str::FromStr;

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

    #[test]
    fn test_typed_tx_without_access_list() {
        let tx: Eip1559TransactionRequest = serde_json::from_str(
            r#"{
            "gas": "0x186a0",
            "maxFeePerGas": "0x77359400",
            "maxPriorityFeePerGas": "0x77359400",
            "data": "0x5544",
            "nonce": "0x2",
            "to": "0x96216849c49358B10257cb55b28eA603c874b05E",
            "value": "0x5af3107a4000",
            "type": "0x2",
            "chainId": "0x539",
            "accessList": [],
            "v": "0x1",
            "r": "0xc3000cd391f991169ebfd5d3b9e93c89d31a61c998a21b07a11dc6b9d66f8a8e",
            "s": "0x22cfe8424b2fbd78b16c9911da1be2349027b0a3c40adf4b6459222323773f74"
        }"#,
        )
        .unwrap();

        let envelope = TypedTransaction::Eip1559(tx);

        let expected =
            H256::from_str("0xa1ea3121940930f7e7b54506d80717f14c5163807951624c36354202a8bffda6")
                .unwrap();
        let actual = envelope.sighash();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_typed_tx() {
        let tx: Eip1559TransactionRequest = serde_json::from_str(
            r#"{
            "gas": "0x186a0",
            "maxFeePerGas": "0x77359400",
            "maxPriorityFeePerGas": "0x77359400",
            "data": "0x5544",
            "nonce": "0x2",
            "to": "0x96216849c49358B10257cb55b28eA603c874b05E",
            "value": "0x5af3107a4000",
            "type": "0x2",
            "accessList": [
                {
                    "address": "0x0000000000000000000000000000000000000001",
                    "storageKeys": [
                        "0x0100000000000000000000000000000000000000000000000000000000000000"
                    ]
                }
            ],
            "chainId": "0x539",
            "v": "0x1",
            "r": "0xc3000cd391f991169ebfd5d3b9e93c89d31a61c998a21b07a11dc6b9d66f8a8e",
            "s": "0x22cfe8424b2fbd78b16c9911da1be2349027b0a3c40adf4b6459222323773f74"
        }"#,
        )
        .unwrap();

        let envelope = TypedTransaction::Eip1559(tx);

        let expected =
            H256::from_str("0x090b19818d9d087a49c3d2ecee4829ee4acea46089c1381ac5e588188627466d")
                .unwrap();
        let actual = envelope.sighash();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_typed_tx_decode() {
        // this is the same transaction as the above test
        let typed_tx_hex = hex::decode("02f86b8205390284773594008477359400830186a09496216849c49358b10257cb55b28ea603c874b05e865af3107a4000825544f838f7940000000000000000000000000000000000000001e1a00100000000000000000000000000000000000000000000000000000000000000").unwrap();
        let tx_rlp = rlp::Rlp::new(typed_tx_hex.as_slice());
        let actual_tx = TypedTransaction::decode(&tx_rlp).unwrap();

        let expected =
            H256::from_str("0x090b19818d9d087a49c3d2ecee4829ee4acea46089c1381ac5e588188627466d")
                .unwrap();
        let actual = actual_tx.sighash();
        assert_eq!(expected, actual);
    }

    #[cfg(not(feature = "celo"))]
    #[test]
    fn test_eip155_decode() {
        let tx = TransactionRequest::new()
            .nonce(9)
            .to("3535353535353535353535353535353535353535".parse::<Address>().unwrap())
            .value(1000000000000000000u64)
            .gas_price(20000000000u64)
            .gas(21000)
            .chain_id(1);

        let expected_hex = hex::decode("ec098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a764000080018080").unwrap();
        let expected_rlp = rlp::Rlp::new(expected_hex.as_slice());
        let decoded_transaction = TypedTransaction::decode(&expected_rlp).unwrap();
        assert_eq!(tx.sighash(), decoded_transaction.sighash());
    }
}
