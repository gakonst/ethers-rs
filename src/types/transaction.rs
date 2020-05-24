//! Transaction types
//!
//! We define 3 transaction types, `TransactionRequest`, `UnsignedTransaction` and `Transaction`.
//! `TransactionRequest` and `UnsignedTransaction` are both unsigned transaction objects.
//!
//! The former gets submitted to the node via an `eth_sendTransaction` call, which populates any missing fields,
//! signs it and broadcasts it. The latter is signed locally by a private key, and the signed
//! transaction is broadcast via `eth_sendRawTransaction`.
use crate::{
    types::{Address, Bytes, Signature, H256, U256, U64},
    utils::keccak256,
};

use serde::{Deserialize, Serialize};

/// Parameters for sending a transaction to via the eth_sendTransaction API.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TransactionRequest {
    /// Sender address or ENS name
    pub from: Address,

    /// Recipient address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,

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
}

/// A raw unsigned transaction where all the information is already specified
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct UnsignedTransaction {
    /// Recipient address (None for contract creation)
    pub to: Option<Address>,

    /// Supplied gas
    pub gas: U256,

    /// Gas price
    #[serde(rename = "gasPrice")]
    pub gas_price: U256,

    /// Transfered value
    pub value: U256,

    /// The compiled code of a contract OR the first 4 bytes of the hash of the
    /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
    pub input: Bytes,

    /// Transaction nonce (None for next available nonce)
    pub nonce: U256,
}

use rlp::RlpStream;

impl UnsignedTransaction {
    fn rlp_base(&self, rlp: &mut RlpStream) {
        rlp.append(&self.nonce);
        rlp.append(&self.gas_price);
        rlp.append(&self.gas);
        rlp_opt(rlp, &self.to);
        rlp.append(&self.value);
        rlp.append(&self.input.0);
    }

    pub fn hash(&self, chain_id: Option<U64>) -> H256 {
        let mut rlp = RlpStream::new();
        rlp.begin_list(9);
        self.rlp_base(&mut rlp);

        rlp.append(&chain_id.unwrap_or_else(U64::zero));
        rlp.append(&0u8);
        rlp.append(&0u8);

        keccak256(rlp.out().as_ref()).into()
    }

    pub fn rlp_signed(&self, signature: &Signature) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(9);
        self.rlp_base(&mut rlp);

        rlp.append(&signature.v);
        rlp.append(&signature.r);
        rlp.append(&signature.s);

        rlp.out().into()
    }
}

/// Details of a signed transaction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
}

impl Transaction {
    pub fn hash(&self) -> H256 {
        keccak256(&self.rlp().0).into()
    }

    pub fn rlp(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(9);
        rlp.append(&self.nonce);
        rlp.append(&self.gas_price);
        rlp.append(&self.gas);
        rlp_opt(&mut rlp, &self.to);
        rlp.append(&self.value);
        rlp.append(&self.input.0);
        rlp.append(&self.v);
        rlp.append(&self.r);
        rlp.append(&self.s);

        rlp.out().into()
    }
}

fn rlp_opt<T: rlp::Encodable>(rlp: &mut RlpStream, opt: &Option<T>) {
    if let Some(inner) = opt {
        rlp.append(inner);
    } else {
        rlp.append(&"");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_unsigned_transaction() {
        let _res: UnsignedTransaction = serde_json::from_str(
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
