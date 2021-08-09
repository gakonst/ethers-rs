use super::{eip2930::AccessList, normalize_v, rlp_opt};
use crate::{
    types::{Address, Bytes, NameOrAddress, Signature, H256, U256, U64},
    utils::keccak256,
};
use rlp::RlpStream;

/// EIP-1559 transactions have 9 fields
const NUM_TX_FIELDS: usize = 9;

use serde::{Deserialize, Serialize};
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

    #[serde(rename = "accessList", default)]
    pub access_list: AccessList,

    #[serde(
        rename = "maxPriorityFeePerGas",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    /// Represents the maximum tx fee that will go to the miner as part of the user's
    /// fee payment. It serves 3 purposes:
    /// 1. Compensates miners for the uncle/ommer risk + fixed costs of including transaction in a block;
    /// 2. Allows users with high opportunity costs to pay a premium to miners;
    /// 3. In times where demand exceeds the available block space (i.e. 100% full, 30mm gas),
    /// this component allows first price auctions (i.e. the pre-1559 fee model) to happen on the priority fee.
    ///
    /// More context [here](https://hackmd.io/@q8X_WM2nTfu6nuvAzqXiTQ/1559-wallets)
    pub max_priority_fee_per_gas: Option<U256>,

    #[serde(
        rename = "maxFeePerGas",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    /// Represents the maximum amount that a user is willing to pay for their tx (inclusive of baseFeePerGas and maxPriorityFeePerGas).
    /// The difference between maxFeePerGas and baseFeePerGas + maxPriorityFeePerGas is “refunded” to the user.
    pub max_fee_per_gas: Option<U256>,
}

impl Eip1559TransactionRequest {
    /// Creates an empty transaction request with all fields left empty
    pub fn new() -> Self {
        Self::default()
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

    /// Sets the `max_priority_fee_per_gas` field in the transaction to the provided value
    pub fn max_priority_fee_per_gas<T: Into<U256>>(mut self, max_priority_fee_per_gas: T) -> Self {
        self.max_priority_fee_per_gas = Some(max_priority_fee_per_gas.into());
        self
    }

    /// Sets the `max_fee_per_gas` field in the transaction to the provided value
    pub fn max_fee_per_gas<T: Into<U256>>(mut self, max_fee_per_gas: T) -> Self {
        self.max_fee_per_gas = Some(max_fee_per_gas.into());
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

    /// Sets the `access_list` field in the transaction to the provided value
    pub fn access_list<T: Into<AccessList>>(mut self, access_list: T) -> Self {
        self.access_list = access_list.into();
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
        self.rlp_base(chain_id, &mut rlp);
        rlp.out().freeze().into()
    }

    /// Produces the RLP encoding of the transaction with the provided signature
    pub fn rlp_signed<T: Into<U64>>(&self, chain_id: T, signature: &Signature) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_unbounded_list();
        let chain_id = chain_id.into();
        self.rlp_base(chain_id, &mut rlp);

        // append the signature
        let v = normalize_v(signature.v, chain_id);
        rlp.append(&v);
        rlp.append(&signature.r);
        rlp.append(&signature.s);
        rlp.finalize_unbounded_list();
        rlp.out().freeze().into()
    }

    pub(crate) fn rlp_base<T: Into<U64>>(&self, chain_id: T, rlp: &mut RlpStream) {
        rlp.append(&chain_id.into());
        rlp_opt(rlp, &self.nonce);
        rlp_opt(rlp, &self.max_priority_fee_per_gas);
        rlp_opt(rlp, &self.max_fee_per_gas);
        rlp_opt(rlp, &self.gas);
        rlp_opt(rlp, &self.to.as_ref());
        rlp_opt(rlp, &self.value);
        rlp_opt(rlp, &self.data.as_ref().map(|d| d.as_ref()));
        rlp.append(&self.access_list);
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
        }
    }
}
