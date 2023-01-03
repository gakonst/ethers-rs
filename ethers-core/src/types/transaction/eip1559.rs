use super::{decode_to, eip2718::TypedTransaction, eip2930::AccessList, normalize_v, rlp_opt};
use crate::types::{
    Address, Bytes, NameOrAddress, Signature, SignatureError, Transaction, U256, U64,
};
use rlp::{Decodable, DecoderError, RlpStream};
use thiserror::Error;

/// EIP-1559 transactions have 9 fields
const NUM_TX_FIELDS: usize = 9;

use serde::{Deserialize, Serialize};

/// An error involving an EIP1559 transaction request.
#[derive(Debug, Error)]
pub enum Eip1559RequestError {
    /// When decoding a transaction request from RLP
    #[error(transparent)]
    DecodingError(#[from] rlp::DecoderError),
    /// When recovering the address from a signature
    #[error(transparent)]
    RecoveryError(#[from] SignatureError),
}

/// Parameters for sending a transaction
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Eip1559TransactionRequest {
    /// Sender address or ENS name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,

    /// Recipient address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<NameOrAddress>,

    /// Supplied gas (None for sensible default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<U256>,

    /// Transferred value (None for no transfer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,

    /// The compiled code of a contract OR the first 4 bytes of the hash of the
    /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Bytes>,

    /// Transaction nonce (None for next available nonce)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<U256>,

    #[serde(rename = "accessList", default)]
    pub access_list: AccessList,

    #[serde(rename = "maxPriorityFeePerGas", default, skip_serializing_if = "Option::is_none")]
    /// Represents the maximum tx fee that will go to the miner as part of the user's
    /// fee payment. It serves 3 purposes:
    /// 1. Compensates miners for the uncle/ommer risk + fixed costs of including transaction in a
    /// block;
    /// 2. Allows users with high opportunity costs to pay a premium to miners;
    /// 3. In times where demand exceeds the available block space (i.e. 100% full, 30mm gas),
    /// this component allows first price auctions (i.e. the pre-1559 fee model) to happen on the
    /// priority fee.
    ///
    /// More context [here](https://hackmd.io/@q8X_WM2nTfu6nuvAzqXiTQ/1559-wallets)
    pub max_priority_fee_per_gas: Option<U256>,

    #[serde(rename = "maxFeePerGas", default, skip_serializing_if = "Option::is_none")]
    /// Represents the maximum amount that a user is willing to pay for their tx (inclusive of
    /// baseFeePerGas and maxPriorityFeePerGas). The difference between maxFeePerGas and
    /// baseFeePerGas + maxPriorityFeePerGas is “refunded” to the user.
    pub max_fee_per_gas: Option<U256>,

    #[serde(skip_serializing)]
    #[serde(default, rename = "chainId")]
    /// Chain ID (None for mainnet)
    pub chain_id: Option<U64>,
}

impl Eip1559TransactionRequest {
    /// Creates an empty transaction request with all fields left empty
    pub fn new() -> Self {
        Self::default()
    }

    // Builder pattern helpers

    /// Sets the `from` field in the transaction to the provided value
    #[must_use]
    pub fn from<T: Into<Address>>(mut self, from: T) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Sets the `to` field in the transaction to the provided value
    #[must_use]
    pub fn to<T: Into<NameOrAddress>>(mut self, to: T) -> Self {
        self.to = Some(to.into());
        self
    }

    /// Sets the `gas` field in the transaction to the provided value
    #[must_use]
    pub fn gas<T: Into<U256>>(mut self, gas: T) -> Self {
        self.gas = Some(gas.into());
        self
    }

    /// Sets the `max_priority_fee_per_gas` field in the transaction to the provided value
    #[must_use]
    pub fn max_priority_fee_per_gas<T: Into<U256>>(mut self, max_priority_fee_per_gas: T) -> Self {
        self.max_priority_fee_per_gas = Some(max_priority_fee_per_gas.into());
        self
    }

    /// Sets the `max_fee_per_gas` field in the transaction to the provided value
    #[must_use]
    pub fn max_fee_per_gas<T: Into<U256>>(mut self, max_fee_per_gas: T) -> Self {
        self.max_fee_per_gas = Some(max_fee_per_gas.into());
        self
    }

    /// Sets the `value` field in the transaction to the provided value
    #[must_use]
    pub fn value<T: Into<U256>>(mut self, value: T) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets the `data` field in the transaction to the provided value
    #[must_use]
    pub fn data<T: Into<Bytes>>(mut self, data: T) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Sets the `access_list` field in the transaction to the provided value
    #[must_use]
    pub fn access_list<T: Into<AccessList>>(mut self, access_list: T) -> Self {
        self.access_list = access_list.into();
        self
    }

    /// Sets the `nonce` field in the transaction to the provided value
    #[must_use]
    pub fn nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
        self.nonce = Some(nonce.into());
        self
    }

    /// Sets the `chain_id` field in the transaction to the provided value
    #[must_use]
    pub fn chain_id<T: Into<U64>>(mut self, chain_id: T) -> Self {
        self.chain_id = Some(chain_id.into());
        self
    }

    /// Gets the unsigned transaction's RLP encoding
    pub fn rlp(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_TX_FIELDS);
        self.rlp_base(&mut rlp);
        rlp.out().freeze().into()
    }

    /// Produces the RLP encoding of the transaction with the provided signature
    pub fn rlp_signed(&self, signature: &Signature) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_unbounded_list();
        self.rlp_base(&mut rlp);

        // if the chain_id is none we assume mainnet and choose one
        let chain_id = self.chain_id.unwrap_or_else(U64::one);

        // append the signature
        let v = normalize_v(signature.v, chain_id);
        rlp.append(&v);
        rlp.append(&signature.r);
        rlp.append(&signature.s);
        rlp.finalize_unbounded_list();
        rlp.out().freeze().into()
    }

    pub(crate) fn rlp_base(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, &self.chain_id);
        rlp_opt(rlp, &self.nonce);
        rlp_opt(rlp, &self.max_priority_fee_per_gas);
        rlp_opt(rlp, &self.max_fee_per_gas);
        rlp_opt(rlp, &self.gas);
        rlp_opt(rlp, &self.to.as_ref());
        rlp_opt(rlp, &self.value);
        rlp_opt(rlp, &self.data.as_ref().map(|d| d.as_ref()));
        rlp.append(&self.access_list);
    }

    /// Decodes fields of the request starting at the RLP offset passed. Increments the offset for
    /// each element parsed.
    #[inline]
    pub fn decode_base_rlp(rlp: &rlp::Rlp, offset: &mut usize) -> Result<Self, DecoderError> {
        let mut tx = Self::new();
        tx.chain_id = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.nonce = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.max_priority_fee_per_gas = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.max_fee_per_gas = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.gas = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.to = decode_to(rlp, offset)?.map(NameOrAddress::Address);
        tx.value = Some(rlp.val_at(*offset)?);
        *offset += 1;
        let data = rlp::Rlp::new(rlp.at(*offset)?.as_raw()).data()?;
        tx.data = match data.len() {
            0 => None,
            _ => Some(Bytes::from(data.to_vec())),
        };
        *offset += 1;
        tx.access_list = rlp.val_at(*offset)?;
        *offset += 1;
        Ok(tx)
    }

    /// Decodes the given RLP into a transaction, attempting to decode its signature as well.
    pub fn decode_signed_rlp(rlp: &rlp::Rlp) -> Result<(Self, Signature), Eip1559RequestError> {
        let mut offset = 0;
        let mut txn = Self::decode_base_rlp(rlp, &mut offset)?;

        let v = rlp.val_at(offset)?;
        offset += 1;
        let r = rlp.val_at(offset)?;
        offset += 1;
        let s = rlp.val_at(offset)?;

        let sig = Signature { r, s, v };
        txn.from = Some(sig.recover(TypedTransaction::Eip1559(txn.clone()).sighash())?);

        Ok((txn, sig))
    }
}

impl Decodable for Eip1559TransactionRequest {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        Self::decode_base_rlp(rlp, &mut 0)
    }
}

impl From<Eip1559TransactionRequest> for super::request::TransactionRequest {
    fn from(tx: Eip1559TransactionRequest) -> Self {
        Self {
            from: tx.from,
            to: tx.to,
            gas: tx.gas,
            gas_price: tx.max_fee_per_gas,
            value: tx.value,
            data: tx.data,
            nonce: tx.nonce,
            #[cfg(feature = "celo")]
            fee_currency: None,
            #[cfg(feature = "celo")]
            gateway_fee_recipient: None,
            #[cfg(feature = "celo")]
            gateway_fee: None,
            chain_id: tx.chain_id,
        }
    }
}

impl From<&Transaction> for Eip1559TransactionRequest {
    fn from(tx: &Transaction) -> Eip1559TransactionRequest {
        Eip1559TransactionRequest {
            from: Some(tx.from),
            to: tx.to.map(NameOrAddress::Address),
            gas: Some(tx.gas),
            value: Some(tx.value),
            data: Some(Bytes(tx.input.0.clone())),
            nonce: Some(tx.nonce),
            access_list: tx.access_list.clone().unwrap_or_default(),
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
            max_fee_per_gas: tx.max_fee_per_gas,
            chain_id: tx.chain_id.map(|x| U64::from(x.as_u64())),
        }
    }
}
