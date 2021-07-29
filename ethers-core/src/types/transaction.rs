//! Transaction types
use crate::{
    types::{Address, Bloom, Bytes, Log, NameOrAddress, Signature, H256, U256, U64},
    utils::keccak256,
};

use rlp::RlpStream;
use serde::{Deserialize, Serialize};

const BASE_NUM_TX_FIELDS: usize = 9;

// Number of tx fields before signing
#[cfg(not(feature = "celo"))]
const NUM_TX_FIELDS: usize = BASE_NUM_TX_FIELDS;
// Celo has 3 additional fields
#[cfg(feature = "celo")]
const NUM_TX_FIELDS: usize = BASE_NUM_TX_FIELDS + 3;

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

    fn rlp_base(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, self.nonce);
        rlp_opt(rlp, self.gas_price);
        rlp_opt(rlp, self.gas);

        #[cfg(feature = "celo")]
        self.inject_celo_metadata(rlp);

        rlp_opt(rlp, self.to.as_ref());
        rlp_opt(rlp, self.value);
        rlp_opt(rlp, self.data.as_ref().map(|d| d.as_ref()));
    }
}

// Separate impl block for the celo-specific fields
#[cfg(feature = "celo")]
impl TransactionRequest {
    // modifies the RLP stream with the Celo-specific information
    fn inject_celo_metadata(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, self.fee_currency);
        rlp_opt(rlp, self.gateway_fee_recipient);
        rlp_opt(rlp, self.gateway_fee);
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

fn rlp_opt<T: rlp::Encodable>(rlp: &mut RlpStream, opt: Option<T>) {
    if let Some(ref inner) = opt {
        rlp.append(inner);
    } else {
        rlp.append(&"");
    }
}

/// Details of a signed transaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Transaction {
    /// The transaction's hash
    pub hash: H256,

    /// The transaction's nonce
    pub nonce: U256,

    /// Block hash. None when pending.
    #[serde(rename = "blockHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<H256>,

    /// Block number. None when pending.
    #[serde(rename = "blockNumber")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<U64>,

    /// Transaction Index. None when pending.
    #[serde(rename = "transactionIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_index: Option<U64>,

    /// Sender
    pub from: Address,

    /// Recipient (None when contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,

    /// Transfered value
    pub value: U256,

    /// Gas Price
    #[serde(rename = "gasPrice")]
    pub gas_price: U256,

    /// Gas amount
    pub gas: U256,

    /// Input data
    pub input: Bytes,

    /// ECDSA recovery id
    pub v: U64,

    /// ECDSA signature r
    pub r: U256,

    /// ECDSA signature s
    pub s: U256,

    /////////////////  Celo-specific transaction fields /////////////////
    /// The currency fees are paid in (None for native currency)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none", rename = "feeCurrency")]
    pub fee_currency: Option<Address>,

    /// Gateway fee recipient (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "gatewayFeeRecipient"
    )]
    pub gateway_fee_recipient: Option<Address>,

    /// Gateway fee amount (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none", rename = "gatewayFee")]
    pub gateway_fee: Option<U256>,
}

impl Transaction {
    // modifies the RLP stream with the Celo-specific information
    // This is duplicated from TransactionRequest. Is there a good way to get rid
    // of this code duplication?
    #[cfg(feature = "celo")]
    fn inject_celo_metadata(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, self.fee_currency);
        rlp_opt(rlp, self.gateway_fee_recipient);
        rlp_opt(rlp, self.gateway_fee);
    }

    pub fn hash(&self) -> H256 {
        keccak256(&self.rlp().as_ref()).into()
    }

    pub fn rlp(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_TX_FIELDS);
        rlp.append(&self.nonce);
        rlp.append(&self.gas_price);
        rlp.append(&self.gas);

        #[cfg(feature = "celo")]
        self.inject_celo_metadata(&mut rlp);

        rlp_opt(&mut rlp, self.to);
        rlp.append(&self.value);
        rlp.append(&self.input.as_ref());
        rlp.append(&self.v);
        rlp.append(&self.r);
        rlp.append(&self.s);

        rlp.out().freeze().into()
    }
}

/// "Receipt" of an executed transaction: details of its execution.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction hash.
    #[serde(rename = "transactionHash")]
    pub transaction_hash: H256,
    /// Index within the block.
    #[serde(rename = "transactionIndex")]
    pub transaction_index: U64,
    /// Hash of the block this transaction was included within.
    #[serde(rename = "blockHash")]
    pub block_hash: Option<H256>,
    /// Number of the block this transaction was included within.
    #[serde(rename = "blockNumber")]
    pub block_number: Option<U64>,
    /// Cumulative gas used within the block after this was executed.
    #[serde(rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: U256,
    /// Gas used by this transaction alone.
    ///
    /// Gas used is `None` if the the client is running in light client mode.
    #[serde(rename = "gasUsed")]
    pub gas_used: Option<U256>,
    /// Contract address created, or `None` if not a deployment.
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<Address>,
    /// Logs generated within this transaction.
    pub logs: Vec<Log>,
    /// Status: either 1 (success) or 0 (failure). Only present after activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub status: Option<U64>,
    /// State root. Only present before activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub root: Option<H256>,
    /// Logs bloom
    #[serde(rename = "logsBloom")]
    pub logs_bloom: Bloom,
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

    #[test]
    fn decode_transaction_response() {
        let _res: Transaction = serde_json::from_str(
            r#"{
    "blockHash":"0x1d59ff54b1eb26b013ce3cb5fc9dab3705b415a67127a003c3e61eb445bb8df2",
    "blockNumber":"0x5daf3b",
    "from":"0xa7d9ddbe1f17865597fbd27ec712455208b6b76d",
    "gas":"0xc350",
    "gasPrice":"0x4a817c800",
    "hash":"0x88df016429689c079f3b2f6ad39fa052532c56795b733da78a91ebe6a713944b",
    "input":"0x68656c6c6f21",
    "nonce":"0x15",
    "to":"0xf02c1c8e6114b1dbe8937a39260b5b0a374432bb",
    "transactionIndex":"0x41",
    "value":"0xf3dbb76162000",
    "v":"0x25",
    "r":"0x1b5e176d927f8e9ab405058b2d2457392da3e20f328b16ddabcebc33eaac5fea",
    "s":"0x4ba69724e8f69de52f0125ad8b3c5c2cef33019bac3249e2c0a2192766d1721c"
  }"#,
        )
        .unwrap();

        let _res: Transaction = serde_json::from_str(
            r#"{
            "hash":"0xdd79ab0f996150aa3c9f135bbb9272cf0dedb830fafcbbf0c06020503565c44f",
            "nonce":"0xe",
            "blockHash":"0xef3fe1f532c3d8783a6257619bc123e9453aa8d6614e4cdb4cc8b9e1ed861404",
            "blockNumber":"0xf",
            "transactionIndex":"0x0",
            "from":"0x1b67b03cdccfae10a2d80e52d3d026dbe2960ad0",
            "to":"0x986ee0c8b91a58e490ee59718cca41056cf55f24",
            "value":"0x2710",
            "gas":"0x5208",
            "gasPrice":"0x186a0",
            "input":"0x",
            "v":"0x25",
            "r":"0x75188beb2f601bb8cf52ef89f92a6ba2bb7edcf8e3ccde90548cc99cbea30b1e",
            "s":"0xc0559a540f16d031f3404d5df2bb258084eee56ed1193d8b534bb6affdb3c2c"
    }"#,
        )
        .unwrap();
    }
}
