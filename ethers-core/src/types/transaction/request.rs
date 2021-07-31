//! Transaction types
use super::{rlp_opt, NUM_TX_FIELDS};
use crate::{
    types::{Address, Bytes, NameOrAddress, Signature, H256, U256, U64},
    utils::keccak256,
};

use rlp::RlpStream;
use serde::{Deserialize, Serialize};

/// Parameters for sending a transaction
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TransactionRequest {
    /// Sender address or ENS name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,

    /// Recipient address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<NameOrAddress>,

    /// Supplied gas (None for sensible default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<U256>,

    /// Gas price (None for sensible default)
    #[serde(rename = "gasPrice")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>,

    /// Transfered value (None for no transfer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,

    /// The compiled code of a contract OR the first 4 bytes of the hash of the
    /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Bytes>,

    /// Transaction nonce (None for next available nonce)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<U256>,

    /////////////////  Celo-specific transaction fields /////////////////
    /// The currency fees are paid in (None for native currency)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_currency: Option<Address>,

    /// Gateway fee recipient (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_fee_recipient: Option<Address>,

    /// Gateway fee amount (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_fee: Option<U256>,
}

impl TransactionRequest {
    /// Creates an empty transaction request with all fields left empty
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience function for sending a new payment transaction to the receiver.
    pub fn pay<T: Into<NameOrAddress>, V: Into<U256>>(to: T, value: V) -> Self {
        TransactionRequest {
            to: Some(to.into()),
            value: Some(value.into()),
            ..Default::default()
        }
    }

    // Builder pattern helpers

    /// Sets the `from` field in the transaction to the provided value
    pub fn from<T: Into<Address>>(mut self, from: T) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Sets the `to` field in the transaction to the provided value
    pub fn to<T: Into<NameOrAddress>>(mut self, to: T) -> Self {
        self.to = Some(to.into());
        self
    }

    /// Sets the `gas` field in the transaction to the provided value
    pub fn gas<T: Into<U256>>(mut self, gas: T) -> Self {
        self.gas = Some(gas.into());
        self
    }

    /// Sets the `gas_price` field in the transaction to the provided value
    pub fn gas_price<T: Into<U256>>(mut self, gas_price: T) -> Self {
        self.gas_price = Some(gas_price.into());
        self
    }

    /// Sets the `value` field in the transaction to the provided value
    pub fn value<T: Into<U256>>(mut self, value: T) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets the `data` field in the transaction to the provided value
    pub fn data<T: Into<Bytes>>(mut self, data: T) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Sets the `nonce` field in the transaction to the provided value
    pub fn nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
        self.nonce = Some(nonce.into());
        self
    }

    /// Hashes the transaction's data with the provided chain id
    pub fn sighash<T: Into<U64>>(&self, chain_id: T) -> H256 {
        keccak256(self.rlp(chain_id).as_ref()).into()
    }

    /// Gets the unsigned transaction's RLP encoding
    pub fn rlp<T: Into<U64>>(&self, chain_id: T) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_TX_FIELDS);
        self.rlp_base(&mut rlp);

        // Only hash the 3 extra fields when preparing the
        // data to sign if chain_id is present
        rlp.append(&chain_id.into());
        rlp.append(&0u8);
        rlp.append(&0u8);
        rlp.out().freeze().into()
    }

    /// Produces the RLP encoding of the transaction with the provided signature
    pub fn rlp_signed(&self, signature: &Signature) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_TX_FIELDS);
        self.rlp_base(&mut rlp);

        // append the signature
        rlp.append(&signature.v);
        rlp.append(&signature.r);
        rlp.append(&signature.s);
        rlp.out().freeze().into()
    }

    pub(crate) fn rlp_base(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, &self.nonce);
        rlp_opt(rlp, &self.gas_price);
        rlp_opt(rlp, &self.gas);

        #[cfg(feature = "celo")]
        self.inject_celo_metadata(rlp);

        rlp_opt(rlp, &self.to.as_ref());
        rlp_opt(rlp, &self.value);
        rlp_opt(rlp, &self.data.as_ref().map(|d| d.as_ref()));
    }
}

// Separate impl block for the celo-specific fields
#[cfg(feature = "celo")]
impl TransactionRequest {
    // modifies the RLP stream with the Celo-specific information
    fn inject_celo_metadata(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, &self.fee_currency);
        rlp_opt(rlp, &self.gateway_fee_recipient);
        rlp_opt(rlp, &self.gateway_fee);
    }

    /// Sets the `fee_currency` field in the transaction to the provided value
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    pub fn fee_currency<T: Into<Address>>(mut self, fee_currency: T) -> Self {
        self.fee_currency = Some(fee_currency.into());
        self
    }

    /// Sets the `gateway_fee` field in the transaction to the provided value
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    pub fn gateway_fee<T: Into<U256>>(mut self, gateway_fee: T) -> Self {
        self.gateway_fee = Some(gateway_fee.into());
        self
    }

    /// Sets the `gateway_fee_recipient` field in the transaction to the provided value
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    pub fn gateway_fee_recipient<T: Into<Address>>(mut self, gateway_fee_recipient: T) -> Self {
        self.gateway_fee_recipient = Some(gateway_fee_recipient.into());
        self
    }
}

#[cfg(test)]
#[cfg(not(feature = "celo"))]
mod tests {
    use super::*;

    #[test]
    fn decode_unsigned_transaction() {
        let _res: TransactionRequest = serde_json::from_str(
            r#"{
    "gas":"0xc350",
    "gasPrice":"0x4a817c800",
    "hash":"0x88df016429689c079f3b2f6ad39fa052532c56795b733da78a91ebe6a713944b",
    "input":"0x68656c6c6f21",
    "nonce":"0x15",
    "to":"0xf02c1c8e6114b1dbe8937a39260b5b0a374432bb",
    "transactionIndex":"0x41",
    "value":"0xf3dbb76162000",
    "chain_id": "0x1"
  }"#,
        )
        .unwrap();
    }
}
