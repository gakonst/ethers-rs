use super::{
    eip1559::{Eip1559RequestError, Eip1559TransactionRequest},
    eip2930::{AccessList, Eip2930RequestError, Eip2930TransactionRequest},
    request::RequestError,
};
use crate::{
    types::{
        Address, Bytes, NameOrAddress, Signature, Transaction, TransactionRequest, H256, U256, U64,
    },
    utils::keccak256,
};
use rlp::Decodable;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "optimism")]
use super::optimism_deposited::{
    OptimismDepositedRequestError, OptimismDepositedTransactionRequest,
};

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
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
    // 0x7E
    #[cfg(feature = "optimism")]
    #[serde(rename = "0x7E")]
    OptimismDeposited(OptimismDepositedTransactionRequest),
}

/// An error involving a typed transaction request.
#[derive(Debug, Error)]
pub enum TypedTransactionError {
    /// When decoding a signed legacy transaction
    #[error(transparent)]
    LegacyError(#[from] RequestError),
    /// When decoding a signed Eip1559 transaction
    #[error(transparent)]
    Eip1559Error(#[from] Eip1559RequestError),
    /// When decoding a signed Eip2930 transaction
    #[error(transparent)]
    Eip2930Error(#[from] Eip2930RequestError),
    /// When decoding a signed Optimism Deposited transaction
    #[cfg(feature = "optimism")]
    #[error(transparent)]
    OptimismDepositedError(#[from] OptimismDepositedRequestError),
    /// Error decoding the transaction type from the transaction's RLP encoding
    #[error(transparent)]
    TypeDecodingError(#[from] rlp::DecoderError),
    /// Missing transaction type when decoding from RLP
    #[error("Missing transaction type when decoding")]
    MissingTransactionType,
    /// Missing transaction payload when decoding from RLP
    #[error("Missing transaction payload when decoding")]
    MissingTransactionPayload,
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
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.from.as_ref(),
        }
    }

    pub fn set_from(&mut self, from: Address) -> &mut Self {
        match self {
            Legacy(inner) => inner.from = Some(from),
            Eip2930(inner) => inner.tx.from = Some(from),
            Eip1559(inner) => inner.from = Some(from),
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.from = Some(from),
        };
        self
    }

    pub fn to(&self) -> Option<&NameOrAddress> {
        match self {
            Legacy(inner) => inner.to.as_ref(),
            Eip2930(inner) => inner.tx.to.as_ref(),
            Eip1559(inner) => inner.to.as_ref(),
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.to.as_ref(),
        }
    }

    pub fn to_addr(&self) -> Option<&Address> {
        self.to().and_then(|t| t.as_address())
    }

    pub fn set_to<T: Into<NameOrAddress>>(&mut self, to: T) -> &mut Self {
        let to = to.into();
        match self {
            Legacy(inner) => inner.to = Some(to),
            Eip2930(inner) => inner.tx.to = Some(to),
            Eip1559(inner) => inner.to = Some(to),
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.to = Some(to),
        };
        self
    }

    pub fn nonce(&self) -> Option<&U256> {
        match self {
            Legacy(inner) => inner.nonce.as_ref(),
            Eip2930(inner) => inner.tx.nonce.as_ref(),
            Eip1559(inner) => inner.nonce.as_ref(),
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.nonce.as_ref(),
        }
    }

    pub fn set_nonce<T: Into<U256>>(&mut self, nonce: T) -> &mut Self {
        let nonce = nonce.into();
        match self {
            Legacy(inner) => inner.nonce = Some(nonce),
            Eip2930(inner) => inner.tx.nonce = Some(nonce),
            Eip1559(inner) => inner.nonce = Some(nonce),
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.nonce = Some(nonce),
        };
        self
    }

    pub fn value(&self) -> Option<&U256> {
        match self {
            Legacy(inner) => inner.value.as_ref(),
            Eip2930(inner) => inner.tx.value.as_ref(),
            Eip1559(inner) => inner.value.as_ref(),
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.value.as_ref(),
        }
    }

    pub fn set_value<T: Into<U256>>(&mut self, value: T) -> &mut Self {
        let value = value.into();
        match self {
            Legacy(inner) => inner.value = Some(value),
            Eip2930(inner) => inner.tx.value = Some(value),
            Eip1559(inner) => inner.value = Some(value),
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.value = Some(value),
        };
        self
    }

    pub fn gas(&self) -> Option<&U256> {
        match self {
            Legacy(inner) => inner.gas.as_ref(),
            Eip2930(inner) => inner.tx.gas.as_ref(),
            Eip1559(inner) => inner.gas.as_ref(),
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.gas.as_ref(),
        }
    }

    pub fn gas_mut(&mut self) -> &mut Option<U256> {
        match self {
            Legacy(inner) => &mut inner.gas,
            Eip2930(inner) => &mut inner.tx.gas,
            Eip1559(inner) => &mut inner.gas,
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => &mut inner.tx.gas,
        }
    }

    pub fn set_gas<T: Into<U256>>(&mut self, gas: T) -> &mut Self {
        let gas = gas.into();
        match self {
            Legacy(inner) => inner.gas = Some(gas),
            Eip2930(inner) => inner.tx.gas = Some(gas),
            Eip1559(inner) => inner.gas = Some(gas),
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.gas = Some(gas),
        };
        self
    }

    pub fn gas_price(&self) -> Option<U256> {
        match self {
            Legacy(inner) => inner.gas_price,
            Eip2930(inner) => inner.tx.gas_price,
            Eip1559(inner) => {
                match (inner.max_fee_per_gas, inner.max_priority_fee_per_gas) {
                    (Some(max_fee), Some(_)) => Some(max_fee),
                    // this also covers the None, None case
                    (None, prio_fee) => prio_fee,
                    (max_fee, None) => max_fee,
                }
            }
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.gas_price,
        }
    }

    pub fn set_gas_price<T: Into<U256>>(&mut self, gas_price: T) -> &mut Self {
        let gas_price = gas_price.into();
        match self {
            Legacy(inner) => inner.gas_price = Some(gas_price),
            Eip2930(inner) => inner.tx.gas_price = Some(gas_price),
            Eip1559(inner) => {
                inner.max_fee_per_gas = Some(gas_price);
                inner.max_priority_fee_per_gas = Some(gas_price);
            }
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.gas_price = Some(gas_price),
        };
        self
    }

    pub fn chain_id(&self) -> Option<U64> {
        match self {
            Legacy(inner) => inner.chain_id,
            Eip2930(inner) => inner.tx.chain_id,
            Eip1559(inner) => inner.chain_id,
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.chain_id,
        }
    }

    pub fn set_chain_id<T: Into<U64>>(&mut self, chain_id: T) -> &mut Self {
        let chain_id = chain_id.into();
        match self {
            Legacy(inner) => inner.chain_id = Some(chain_id),
            Eip2930(inner) => inner.tx.chain_id = Some(chain_id),
            Eip1559(inner) => inner.chain_id = Some(chain_id),
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.chain_id = Some(chain_id),
        };
        self
    }

    pub fn data(&self) -> Option<&Bytes> {
        match self {
            Legacy(inner) => inner.data.as_ref(),
            Eip2930(inner) => inner.tx.data.as_ref(),
            Eip1559(inner) => inner.data.as_ref(),
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.data.as_ref(),
        }
    }

    pub fn access_list(&self) -> Option<&AccessList> {
        match self {
            Legacy(_) => None,
            Eip2930(inner) => Some(&inner.access_list),
            Eip1559(inner) => Some(&inner.access_list),
            #[cfg(feature = "optimism")]
            OptimismDeposited(_) => None,
        }
    }

    pub fn set_access_list(&mut self, access_list: AccessList) -> &mut Self {
        match self {
            Legacy(_) => {}
            Eip2930(inner) => inner.access_list = access_list,
            Eip1559(inner) => inner.access_list = access_list,
            #[cfg(feature = "optimism")]
            OptimismDeposited(_) => {}
        };
        self
    }

    pub fn set_data(&mut self, data: Bytes) -> &mut Self {
        match self {
            Legacy(inner) => inner.data = Some(data),
            Eip2930(inner) => inner.tx.data = Some(data),
            Eip1559(inner) => inner.data = Some(data),
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => inner.tx.data = Some(data),
        };
        self
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
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => {
                encoded.extend_from_slice(&[0x7E]);
                encoded.extend_from_slice(inner.rlp().as_ref());
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
            #[cfg(feature = "optimism")]
            OptimismDeposited(inner) => {
                encoded.extend_from_slice(&[0x7E]);
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

    /// Max cost of the transaction
    pub fn max_cost(&self) -> Option<U256> {
        let gas_limit = self.gas();
        let gas_price = self.gas_price();
        match (gas_limit, gas_price) {
            (Some(gas_limit), Some(gas_price)) => Some(gas_limit * gas_price),
            _ => None,
        }
    }

    /// Hashes the transaction's data with the included signature.
    pub fn hash(&self, signature: &Signature) -> H256 {
        keccak256(self.rlp_signed(signature).as_ref()).into()
    }

    /// Decodes a signed TypedTransaction from a rlp encoded byte stream
    pub fn decode_signed(rlp: &rlp::Rlp) -> Result<(Self, Signature), TypedTransactionError> {
        let data = rlp.data()?;
        let first = *data.first().ok_or(rlp::DecoderError::Custom("empty slice"))?;
        if rlp.is_list() {
            // Legacy (0x00)
            // use the original rlp
            let decoded_request = TransactionRequest::decode_signed_rlp(rlp)?;
            return Ok((Self::Legacy(decoded_request.0), decoded_request.1))
        }

        let rest = rlp::Rlp::new(
            rlp.as_raw().get(1..).ok_or(TypedTransactionError::MissingTransactionPayload)?,
        );

        if first == 0x01 {
            // EIP-2930 (0x01)
            let decoded_request = Eip2930TransactionRequest::decode_signed_rlp(&rest)?;
            return Ok((Self::Eip2930(decoded_request.0), decoded_request.1))
        }
        if first == 0x02 {
            // EIP-1559 (0x02)
            let decoded_request = Eip1559TransactionRequest::decode_signed_rlp(&rest)?;
            return Ok((Self::Eip1559(decoded_request.0), decoded_request.1))
        }
        #[cfg(feature = "optimism")]
        if first == 0x7E {
            // Optimism Deposited (0x7E)
            let decoded_request = OptimismDepositedTransactionRequest::decode_signed_rlp(&rest)?;
            return Ok((Self::OptimismDeposited(decoded_request.0), decoded_request.1))
        }

        Err(rlp::DecoderError::Custom("invalid tx type").into())
    }
}

/// Get a TypedTransaction directly from a rlp encoded byte stream
impl Decodable for TypedTransaction {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let tx_type: Option<U64> = match rlp.is_data() {
            true => Ok(Some(rlp.data()?.into())),
            false => Ok(None),
        }?;
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
            #[cfg(feature = "optimism")]
            Some(x) if x == U64::from(0x7E) => {
                // Optimism Deposited (0x7E)
                Ok(Self::OptimismDeposited(OptimismDepositedTransactionRequest::decode(&rest)?))
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

#[cfg(feature = "optimism")]
impl From<OptimismDepositedTransactionRequest> for TypedTransaction {
    fn from(src: OptimismDepositedTransactionRequest) -> TypedTransaction {
        TypedTransaction::OptimismDeposited(src)
    }
}

impl From<&Transaction> for TypedTransaction {
    fn from(tx: &Transaction) -> TypedTransaction {
        match tx.transaction_type {
            // EIP-2930 (0x01)
            Some(x) if x == U64::from(1) => {
                let request: Eip2930TransactionRequest = tx.into();
                request.into()
            }
            // EIP-1559 (0x02)
            Some(x) if x == U64::from(2) => {
                let request: Eip1559TransactionRequest = tx.into();
                request.into()
            }
            #[cfg(feature = "optimism")]
            // Optimism Deposited (0x7E)
            Some(x) if x == U64::from(0x7E) => {
                let request: OptimismDepositedTransactionRequest = tx.into();
                request.into()
            }
            // Legacy (0x00)
            _ => {
                let request: TransactionRequest = tx.into();
                request.into()
            }
        }
    }
}

impl TypedTransaction {
    pub fn as_legacy_ref(&self) -> Option<&TransactionRequest> {
        match self {
            Legacy(tx) => Some(tx),
            _ => None,
        }
    }
    pub fn as_eip2930_ref(&self) -> Option<&Eip2930TransactionRequest> {
        match self {
            Eip2930(tx) => Some(tx),
            _ => None,
        }
    }
    pub fn as_eip1559_ref(&self) -> Option<&Eip1559TransactionRequest> {
        match self {
            Eip1559(tx) => Some(tx),
            _ => None,
        }
    }
    #[cfg(feature = "optimism")]
    pub fn as_optimism_deposited_ref(&self) -> Option<&OptimismDepositedTransactionRequest> {
        match self {
            OptimismDeposited(tx) => Some(tx),
            _ => None,
        }
    }

    pub fn as_legacy_mut(&mut self) -> Option<&mut TransactionRequest> {
        match self {
            Legacy(tx) => Some(tx),
            _ => None,
        }
    }
    pub fn as_eip2930_mut(&mut self) -> Option<&mut Eip2930TransactionRequest> {
        match self {
            Eip2930(tx) => Some(tx),
            _ => None,
        }
    }
    pub fn as_eip1559_mut(&mut self) -> Option<&mut Eip1559TransactionRequest> {
        match self {
            Eip1559(tx) => Some(tx),
            _ => None,
        }
    }
    #[cfg(feature = "optimism")]
    pub fn as_optimism_deposited_mut(
        &mut self,
    ) -> Option<&mut OptimismDepositedTransactionRequest> {
        match self {
            OptimismDeposited(tx) => Some(tx),
            _ => None,
        }
    }
}

impl TypedTransaction {
    fn into_eip1559(self) -> Eip1559TransactionRequest {
        match self {
            Eip1559(tx) => tx,
            _ => Eip1559TransactionRequest {
                from: self.from().copied(),
                to: self.to().cloned(),
                nonce: self.nonce().copied(),
                value: self.value().copied(),
                gas: self.gas().copied(),
                chain_id: self.chain_id(),
                data: self.data().cloned(),
                access_list: self.access_list().cloned().unwrap_or_default(),
                ..Default::default()
            },
        }
    }
}

impl From<TypedTransaction> for Eip1559TransactionRequest {
    fn from(src: TypedTransaction) -> Eip1559TransactionRequest {
        src.into_eip1559()
    }
}

impl TypedTransaction {
    fn into_legacy(self) -> TransactionRequest {
        match self {
            Legacy(tx) => tx,
            Eip2930(tx) => tx.tx,
            Eip1559(_) => TransactionRequest {
                from: self.from().copied(),
                to: self.to().cloned(),
                nonce: self.nonce().copied(),
                value: self.value().copied(),
                gas: self.gas().copied(),
                gas_price: self.gas_price(),
                chain_id: self.chain_id(),
                data: self.data().cloned(),
                #[cfg(feature = "celo")]
                #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
                fee_currency: None,
                #[cfg(feature = "celo")]
                #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
                gateway_fee_recipient: None,
                #[cfg(feature = "celo")]
                #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
                gateway_fee: None,
            },
            #[cfg(feature = "optimism")]
            OptimismDeposited(tx) => tx.tx,
        }
    }
}

impl From<TypedTransaction> for TransactionRequest {
    fn from(src: TypedTransaction) -> TransactionRequest {
        src.into_legacy()
    }
}

impl TypedTransaction {
    fn into_eip2930(self) -> Eip2930TransactionRequest {
        let access_list = self.access_list().cloned().unwrap_or_default();

        match self {
            Eip2930(tx) => tx,
            Legacy(tx) => Eip2930TransactionRequest { tx, access_list },
            Eip1559(_) => Eip2930TransactionRequest {
                tx: TransactionRequest {
                    from: self.from().copied(),
                    to: self.to().cloned(),
                    nonce: self.nonce().copied(),
                    value: self.value().copied(),
                    gas: self.gas().copied(),
                    gas_price: self.gas_price(),
                    chain_id: self.chain_id(),
                    data: self.data().cloned(),
                    #[cfg(feature = "celo")]
                    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
                    fee_currency: None,
                    #[cfg(feature = "celo")]
                    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
                    gateway_fee_recipient: None,
                    #[cfg(feature = "celo")]
                    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
                    gateway_fee: None,
                },
                access_list,
            },
            #[cfg(feature = "optimism")]
            OptimismDeposited(tx) => Eip2930TransactionRequest { tx: tx.tx, access_list },
        }
    }
}

impl From<TypedTransaction> for Eip2930TransactionRequest {
    fn from(src: TypedTransaction) -> Eip2930TransactionRequest {
        src.into_eip2930()
    }
}

#[cfg(test)]
mod tests {
    use hex::ToHex;
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

    #[test]
    fn test_signed_tx_decode() {
        let expected_tx = Eip1559TransactionRequest::new()
            .from(Address::from_str("0x1acadd971da208d25122b645b2ef879868a83e21").unwrap())
            .chain_id(1u64)
            .nonce(0u64)
            .max_priority_fee_per_gas(413047990155u64)
            .max_fee_per_gas(768658734568u64)
            .gas(184156u64)
            .to(Address::from_str("0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1").unwrap())
            .value(200000000000000000u64)
            .data(
                Bytes::from_str(
                    "0x6ecd23060000000000000000000000000000000000000000000000000000000000000002",
                )
                .unwrap(),
            );

        let expected_envelope = TypedTransaction::Eip1559(expected_tx);
        let typed_tx_hex = hex::decode("02f899018085602b94278b85b2f7a17de88302cf5c940aa7420c43b8c1a7b165d216948870c8ecfe1ee18802c68af0bb140000a46ecd23060000000000000000000000000000000000000000000000000000000000000002c080a0c5f35bf1cc6ab13053e33b1af7400c267be17218aeadcdb4ae3eefd4795967e8a04f6871044dd6368aea8deecd1c29f55b5531020f5506502e3f79ad457051bc4a").unwrap();

        let tx_rlp = rlp::Rlp::new(typed_tx_hex.as_slice());
        let (actual_tx, signature) = TypedTransaction::decode_signed(&tx_rlp).unwrap();
        assert_eq!(expected_envelope, actual_tx);
        assert_eq!(
            expected_envelope.hash(&signature),
            H256::from_str("0x206e4c71335333f8658e995cc0c4ee54395d239acb08587ab8e5409bfdd94a6f")
                .unwrap()
        );
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

    #[test]
    fn test_eip1559_deploy_tx_decode() {
        let typed_tx_hex =
            hex::decode("02dc8205058193849502f90085010c388d00837a120080808411223344c0").unwrap();
        let tx_rlp = rlp::Rlp::new(typed_tx_hex.as_slice());
        TypedTransaction::decode(&tx_rlp).unwrap();
    }

    #[test]
    fn test_signed_tx_decode_all_fields() {
        let typed_tx_hex = hex::decode("02f90188052b85012a05f20085012a05f2148301b3cd8080b9012d608060405234801561001057600080fd5b5061010d806100206000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c8063cfae3217146037578063f8a8fd6d146066575b600080fd5b604080518082019091526003815262676d2160e81b60208201525b604051605d91906085565b60405180910390f35b6040805180820190915260048152636f6f662160e01b60208201526052565b600060208083528351808285015260005b8181101560b0578581018301518582016040015282016096565b8181111560c1576000604083870101525b50601f01601f191692909201604001939250505056fea2646970667358221220f89093a9819ba5d2a3384305511d0945ea94f36a8aa162ab62921b3841fe3afd64736f6c634300080c0033c080a08085850e935fd6af9ace1b0343b9e21d2dcc7e914c36cce61a4e32756c785980a04c57c184d5096263df981cb8a2f2c7f81640792856909dbf3295a2b7a1dc4a55").unwrap();
        let tx_rlp = rlp::Rlp::new(typed_tx_hex.as_slice());
        let (tx, sig) = TypedTransaction::decode_signed(&tx_rlp).unwrap();

        let tx = match tx {
            TypedTransaction::Eip1559(tx) => tx,
            _ => panic!("The raw bytes should decode to an EIP1559 tranaction"),
        };

        // pre-sighash fields - if a value here is incorrect it will show up before the sighash
        // and from asserts fail
        let data = Bytes::from_str("0x608060405234801561001057600080fd5b5061010d806100206000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c8063cfae3217146037578063f8a8fd6d146066575b600080fd5b604080518082019091526003815262676d2160e81b60208201525b604051605d91906085565b60405180910390f35b6040805180820190915260048152636f6f662160e01b60208201526052565b600060208083528351808285015260005b8181101560b0578581018301518582016040015282016096565b8181111560c1576000604083870101525b50601f01601f191692909201604001939250505056fea2646970667358221220f89093a9819ba5d2a3384305511d0945ea94f36a8aa162ab62921b3841fe3afd64736f6c634300080c0033").unwrap();
        assert_eq!(&data, tx.data.as_ref().unwrap());

        let chain_id = U64::from(5u64);
        assert_eq!(chain_id, tx.chain_id.unwrap());

        let nonce = Some(43u64.into());
        assert_eq!(nonce, tx.nonce);

        let max_fee_per_gas = Some(5000000020u64.into());
        assert_eq!(max_fee_per_gas, tx.max_fee_per_gas);

        let max_priority_fee_per_gas = Some(5000000000u64.into());
        assert_eq!(max_priority_fee_per_gas, tx.max_priority_fee_per_gas);

        let gas = Some(111565u64.into());
        assert_eq!(gas, tx.gas);

        // empty fields
        assert_eq!(None, tx.to);
        assert_eq!(AccessList(vec![]), tx.access_list);

        // compare rlp - sighash should then be the same
        let tx_expected_rlp = "f90145052b85012a05f20085012a05f2148301b3cd8080b9012d608060405234801561001057600080fd5b5061010d806100206000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c8063cfae3217146037578063f8a8fd6d146066575b600080fd5b604080518082019091526003815262676d2160e81b60208201525b604051605d91906085565b60405180910390f35b6040805180820190915260048152636f6f662160e01b60208201526052565b600060208083528351808285015260005b8181101560b0578581018301518582016040015282016096565b8181111560c1576000604083870101525b50601f01601f191692909201604001939250505056fea2646970667358221220f89093a9819ba5d2a3384305511d0945ea94f36a8aa162ab62921b3841fe3afd64736f6c634300080c0033c0";
        let tx_real_rlp_vec = tx.rlp().to_vec();
        let tx_real_rlp: String = tx_real_rlp_vec.encode_hex();
        assert_eq!(tx_expected_rlp, tx_real_rlp);

        let r =
            U256::from_str("0x8085850e935fd6af9ace1b0343b9e21d2dcc7e914c36cce61a4e32756c785980")
                .unwrap();
        let s =
            U256::from_str("0x4c57c184d5096263df981cb8a2f2c7f81640792856909dbf3295a2b7a1dc4a55")
                .unwrap();
        let v = 0;
        assert_eq!(r, sig.r);
        assert_eq!(s, sig.s);
        assert_eq!(v, sig.v);

        // finally check from
        let addr = Address::from_str("0x216b32eCEbAe6aF164921D3943cd7A9634FcB199").unwrap();
        assert_eq!(addr, tx.from.unwrap());
    }

    #[test]
    fn test_tx_casts() {
        // eip1559 tx
        let typed_tx_hex = hex::decode("02f86b8205390284773594008477359400830186a09496216849c49358b10257cb55b28ea603c874b05e865af3107a4000825544f838f7940000000000000000000000000000000000000001e1a00100000000000000000000000000000000000000000000000000000000000000").unwrap();
        let tx_rlp = rlp::Rlp::new(typed_tx_hex.as_slice());
        let tx = TypedTransaction::decode(&tx_rlp).unwrap();

        {
            let typed_tx: TypedTransaction = tx.clone();

            let tx0: TransactionRequest = typed_tx.clone().into();
            assert!(typed_tx.as_legacy_ref().is_none());

            let tx1 = typed_tx.into_legacy();

            assert_eq!(tx0, tx1);
        }
        {
            let typed_tx: TypedTransaction = tx.clone();
            let tx0: Eip1559TransactionRequest = typed_tx.clone().into();
            assert_eq!(tx.as_eip1559_ref().unwrap(), &tx0);

            let tx1 = typed_tx.into_eip1559();

            assert_eq!(tx0, tx1);
        }
        {
            let typed_tx: TypedTransaction = tx;
            let tx0: Eip2930TransactionRequest = typed_tx.clone().into();
            assert!(typed_tx.as_eip2930_ref().is_none());

            let tx1 = typed_tx.into_eip2930();

            assert_eq!(tx0, tx1);
        }
    }
}
