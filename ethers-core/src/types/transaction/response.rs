//! Transaction types
use super::{
    decode_signature, decode_to, eip2718::TypedTransaction, eip2930::AccessList, normalize_v,
    rlp_opt, rlp_opt_list,
};
use crate::{
    types::{
        transaction::extract_chain_id, Address, Bloom, Bytes, Log, Signature, SignatureError, H256,
        U256, U64,
    },
    utils::keccak256,
};
use rlp::{Decodable, DecoderError, RlpStream};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Details of a signed transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Transaction {
    /// The transaction's hash
    pub hash: H256,

    /// The transaction's nonce
    pub nonce: U256,

    /// Block hash. None when pending.
    #[serde(default, rename = "blockHash")]
    pub block_hash: Option<H256>,

    /// Block number. None when pending.
    #[serde(default, rename = "blockNumber")]
    pub block_number: Option<U64>,

    /// Transaction Index. None when pending.
    #[serde(default, rename = "transactionIndex")]
    pub transaction_index: Option<U64>,

    /// Sender
    #[serde(default = "crate::types::Address::zero")]
    pub from: Address,

    /// Recipient (None when contract creation)
    #[serde(default)]
    pub to: Option<Address>,

    /// Transferred value
    pub value: U256,

    /// Gas Price, null for Type 2 transactions
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<U256>,

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

    ///////////////// Optimism-specific transaction fields //////////////
    /// The source-hash that uniquely identifies the origin of the deposit
    #[cfg(feature = "optimism")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sourceHash")]
    pub source_hash: Option<H256>,

    /// The ETH value to mint on L2
    #[cfg(feature = "optimism")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mint: Option<U256>,

    /// True if the transaction does not interact with the L2 block gas pool
    #[cfg(feature = "optimism")]
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "isSystemTx")]
    pub is_system_tx: Option<bool>,

    /////////////////  Celo-specific transaction fields /////////////////
    /// The currency fees are paid in (None for native currency)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none", rename = "feeCurrency")]
    pub fee_currency: Option<Address>,

    /// Gateway fee recipient (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none", rename = "gatewayFeeRecipient")]
    pub gateway_fee_recipient: Option<Address>,

    /// Gateway fee amount (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none", rename = "gatewayFee")]
    pub gateway_fee: Option<U256>,

    // EIP2718
    /// Transaction type, Some(2) for EIP-1559 transaction,
    /// Some(1) for AccessList transaction, None for Legacy
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<U64>,

    // EIP2930
    #[serde(rename = "accessList", default, skip_serializing_if = "Option::is_none")]
    pub access_list: Option<AccessList>,

    #[serde(rename = "maxPriorityFeePerGas", default, skip_serializing_if = "Option::is_none")]
    /// Represents the maximum tx fee that will go to the miner as part of the user's
    /// fee payment. It serves 3 purposes:
    /// 1. Compensates miners for the uncle/ommer risk + fixed costs of including transaction in a
    /// block; 2. Allows users with high opportunity costs to pay a premium to miners;
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

    #[serde(rename = "chainId", default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<U256>,

    /// Captures unknown fields such as additional fields used by L2s
    #[cfg(not(any(feature = "celo", feature = "optimism")))]
    #[serde(flatten)]
    pub other: crate::types::OtherFields,
}

impl Transaction {
    // modifies the RLP stream with the Celo-specific information
    // This is duplicated from TransactionRequest. Is there a good way to get rid
    // of this code duplication?
    #[cfg(feature = "celo")]
    fn inject_celo_metadata(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, &self.fee_currency);
        rlp_opt(rlp, &self.gateway_fee_recipient);
        rlp_opt(rlp, &self.gateway_fee);
    }

    pub fn hash(&self) -> H256 {
        keccak256(self.rlp().as_ref()).into()
    }

    pub fn rlp(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_unbounded_list();

        match self.transaction_type {
            // EIP-2930 (0x01)
            Some(x) if x == U64::from(0x1) => {
                rlp_opt(&mut rlp, &self.chain_id);
                rlp.append(&self.nonce);
                rlp_opt(&mut rlp, &self.gas_price);
                rlp.append(&self.gas);

                #[cfg(feature = "celo")]
                self.inject_celo_metadata(&mut rlp);

                rlp_opt(&mut rlp, &self.to);
                rlp.append(&self.value);
                rlp.append(&self.input.as_ref());
                rlp_opt_list(&mut rlp, &self.access_list);
                if let Some(chain_id) = self.chain_id {
                    rlp.append(&normalize_v(self.v.as_u64(), U64::from(chain_id.as_u64())));
                }
                rlp.append(&self.r);
                rlp.append(&self.s);
            }
            // EIP-1559 (0x02)
            Some(x) if x == U64::from(0x2) => {
                rlp_opt(&mut rlp, &self.chain_id);
                rlp.append(&self.nonce);
                rlp_opt(&mut rlp, &self.max_priority_fee_per_gas);
                rlp_opt(&mut rlp, &self.max_fee_per_gas);
                rlp.append(&self.gas);
                rlp_opt(&mut rlp, &self.to);
                rlp.append(&self.value);
                rlp.append(&self.input.as_ref());
                rlp_opt_list(&mut rlp, &self.access_list);
                if let Some(chain_id) = self.chain_id {
                    rlp.append(&normalize_v(self.v.as_u64(), U64::from(chain_id.as_u64())));
                }
                rlp.append(&self.r);
                rlp.append(&self.s);
            }
            // Optimism Deposited Transaction
            #[cfg(feature = "optimism")]
            Some(x) if x == U64::from(0x7E) => {
                rlp_opt(&mut rlp, &self.source_hash);
                rlp.append(&self.from);
                rlp_opt(&mut rlp, &self.to);
                rlp_opt(&mut rlp, &self.mint);
                rlp.append(&self.value);
                rlp.append(&self.gas);
                rlp_opt(&mut rlp, &self.is_system_tx);
                rlp.append(&self.input.as_ref());
            }
            // Legacy (0x00)
            _ => {
                rlp.append(&self.nonce);
                rlp_opt(&mut rlp, &self.gas_price);
                rlp.append(&self.gas);

                #[cfg(feature = "celo")]
                self.inject_celo_metadata(&mut rlp);

                rlp_opt(&mut rlp, &self.to);
                rlp.append(&self.value);
                rlp.append(&self.input.as_ref());
                rlp.append(&self.v);
                rlp.append(&self.r);
                rlp.append(&self.s);
            }
        }

        rlp.finalize_unbounded_list();

        let rlp_bytes: Bytes = rlp.out().freeze().into();
        let mut encoded = vec![];
        match self.transaction_type {
            Some(x) if x == U64::from(0x1) => {
                encoded.extend_from_slice(&[0x1]);
                encoded.extend_from_slice(rlp_bytes.as_ref());
                encoded.into()
            }
            Some(x) if x == U64::from(0x2) => {
                encoded.extend_from_slice(&[0x2]);
                encoded.extend_from_slice(rlp_bytes.as_ref());
                encoded.into()
            }
            #[cfg(feature = "optimism")]
            Some(x) if x == U64::from(0x7E) => {
                encoded.extend_from_slice(&[0x7E]);
                encoded.extend_from_slice(rlp_bytes.as_ref());
                encoded.into()
            }
            _ => rlp_bytes,
        }
    }

    /// Decodes the Celo-specific metadata starting at the RLP offset passed.
    /// Increments the offset for each element parsed.
    #[cfg(feature = "celo")]
    #[inline]
    fn decode_celo_metadata(
        &mut self,
        rlp: &rlp::Rlp,
        offset: &mut usize,
    ) -> Result<(), DecoderError> {
        self.fee_currency = Some(rlp.val_at(*offset)?);
        *offset += 1;
        self.gateway_fee_recipient = Some(rlp.val_at(*offset)?);
        *offset += 1;
        self.gateway_fee = Some(rlp.val_at(*offset)?);
        *offset += 1;
        Ok(())
    }

    /// Decodes fields of the type 2 transaction response starting at the RLP offset passed.
    /// Increments the offset for each element parsed.
    #[inline]
    fn decode_base_eip1559(
        &mut self,
        rlp: &rlp::Rlp,
        offset: &mut usize,
    ) -> Result<(), DecoderError> {
        self.chain_id = Some(rlp.val_at(*offset)?);
        *offset += 1;
        self.nonce = rlp.val_at(*offset)?;
        *offset += 1;
        self.max_priority_fee_per_gas = Some(rlp.val_at(*offset)?);
        *offset += 1;
        self.max_fee_per_gas = Some(rlp.val_at(*offset)?);
        *offset += 1;
        self.gas = rlp.val_at(*offset)?;
        *offset += 1;
        self.to = decode_to(rlp, offset)?;
        self.value = rlp.val_at(*offset)?;
        *offset += 1;
        let input = rlp::Rlp::new(rlp.at(*offset)?.as_raw()).data()?;
        self.input = Bytes::from(input.to_vec());
        *offset += 1;
        self.access_list = Some(rlp.val_at(*offset)?);
        *offset += 1;
        Ok(())
    }

    /// Decodes fields of the type 1 transaction response based on the RLP offset passed.
    /// Increments the offset for each element parsed.
    fn decode_base_eip2930(
        &mut self,
        rlp: &rlp::Rlp,
        offset: &mut usize,
    ) -> Result<(), DecoderError> {
        self.chain_id = Some(rlp.val_at(*offset)?);
        *offset += 1;
        self.nonce = rlp.val_at(*offset)?;
        *offset += 1;
        self.gas_price = Some(rlp.val_at(*offset)?);
        *offset += 1;
        self.gas = rlp.val_at(*offset)?;
        *offset += 1;

        #[cfg(feature = "celo")]
        self.decode_celo_metadata(rlp, offset)?;

        self.to = decode_to(rlp, offset)?;
        self.value = rlp.val_at(*offset)?;
        *offset += 1;
        let input = rlp::Rlp::new(rlp.at(*offset)?.as_raw()).data()?;
        self.input = Bytes::from(input.to_vec());
        *offset += 1;
        self.access_list = Some(rlp.val_at(*offset)?);
        *offset += 1;

        Ok(())
    }

    /// Decodes a legacy transaction starting at the RLP offset passed.
    /// Increments the offset for each element parsed.
    #[inline]
    fn decode_base_legacy(
        &mut self,
        rlp: &rlp::Rlp,
        offset: &mut usize,
    ) -> Result<(), DecoderError> {
        self.nonce = rlp.val_at(*offset)?;
        *offset += 1;
        self.gas_price = Some(rlp.val_at(*offset)?);
        *offset += 1;
        self.gas = rlp.val_at(*offset)?;
        *offset += 1;

        #[cfg(feature = "celo")]
        self.decode_celo_metadata(rlp, offset)?;

        self.to = decode_to(rlp, offset)?;
        self.value = rlp.val_at(*offset)?;
        *offset += 1;
        let input = rlp::Rlp::new(rlp.at(*offset)?.as_raw()).data()?;
        self.input = Bytes::from(input.to_vec());
        *offset += 1;
        Ok(())
    }

    /// Recover the sender of the tx from signature
    pub fn recover_from(&self) -> Result<Address, SignatureError> {
        let signature = Signature { r: self.r, s: self.s, v: self.v.as_u64() };
        let typed_tx: TypedTransaction = self.into();
        signature.recover(typed_tx.sighash())
    }

    /// Recover the sender of the tx from signature and set the from field
    pub fn recover_from_mut(&mut self) -> Result<Address, SignatureError> {
        let from = self.recover_from()?;
        self.from = from;
        Ok(from)
    }
}

/// Get a Transaction directly from a rlp encoded byte stream
impl Decodable for Transaction {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, DecoderError> {
        let mut txn = Self { hash: H256(keccak256(rlp.as_raw())), ..Default::default() };
        // we can get the type from the first value
        let mut offset = 0;

        // only untyped legacy transactions are lists
        if rlp.is_list() {
            // Legacy (0x00)
            // use the original rlp
            txn.decode_base_legacy(rlp, &mut offset)?;
            let sig = decode_signature(rlp, &mut offset)?;
            txn.r = sig.r;
            txn.s = sig.s;
            txn.v = sig.v.into();
            // extract chain id if legacy
            txn.chain_id = extract_chain_id(sig.v).map(|id| id.as_u64().into());
        } else {
            // if it is not enveloped then we need to use rlp.as_raw instead of rlp.data
            let first_byte = rlp.as_raw()[0];
            let (first, data) = if first_byte <= 0x7f {
                (first_byte, rlp.as_raw())
            } else {
                let data = rlp.data()?;
                let first = *data.first().ok_or(DecoderError::Custom("empty slice"))?;
                (first, data)
            };

            let bytes = data.get(1..).ok_or(DecoderError::Custom("no tx body"))?;
            let rest = rlp::Rlp::new(bytes);
            match first {
                0x01 => {
                    txn.decode_base_eip2930(&rest, &mut offset)?;
                    txn.transaction_type = Some(1u64.into());
                }
                0x02 => {
                    txn.decode_base_eip1559(&rest, &mut offset)?;
                    txn.transaction_type = Some(2u64.into());
                }
                _ => return Err(DecoderError::Custom("invalid tx type")),
            }

            let odd_y_parity: bool = rest.val_at(offset)?;
            txn.v = (odd_y_parity as u8).into();
            txn.r = rest.val_at(offset + 1)?;
            txn.s = rest.val_at(offset + 2)?;
        }

        Ok(txn)
    }
}

/// "Receipt" of an executed transaction: details of its execution.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// address of the sender.
    pub from: Address,
    // address of the receiver. null when its a contract creation transaction.
    pub to: Option<Address>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<H256>,
    /// Logs bloom
    #[serde(rename = "logsBloom")]
    pub logs_bloom: Bloom,
    /// Transaction type, Some(1) for AccessList transaction, None for Legacy
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<U64>,
    /// The price paid post-execution by the transaction (i.e. base fee + priority fee).
    /// Both fields in 1559-style transactions are *maximums* (max fee + max priority fee), the
    /// amount that's actually paid by users can only be determined post-execution
    #[serde(rename = "effectiveGasPrice", default, skip_serializing_if = "Option::is_none")]
    pub effective_gas_price: Option<U256>,
    /// Captures unknown fields such as additional fields used by L2s
    #[cfg(not(feature = "celo"))]
    #[serde(flatten)]
    pub other: crate::types::OtherFields,
}

impl rlp::Encodable for TransactionReceipt {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(4);
        rlp_opt(s, &self.status);
        s.append(&self.cumulative_gas_used);
        s.append(&self.logs_bloom);
        s.append_list(&self.logs);
    }
}

// Compares the transaction receipt against another receipt by checking the blocks first and then
// the transaction index in the block
impl Ord for TransactionReceipt {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.block_number, other.block_number) {
            (Some(number), Some(other_number)) => match number.cmp(&other_number) {
                Ordering::Equal => self.transaction_index.cmp(&other.transaction_index),
                ord => ord,
            },
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => self.transaction_index.cmp(&other.transaction_index),
        }
    }
}

impl PartialOrd<Self> for TransactionReceipt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
#[cfg(not(any(feature = "celo", feature = "optimism")))]
mod tests {
    use rlp::{Encodable, Rlp};

    use crate::types::transaction::eip2930::AccessListItem;

    use super::*;
    use std::str::FromStr;

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

    #[test]
    fn decode_london_receipt() {
        let receipt: TransactionReceipt = serde_json::from_value(serde_json::json!({"blockHash":"0x55ae43d3511e327dc532855510d110676d340aa1bbba369b4b98896d86559586","blockNumber":"0xa3d322","contractAddress":null,"cumulativeGasUsed":"0x207a5b","effectiveGasPrice":"0x3b9aca07","from":"0x541d6a0e9ca9e7a083e41e2e178eef9f22d7492e","gasUsed":"0x6a40","logs":[],"logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","status":"0x1","to":"0x8210357f377e901f18e45294e86a2a32215cc3c9","transactionHash":"0x824384376c5972498c6fcafe71fd8cad1689f64e7d5e270d025a898638c0c34d","transactionIndex":"0xd","type":"0x2"})).unwrap();
        assert_eq!(receipt.transaction_type.unwrap().as_u64(), 2);
        assert_eq!(receipt.effective_gas_price.unwrap().as_u64(), 0x3b9aca07);
    }

    #[test]
    fn decode_london_tx() {
        let tx: Transaction = serde_json::from_value(serde_json::json!({"accessList":[{"address":"0x8ba1f109551bd432803012645ac136ddd64dba72","storageKeys":["0x0000000000000000000000000000000000000000000000000000000000000000","0x0000000000000000000000000000000000000000000000000000000000000042"]}],"blockHash":"0x55ae43d3511e327dc532855510d110676d340aa1bbba369b4b98896d86559586","blockNumber":"0xa3d322","chainId":"0x3","from":"0x541d6a0e9ca9e7a083e41e2e178eef9f22d7492e","gas":"0x6a40","gasPrice":"0x3b9aca07","hash":"0x824384376c5972498c6fcafe71fd8cad1689f64e7d5e270d025a898638c0c34d","input":"0x","maxFeePerGas":"0x3b9aca0e","maxPriorityFeePerGas":"0x3b9aca00","nonce":"0x2","r":"0xf13b5088108f783f4b6048d4be456971118aabfb88be96bb541d734b6c2b20dc","s":"0x13fb7eb25a7d5df42a176cd4c6a086e19163ed7cd8ffba015f939d24f66bc17a","to":"0x8210357f377e901f18e45294e86a2a32215cc3c9","transactionIndex":"0xd","type":"0x2","v":"0x1","value":"0x7b"})).unwrap();
        assert_eq!(tx.transaction_type.unwrap().as_u64(), 2);
        let lst = AccessList(vec![AccessListItem {
            address: "0x8ba1f109551bd432803012645ac136ddd64dba72".parse().unwrap(),
            storage_keys: vec![
                "0x0000000000000000000000000000000000000000000000000000000000000000"
                    .parse()
                    .unwrap(),
                "0x0000000000000000000000000000000000000000000000000000000000000042"
                    .parse()
                    .unwrap(),
            ],
        }]);
        assert_eq!(tx.access_list.unwrap(), lst);
        assert_eq!(tx.max_fee_per_gas.unwrap().as_u64(), 0x3b9aca0e);
        assert_eq!(tx.max_priority_fee_per_gas.unwrap().as_u64(), 0x3b9aca00);
    }

    #[test]
    fn tx_roundtrip() {
        let json = serde_json::json!({"accessList":[{"address":"0x8ba1f109551bd432803012645ac136ddd64dba72","storageKeys":["0x0000000000000000000000000000000000000000000000000000000000000000","0x0000000000000000000000000000000000000000000000000000000000000042"]}],"blockHash":"0x55ae43d3511e327dc532855510d110676d340aa1bbba369b4b98896d86559586","blockNumber":"0xa3d322","chainId":"0x3","from":"0x541d6a0e9ca9e7a083e41e2e178eef9f22d7492e","gas":"0x6a40","gasPrice":"0x3b9aca07","hash":"0x824384376c5972498c6fcafe71fd8cad1689f64e7d5e270d025a898638c0c34d","input":"0x","maxFeePerGas":"0x3b9aca0e","maxPriorityFeePerGas":"0x3b9aca00","nonce":"0x0","r":"0xf13b5088108f783f4b6048d4be456971118aabfb88be96bb541d734b6c2b20dc","s":"0x13fb7eb25a7d5df42a176cd4c6a086e19163ed7cd8ffba015f939d24f66bc17a","to":"0x8210357f377e901f18e45294e86a2a32215cc3c9","transactionIndex":"0xd","type":"0x2","v":"0x1","value":"0x7b"});
        let tx: Transaction = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(tx.nonce, 0u64.into());

        let encoded = serde_json::to_value(tx).unwrap();
        assert_eq!(encoded, json);
    }

    #[test]
    fn rlp_london_tx() {
        let tx = Transaction {
            block_hash: None,
            block_number: None,
            from: Address::from_str("057f8d0f6fb2703197363f75c002f766f1c4287a").unwrap(),
            gas: U256::from_str_radix("0x6d22", 16).unwrap(),
            gas_price: Some(U256::from_str_radix("0x1344ead983", 16).unwrap()),
            hash: H256::from_str(
                "781d57642f4e3277fe01d370bd45ba1361b475bea6a35f26814e02a0a2b26549",
            )
            .unwrap(),
            max_fee_per_gas: Some(U256::from_str_radix("0x1344ead983", 16).unwrap()),
            max_priority_fee_per_gas: Some(U256::from_str_radix("0x1344ead983", 16).unwrap()),
            input: Bytes::from(hex::decode("d0e30db0").unwrap()),
            nonce: U256::from(479),
            to: Some(Address::from_str("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap()),
            transaction_index: None,
            value: U256::from_str_radix("0x2b40d6d551c8970c", 16).unwrap(),
            transaction_type: Some(U64::from(0x2)),
            access_list: Some(AccessList::from(vec![])),
            chain_id: Some(U256::from(1)),
            v: U64::from(0x1),
            r: U256::from_str_radix(
                "0x5616cdaec839ca14d209b59eafb706e623169dc9d0fa58fbf13931cef5b5e3b0",
                16,
            )
            .unwrap(),
            s: U256::from_str_radix(
                "0x3e708f8044bd158d29c2e250b6a98ea637c3bc460beeea63a8f00f7cebac432a",
                16,
            )
            .unwrap(),
            other: Default::default(),
        };
        println!("0x{}", hex::encode(&tx.rlp()));
        assert_eq!(
            tx.rlp(),
            Bytes::from(
                hex::decode("02f87a018201df851344ead983851344ead983826d2294c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2882b40d6d551c8970c84d0e30db0c001a05616cdaec839ca14d209b59eafb706e623169dc9d0fa58fbf13931cef5b5e3b0a03e708f8044bd158d29c2e250b6a98ea637c3bc460beeea63a8f00f7cebac432a").unwrap()
            )
        );
    }

    #[test]
    fn rlp_london_no_access_list() {
        let tx = Transaction {
            block_hash: None,
            block_number: None,
            from: Address::from_str("057f8d0f6fb2703197363f75c002f766f1c4287a").unwrap(),
            gas: U256::from_str_radix("0x6d22", 16).unwrap(),
            gas_price: Some(U256::from_str_radix("0x1344ead983", 16).unwrap()),
            hash: H256::from_str(
                "781d57642f4e3277fe01d370bd45ba1361b475bea6a35f26814e02a0a2b26549",
            )
            .unwrap(),
            max_fee_per_gas: Some(U256::from_str_radix("0x1344ead983", 16).unwrap()),
            max_priority_fee_per_gas: Some(U256::from_str_radix("0x1344ead983", 16).unwrap()),
            input: Bytes::from(hex::decode("d0e30db0").unwrap()),
            nonce: U256::from(479),
            to: Some(Address::from_str("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap()),
            transaction_index: None,
            value: U256::from_str_radix("0x2b40d6d551c8970c", 16).unwrap(),
            transaction_type: Some(U64::from(0x2)),
            access_list: None,
            chain_id: Some(U256::from(1)),
            v: U64::from(0x1),
            r: U256::from_str_radix(
                "0x5616cdaec839ca14d209b59eafb706e623169dc9d0fa58fbf13931cef5b5e3b0",
                16,
            )
            .unwrap(),
            s: U256::from_str_radix(
                "0x3e708f8044bd158d29c2e250b6a98ea637c3bc460beeea63a8f00f7cebac432a",
                16,
            )
            .unwrap(),
            other: Default::default(),
        };
        println!("0x{}", hex::encode(&tx.rlp()));
        assert_eq!(
            tx.rlp(),
            Bytes::from(
                hex::decode("02f87a018201df851344ead983851344ead983826d2294c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2882b40d6d551c8970c84d0e30db0c001a05616cdaec839ca14d209b59eafb706e623169dc9d0fa58fbf13931cef5b5e3b0a03e708f8044bd158d29c2e250b6a98ea637c3bc460beeea63a8f00f7cebac432a").unwrap()
            )
        );
    }

    #[test]
    fn rlp_legacy_tx() {
        let tx = Transaction {
            block_hash: None,
            block_number: None,
            from: Address::from_str("c26ad91f4e7a0cad84c4b9315f420ca9217e315d").unwrap(),
            gas: U256::from_str_radix("0x10e2b", 16).unwrap(),
            gas_price: Some(U256::from_str_radix("0x12ec276caf", 16).unwrap()),
            hash: H256::from_str("929ff27a5c7833953df23103c4eb55ebdfb698678139d751c51932163877fada").unwrap(),
            input: Bytes::from(
                hex::decode("a9059cbb000000000000000000000000fdae129ecc2c27d166a3131098bc05d143fa258e0000000000000000000000000000000000000000000000000000000002faf080").unwrap()
            ),
            nonce: U256::zero(),
            to: Some(Address::from_str("dac17f958d2ee523a2206206994597c13d831ec7").unwrap()),
            transaction_index: None,
            value: U256::zero(),
            transaction_type: Some(U64::zero()),
            v: U64::from(0x25),
            r: U256::from_str_radix("c81e70f9e49e0d3b854720143e86d172fecc9e76ef8a8666f2fdc017017c5141", 16).unwrap(),
            s: U256::from_str_radix("1dd3410180f6a6ca3e25ad3058789cd0df3321ed76b5b4dbe0a2bb2dc28ae274", 16).unwrap(),
            chain_id: Some(U256::from(1)),
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            other: Default::default()
        };
        assert_eq!(
            tx.rlp(),
            Bytes::from(
                hex::decode("f8aa808512ec276caf83010e2b94dac17f958d2ee523a2206206994597c13d831ec780b844a9059cbb000000000000000000000000fdae129ecc2c27d166a3131098bc05d143fa258e0000000000000000000000000000000000000000000000000000000002faf08025a0c81e70f9e49e0d3b854720143e86d172fecc9e76ef8a8666f2fdc017017c5141a01dd3410180f6a6ca3e25ad3058789cd0df3321ed76b5b4dbe0a2bb2dc28ae274").unwrap()
            )
        );
    }

    #[test]
    fn rlp_london_goerli() {
        let tx = Transaction {
            hash: H256::from_str(
                "5e2fc091e15119c97722e9b63d5d32b043d077d834f377b91f80d32872c78109",
            )
            .unwrap(),
            nonce: 65.into(),
            block_hash: Some(
                H256::from_str("f43869e67c02c57d1f9a07bb897b54bec1cfa1feb704d91a2ee087566de5df2c")
                    .unwrap(),
            ),
            block_number: Some(6203173.into()),
            transaction_index: Some(10.into()),
            from: Address::from_str("e66b278fa9fbb181522f6916ec2f6d66ab846e04").unwrap(),
            to: Some(Address::from_str("11d7c2ab0d4aa26b7d8502f6a7ef6844908495c2").unwrap()),
            value: 0.into(),
            gas_price: Some(1500000007.into()),
            gas: 106703.into(),
            input: hex::decode("e5225381").unwrap().into(),
            v: 1.into(),
            r: U256::from_str_radix(
                "12010114865104992543118914714169554862963471200433926679648874237672573604889",
                10,
            )
            .unwrap(),
            s: U256::from_str_radix(
                "22830728216401371437656932733690354795366167672037272747970692473382669718804",
                10,
            )
            .unwrap(),
            transaction_type: Some(2.into()),
            access_list: Some(AccessList::default()),
            max_priority_fee_per_gas: Some(1500000000.into()),
            max_fee_per_gas: Some(1500000009.into()),
            chain_id: Some(5.into()),
            other: Default::default(),
        };
        assert_eq!(
            tx.rlp(),
            Bytes::from(
                hex::decode("02f86f05418459682f008459682f098301a0cf9411d7c2ab0d4aa26b7d8502f6a7ef6844908495c28084e5225381c001a01a8d7bef47f6155cbdf13d57107fc577fd52880fa2862b1a50d47641f8839419a03279bbf73fde76de83440d04b9d97f3809fec8617d3557ee40ac3e0edc391514").unwrap()
            )
        );
    }

    // <https://goerli.etherscan.io/tx/0x5e2fc091e15119c97722e9b63d5d32b043d077d834f377b91f80d32872c78109>
    #[test]
    fn decode_rlp_london_goerli() {
        let tx = Transaction {
            hash: H256::from_str(
                "5e2fc091e15119c97722e9b63d5d32b043d077d834f377b91f80d32872c78109",
            )
            .unwrap(),
            nonce: 65.into(),
            block_hash: Some(
                H256::from_str("f43869e67c02c57d1f9a07bb897b54bec1cfa1feb704d91a2ee087566de5df2c")
                    .unwrap(),
            ),
            block_number: Some(6203173.into()),
            transaction_index: Some(10.into()),
            from: Address::from_str("e66b278fa9fbb181522f6916ec2f6d66ab846e04").unwrap(),
            to: Some(Address::from_str("11d7c2ab0d4aa26b7d8502f6a7ef6844908495c2").unwrap()),
            value: 0.into(),
            gas_price: Some(1500000007.into()),
            gas: 106703.into(),
            input: hex::decode("e5225381").unwrap().into(),
            v: 1.into(),
            r: U256::from_str_radix(
                "12010114865104992543118914714169554862963471200433926679648874237672573604889",
                10,
            )
            .unwrap(),
            s: U256::from_str_radix(
                "22830728216401371437656932733690354795366167672037272747970692473382669718804",
                10,
            )
            .unwrap(),
            transaction_type: Some(2.into()),
            access_list: Some(AccessList::default()),
            max_priority_fee_per_gas: Some(1500000000.into()),
            max_fee_per_gas: Some(1500000009.into()),
            chain_id: Some(5.into()),
            other: Default::default(),
        };

        let tx_bytes = hex::decode("02f86f05418459682f008459682f098301a0cf9411d7c2ab0d4aa26b7d8502f6a7ef6844908495c28084e5225381c001a01a8d7bef47f6155cbdf13d57107fc577fd52880fa2862b1a50d47641f8839419a03279bbf73fde76de83440d04b9d97f3809fec8617d3557ee40ac3e0edc391514").unwrap();

        // the `Transaction` a valid rlp input,
        // but EIP-1559 prepends a version byte, so we need to encode the data first to get a
        // valid rlp and then rlp decode impl of `Transaction` will remove and check the
        // version byte

        let rlp_bytes = rlp::encode(&tx_bytes);
        let decoded_transaction = Transaction::decode(&rlp::Rlp::new(&rlp_bytes)).unwrap();

        assert_eq!(
            decoded_transaction.hash(),
            "0x5e2fc091e15119c97722e9b63d5d32b043d077d834f377b91f80d32872c78109".parse().unwrap()
        );
        assert_eq!(decoded_transaction.hash(), tx.hash());

        let from = decoded_transaction.recover_from().unwrap();
        assert_eq!(from, "0xe66b278fa9fbb181522f6916ec2f6d66ab846e04".parse().unwrap());
    }

    /// <https://etherscan.io/tx/0x280cde7cdefe4b188750e76c888f13bd05ce9a4d7767730feefe8a0e50ca6fc4>
    /// https://github.com/gakonst/ethers-rs/issues/1732
    #[test]
    fn test_rlp_decoding_issue_1732() {
        let raw_tx = "f9015482078b8505d21dba0083022ef1947a250d5630b4cf539739df2c5dacb4c659f2488d880c46549a521b13d8b8e47ff36ab50000000000000000000000000000000000000000000066ab5a608bd00a23f2fe000000000000000000000000000000000000000000000000000000000000008000000000000000000000000048c04ed5691981c42154c6167398f95e8f38a7ff00000000000000000000000000000000000000000000000000000000632ceac70000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000006c6ee5e31d828de241282b9606c8e98ea48526e225a0c9077369501641a92ef7399ff81c21639ed4fd8fc69cb793cfa1dbfab342e10aa0615facb2f1bcf3274a354cfe384a38d0cc008a11c2dd23a69111bc6930ba27a8";

        let rlp_bytes = hex::decode(raw_tx).unwrap();

        let decoded_tx: Transaction = rlp::decode(&rlp_bytes).unwrap();

        assert_eq!(
            decoded_tx.recover_from().unwrap(),
            "0xa12e1462d0ced572f396f58b6e2d03894cd7c8a4".parse().unwrap()
        );
    }

    #[test]
    fn decode_rlp_legacy() {
        let tx = Transaction {
            block_hash: None,
            block_number: None,
            from: Address::from_str("c26ad91f4e7a0cad84c4b9315f420ca9217e315d").unwrap(),
            gas: U256::from_str_radix("0x10e2b", 16).unwrap(),
            gas_price: Some(U256::from_str_radix("0x12ec276caf", 16).unwrap()),
            hash: H256::from_str("929ff27a5c7833953df23103c4eb55ebdfb698678139d751c51932163877fada").unwrap(),
            input: Bytes::from(
                hex::decode("a9059cbb000000000000000000000000fdae129ecc2c27d166a3131098bc05d143fa258e0000000000000000000000000000000000000000000000000000000002faf080").unwrap()
            ),
            nonce: U256::zero(),
            to: Some(Address::from_str("dac17f958d2ee523a2206206994597c13d831ec7").unwrap()),
            transaction_index: None,
            value: U256::zero(),
            transaction_type: Some(U64::zero()),
            v: U64::from(0x25),
            r: U256::from_str_radix("c81e70f9e49e0d3b854720143e86d172fecc9e76ef8a8666f2fdc017017c5141", 16).unwrap(),
            s: U256::from_str_radix("1dd3410180f6a6ca3e25ad3058789cd0df3321ed76b5b4dbe0a2bb2dc28ae274", 16).unwrap(),
            chain_id: Some(U256::from(1)),
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            other: Default::default()
        };

        let rlp_bytes = hex::decode("f8aa808512ec276caf83010e2b94dac17f958d2ee523a2206206994597c13d831ec780b844a9059cbb000000000000000000000000fdae129ecc2c27d166a3131098bc05d143fa258e0000000000000000000000000000000000000000000000000000000002faf08025a0c81e70f9e49e0d3b854720143e86d172fecc9e76ef8a8666f2fdc017017c5141a01dd3410180f6a6ca3e25ad3058789cd0df3321ed76b5b4dbe0a2bb2dc28ae274").unwrap();

        let decoded_transaction = Transaction::decode(&rlp::Rlp::new(&rlp_bytes)).unwrap();

        assert_eq!(decoded_transaction.hash(), tx.hash());
    }

    // <https://etherscan.io/tx/0x929ff27a5c7833953df23103c4eb55ebdfb698678139d751c51932163877fada>
    #[test]
    fn decode_rlp_legacy_in_envelope() {
        let tx = Transaction {
            block_hash: None,
            block_number: None,
            from: Address::from_str("c26ad91f4e7a0cad84c4b9315f420ca9217e315d").unwrap(),
            gas: U256::from_str_radix("0x10e2b", 16).unwrap(),
            gas_price: Some(U256::from_str_radix("0x12ec276caf", 16).unwrap()),
            hash: H256::from_str("929ff27a5c7833953df23103c4eb55ebdfb698678139d751c51932163877fada").unwrap(),
            input: Bytes::from(
                hex::decode("a9059cbb000000000000000000000000fdae129ecc2c27d166a3131098bc05d143fa258e0000000000000000000000000000000000000000000000000000000002faf080").unwrap()
            ),
            nonce: U256::zero(),
            to: Some(Address::from_str("dac17f958d2ee523a2206206994597c13d831ec7").unwrap()),
            transaction_index: None,
            value: U256::zero(),
            transaction_type: Some(U64::zero()),
            v: U64::from(0x25),
            r: U256::from_str_radix("c81e70f9e49e0d3b854720143e86d172fecc9e76ef8a8666f2fdc017017c5141", 16).unwrap(),
            s: U256::from_str_radix("1dd3410180f6a6ca3e25ad3058789cd0df3321ed76b5b4dbe0a2bb2dc28ae274", 16).unwrap(),
            chain_id: Some(U256::from(1)),
            access_list: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            other: Default::default()
        };

        let rlp_bytes = hex::decode("f8aa808512ec276caf83010e2b94dac17f958d2ee523a2206206994597c13d831ec780b844a9059cbb000000000000000000000000fdae129ecc2c27d166a3131098bc05d143fa258e0000000000000000000000000000000000000000000000000000000002faf08025a0c81e70f9e49e0d3b854720143e86d172fecc9e76ef8a8666f2fdc017017c5141a01dd3410180f6a6ca3e25ad3058789cd0df3321ed76b5b4dbe0a2bb2dc28ae274").unwrap();

        let decoded = Transaction::decode(&rlp::Rlp::new(&rlp_bytes)).unwrap();
        assert_eq!(
            decoded.hash(),
            "929ff27a5c7833953df23103c4eb55ebdfb698678139d751c51932163877fada".parse().unwrap()
        );
        assert_eq!(decoded.hash(), tx.hash());
        assert_eq!(
            decoded.recover_from().unwrap(),
            "0xc26ad91f4e7a0cad84c4b9315f420ca9217e315d".parse().unwrap()
        );
    }

    // Reference tx hash on Ethereum mainnet:
    // 0x938913ef1df8cd17e0893a85586ade463014559fb1bd2d536ac282f3b1bdea53
    #[test]
    fn decode_tx_assert_hash() {
        let raw_tx = hex::decode("02f874018201bb8405f5e10085096a1d45b782520894d696a5c568160bbbf5a1356f8ac56ee81a190588871550f7dca7000080c080a07df2299b0181d6d5b817795a7d2eff5897d0d3914ff5f602e17d5b75d32ec25fa051833973e8a8c222e682d2dcea02ad7bf3ec5bc3a86bfbcdbbaa3b853e52ad08").unwrap();
        let tx: Transaction = Transaction::decode(&Rlp::new(&raw_tx)).unwrap();
        assert_eq!(
            tx.hash,
            H256::from_str("938913ef1df8cd17e0893a85586ade463014559fb1bd2d536ac282f3b1bdea53")
                .unwrap()
        )
    }

    #[test]
    fn recover_from() {
        let tx = Transaction {
            hash: H256::from_str(
                "5e2fc091e15119c97722e9b63d5d32b043d077d834f377b91f80d32872c78109",
            )
            .unwrap(),
            nonce: 65.into(),
            block_hash: Some(
                H256::from_str("f43869e67c02c57d1f9a07bb897b54bec1cfa1feb704d91a2ee087566de5df2c")
                    .unwrap(),
            ),
            block_number: Some(6203173.into()),
            transaction_index: Some(10.into()),
            from: Address::from_str("e66b278fa9fbb181522f6916ec2f6d66ab846e04").unwrap(),
            to: Some(Address::from_str("11d7c2ab0d4aa26b7d8502f6a7ef6844908495c2").unwrap()),
            value: 0.into(),
            gas_price: Some(1500000007.into()),
            gas: 106703.into(),
            input: hex::decode("e5225381").unwrap().into(),
            v: 1.into(),
            r: U256::from_str_radix(
                "12010114865104992543118914714169554862963471200433926679648874237672573604889",
                10,
            )
            .unwrap(),
            s: U256::from_str_radix(
                "22830728216401371437656932733690354795366167672037272747970692473382669718804",
                10,
            )
            .unwrap(),
            transaction_type: Some(2.into()),
            access_list: Some(AccessList::default()),
            max_priority_fee_per_gas: Some(1500000000.into()),
            max_fee_per_gas: Some(1500000009.into()),
            chain_id: Some(5.into()),
            other: Default::default(),
        };

        assert_eq!(tx.hash, tx.hash());
        assert_eq!(tx.from, tx.recover_from().unwrap());
    }

    #[test]
    fn decode_transaction_receipt() {
        let _res: TransactionReceipt = serde_json::from_str(
            r#"{
        "transactionHash": "0xa3ece39ae137617669c6933b7578b94e705e765683f260fcfe30eaa41932610f",
        "blockHash": "0xf6084155ff2022773b22df3217d16e9df53cbc42689b27ca4789e06b6339beb2",
        "blockNumber": "0x52a975",
        "contractAddress": null,
        "cumulativeGasUsed": "0x797db0",
        "from": "0xd907941c8b3b966546fc408b8c942eb10a4f98df",
        "gasUsed": "0x1308c",
        "logs": [
            {
                "blockHash": "0xf6084155ff2022773b22df3217d16e9df53cbc42689b27ca4789e06b6339beb2",
                "address": "0xd6df5935cd03a768b7b9e92637a01b25e24cb709",
                "logIndex": "0x119",
                "data": "0x0000000000000000000000000000000000000000000000000000008bb2c97000",
                "removed": false,
                "topics": [
                    "0x8940c4b8e215f8822c5c8f0056c12652c746cbc57eedbd2a440b175971d47a77",
                    "0x000000000000000000000000d907941c8b3b966546fc408b8c942eb10a4f98df"
                ],
                "blockNumber": "0x52a975",
                "transactionIndex": "0x29",
                "transactionHash": "0xa3ece39ae137617669c6933b7578b94e705e765683f260fcfe30eaa41932610f"
            },
            {
                "blockHash": "0xf6084155ff2022773b22df3217d16e9df53cbc42689b27ca4789e06b6339beb2",
                "address": "0xd6df5935cd03a768b7b9e92637a01b25e24cb709",
                "logIndex": "0x11a",
                "data": "0x0000000000000000000000000000000000000000000000000000008bb2c97000",
                "removed": false,
                "topics": [
                    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                    "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "0x000000000000000000000000d907941c8b3b966546fc408b8c942eb10a4f98df"
                ],
                "blockNumber": "0x52a975",
                "transactionIndex": "0x29",
                "transactionHash": "0xa3ece39ae137617669c6933b7578b94e705e765683f260fcfe30eaa41932610f"
            }
        ],
        "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000020000000000000000000800000000000000004010000010100000000000000000000000000000000000000000000000000040000080000000000000080000000000000000000000000000000000000000000020000000000000000000000002000000000000000000000000000000000000000000000000000020000000010000000000000000000000000000000000000000000000000000000000",
        "root": null,
        "status": "0x1",
        "to": "0xd6df5935cd03a768b7b9e92637a01b25e24cb709",
        "transactionIndex": "0x29"
    }"#,
        )
        .unwrap();
    }

    #[test]
    fn serde_create_transaction_receipt() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{
    "transactionHash": "0x611b173b0e0dfda94da7bfb6cb77c9f1c03e2f2149ba060e6bddfaa219942369",
    "blockHash": "0xa11871d61e0e703ae33b358a6a9653c43e4216f277d4a1c7377b76b4d5b4cbf1",
    "blockNumber": "0xe3c1d8",
    "contractAddress": "0x08f6db30039218894067023a3593baf27d3f4a2b",
    "cumulativeGasUsed": "0x1246047",
    "effectiveGasPrice": "0xa02ffee00",
    "from": "0x0968995a48162a23af60d3ca25cddfa143cd8891",
    "gasUsed": "0x1b9229",
    "logs": [
      {
        "address": "0x08f6db30039218894067023a3593baf27d3f4a2b",
        "topics": [
          "0x40c340f65e17194d14ddddb073d3c9f888e3cb52b5aae0c6c7706b4fbc905fac"
        ],
        "data": "0x0000000000000000000000000968995a48162a23af60d3ca25cddfa143cd88910000000000000000000000000000000000000000000000000000000000002616",
        "blockNumber": "0xe3c1d8",
        "transactionHash": "0x611b173b0e0dfda94da7bfb6cb77c9f1c03e2f2149ba060e6bddfaa219942369",
        "transactionIndex": "0xdf",
        "blockHash": "0xa11871d61e0e703ae33b358a6a9653c43e4216f277d4a1c7377b76b4d5b4cbf1",
        "logIndex": "0x196",
        "removed": false
      },
      {
        "address": "0x08f6db30039218894067023a3593baf27d3f4a2b",
        "topics": [
          "0x40c340f65e17194d14ddddb073d3c9f888e3cb52b5aae0c6c7706b4fbc905fac"
        ],
        "data": "0x00000000000000000000000059750ac0631f63bfdce0f0867618e468e11ee34700000000000000000000000000000000000000000000000000000000000000fa",
        "blockNumber": "0xe3c1d8",
        "transactionHash": "0x611b173b0e0dfda94da7bfb6cb77c9f1c03e2f2149ba060e6bddfaa219942369",
        "transactionIndex": "0xdf",
        "blockHash": "0xa11871d61e0e703ae33b358a6a9653c43e4216f277d4a1c7377b76b4d5b4cbf1",
        "logIndex": "0x197",
        "removed": false
      }
    ],
    "logsBloom": "0x00000000000000800000000040000000000000000000000000000000000000000000008000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "status": "0x1",
    "to": null,
    "transactionIndex": "0xdf",
    "type": "0x2"
}
"#,
        )
        .unwrap();

        let receipt: TransactionReceipt = serde_json::from_value(v.clone()).unwrap();
        assert!(receipt.to.is_none());
        let receipt = serde_json::to_value(receipt).unwrap();
        assert_eq!(v, receipt);
    }

    #[test]
    fn rlp_encode_receipt() {
        let receipt = TransactionReceipt { status: Some(1u64.into()), ..Default::default() };
        let encoded = receipt.rlp_bytes();

        assert_eq!(
            encoded,
            hex::decode("f901060180b9010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c0").unwrap(),
        );
    }

    #[test]
    fn can_sort_receipts() {
        let mut a = TransactionReceipt { block_number: Some(0u64.into()), ..Default::default() };
        let b = TransactionReceipt { block_number: Some(1u64.into()), ..Default::default() };
        assert!(a < b);

        a = b.clone();
        assert_eq!(a.cmp(&b), Ordering::Equal);

        a.transaction_index = 1u64.into();
        assert!(a > b);
    }

    // from https://github.com/gakonst/ethers-rs/issues/1762
    #[test]
    fn test_rlp_decoding_type_2() {
        use crate::types::*;

        let raw_tx = "0x02f906f20103843b9aca0085049465153e830afdd19468b3465833fb72a70ecdf485e0e4c7bd8665fc4580b906845ae401dc00000000000000000000000000000000000000000000000000000000633c4c730000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000003200000000000000000000000000000000000000000000000000000000000000460000000000000000000000000000000000000000000000000000000000000058000000000000000000000000000000000000000000000000000000000000000e404e45aaf000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec700000000000000000000000012b6893ce26ea6341919fe289212ef77e51688c800000000000000000000000000000000000000000000000000000000000027100000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000017754984000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000124b858183f000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000017754984000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000042dac17f958d2ee523a2206206994597c13d831ec7000bb8c02aaa39b223fe8d0a0e5c4f27ead9083c756cc200271012b6893ce26ea6341919fe289212ef77e51688c8000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000104b858183f00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000006d78ac6800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bdac17f958d2ee523a2206206994597c13d831ec70001f4c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e4472b43f30000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001f8aa12f280116c88954000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc200000000000000000000000012b6893ce26ea6341919fe289212ef77e51688c8000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000064df2ab5bb00000000000000000000000012b6893ce26ea6341919fe289212ef77e51688c8000000000000000000000000000000000000000000002d092097defac5b7a01a000000000000000000000000f69a7cd9649a5b5477fa0e5395385fad03ac639f00000000000000000000000000000000000000000000000000000000c001a0127484994706ff8605f1da80e7bdf0efa3e26192a094413e58d409551398b0b5a06fd706e38eebeba2f235e37ceb0acb426f1e6c91702add97810ee677a15d1980";
        let mut decoded_tx = crate::utils::rlp::decode::<Transaction>(
            &raw_tx.parse::<Bytes>().expect("unable to parse raw tx"),
        )
        .expect("unable to decode raw tx");
        decoded_tx.recover_from_mut().unwrap();
        decoded_tx.hash = decoded_tx.hash();
        assert_eq!(
            H256::from_str("0xeae304417079580c334ccc07e3933a906699461802a17b722034a8191c4a38ea")
                .unwrap(),
            decoded_tx.hash
        );
    }

    #[test]
    fn test_rlp_decoding_issue_1848_first() {
        // slot 5097934, tx index 40, hash
        // 0xf98c9f1a2f30ee316ea1db18c132ccab6383b8e4933ccf6259ca9d1f27d4a364
        let s = "01f9012e01826c6f850737be7600830493ef940c3de458b51a11da7d4616f42f66c861e3859d3e80b8c4f5b22c2a000000000000000000000000e67b950f4b84c5b06ee36ded6727a17443fe749300000000000000000000000000000000000000000000005f344f4a335cc50000000000000000000000000000000000000000000005c2f00b834b7f0000000000000000000000000000000000000000000000000005aa64a95b4a40400000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000c3de458b51a11da7d4616f42f66c861e3859d3ec080a0c4023f0b8f7daecd7e143ef7aaa9b67bd059e643a6f2ae509a0e8483a3966e28a065a20662274cb5f7fe60a2af7dbd466244154440e73243f00b6a69bd08eacda4";
        let b = hex::decode(s).unwrap();
        let r = rlp::Rlp::new(b.as_slice());
        Transaction::decode(&r).unwrap();
    }

    #[test]
    fn test_rlp_decoding_issue_1848_second() {
        // slot 5097936, tx index 0, hash
        // 6d38fc8aee934858815ed41273cece3b676c368e9c6e39f172313a0685e1f175
        let s = "01f8ee0182034c853d9f1b88158307a120940087bb802d9c0e343f00510000729031ce00bf2780b8841e1326a300000000000000000000000088e6a0c2ddd26feeb64f039a2c41296fcb3f56400000000000000000000000000000000000000000000000000000001d3b3e730000000000000000000000000000000000000000000000000596b93e53696740000000000000000000000000000000000000000000000000000000000000000001c001a0bbfd754ed51b34d0a8577f69b4c42ce6b47fee6ecf49114bb135e7e8eadbb336a0433692134eb7e7686e9aefafa9f69c601aa977c00cc85c827782f5fb1f1cff0f";
        let b = hex::decode(s).unwrap();
        let r = rlp::Rlp::new(b.as_slice());
        Transaction::decode(&r).unwrap();
    }

    #[test]
    fn test_rlp_decoding_create_roundtrip() {
        let tx = Transaction {
            block_hash: None,
            block_number: None,
            from: Address::from_str("c26ad91f4e7a0cad84c4b9315f420ca9217e315d").unwrap(),
            gas: U256::from_str_radix("0x10e2b", 16).unwrap(),
            gas_price: Some(U256::from_str_radix("0x12ec276caf", 16).unwrap()),
            hash: H256::from_str("929ff27a5c7833953df23103c4eb55ebdfb698678139d751c51932163877fada").unwrap(),
            input: Bytes::from(
                hex::decode("a9059cbb000000000000000000000000fdae129ecc2c27d166a3131098bc05d143fa258e0000000000000000000000000000000000000000000000000000000002faf080").unwrap()
            ),
            nonce: U256::zero(),
            transaction_index: None,
            value: U256::zero(),
            ..Default::default()
        };
        Transaction::decode(&Rlp::new(&tx.rlp())).unwrap();
    }

    #[test]
    #[cfg(feature = "optimism")]
    fn test_rlp_encode_deposited_tx() {
        let deposited_tx = Transaction {
            hash: H256::from_str("0x7fd17d4a368fccdba4291ab121e48c96329b7dc3d027a373643fb23c20a19a3f").unwrap(),
            nonce: U256::from(4391989),
            block_hash: Some(H256::from_str("0xc2794a16acacd9f7670379ffd12b6968ff98e2a602f57d7d1f880220aa5a4973").unwrap()),
            block_number: Some(8453214u64.into()),
            transaction_index: Some(0u64.into()),
            from: Address::from_str("0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001").unwrap(),
            to: Some(Address::from_str("0x4200000000000000000000000000000000000015").unwrap()),
            value: U256::zero(),
            gas_price: Some(U256::zero()),
            gas: U256::from(1000000u64),
            input: Bytes::from(
                hex::decode("015d8eb90000000000000000000000000000000000000000000000000000000000878c1c00000000000000000000000000000000000000000000000000000000644662bc0000000000000000000000000000000000000000000000000000001ee24fba17b7e19cc10812911dfa8a438e0a81a9933f843aa5b528899b8d9e221b649ae0df00000000000000000000000000000000000000000000000000000000000000060000000000000000000000007431310e026b69bfc676c0013e12a1a11411eec9000000000000000000000000000000000000000000000000000000000000083400000000000000000000000000000000000000000000000000000000000f4240").unwrap()
            ),
            v: U64::zero(),
            r: U256::zero(),
            s: U256::zero(),
            source_hash: Some(H256::from_str("0xa8157ccf61bcdfbcb74a84ec1262e62644dd1e7e3614abcbd8db0c99a60049fc").unwrap()),
            mint: Some(0.into()),
            is_system_tx: None,
            transaction_type: Some(U64::from(126)),
            access_list: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            chain_id: None,
            other: Default::default()
        };

        let rlp = deposited_tx.rlp();

        let expected_rlp = Bytes::from(hex::decode("7ef90159a0a8157ccf61bcdfbcb74a84ec1262e62644dd1e7e3614abcbd8db0c99a60049fc94deaddeaddeaddeaddeaddeaddeaddeaddead00019442000000000000000000000000000000000000158080830f424080b90104015d8eb90000000000000000000000000000000000000000000000000000000000878c1c00000000000000000000000000000000000000000000000000000000644662bc0000000000000000000000000000000000000000000000000000001ee24fba17b7e19cc10812911dfa8a438e0a81a9933f843aa5b528899b8d9e221b649ae0df00000000000000000000000000000000000000000000000000000000000000060000000000000000000000007431310e026b69bfc676c0013e12a1a11411eec9000000000000000000000000000000000000000000000000000000000000083400000000000000000000000000000000000000000000000000000000000f4240").unwrap());

        assert_eq!(rlp, expected_rlp);
    }
}
