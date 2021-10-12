//! Transaction types
use super::{eip2930::AccessList, normalize_v, rlp_opt};
use crate::{
    types::{Address, Bloom, Bytes, Log, H256, U256, U64},
    utils::keccak256,
};
use rlp::RlpStream;
use serde::{Deserialize, Serialize};

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
        keccak256(&self.rlp().as_ref()).into()
    }

    pub fn rlp(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_unbounded_list();

        match self.transaction_type {
            // EIP-2930 (0x01)
            Some(x) if x == U64::from(1) => {
                rlp_opt(&mut rlp, &self.chain_id);
                rlp.append(&self.nonce);
                rlp_opt(&mut rlp, &self.gas_price);
                rlp.append(&self.gas);

                #[cfg(feature = "celo")]
                self.inject_celo_metadata(&mut rlp);

                rlp_opt(&mut rlp, &self.to);
                rlp.append(&self.value);
                rlp.append(&self.input.as_ref());
                rlp_opt(&mut rlp, &self.access_list);
                if let Some(chain_id) = self.chain_id {
                    rlp.append(&normalize_v(self.v.as_u64(), U64::from(chain_id.as_u64())));
                }
            }
            // EIP-1559 (0x02)
            Some(x) if x == U64::from(2) => {
                rlp_opt(&mut rlp, &self.chain_id);
                rlp.append(&self.nonce);
                rlp_opt(&mut rlp, &self.max_priority_fee_per_gas);
                rlp_opt(&mut rlp, &self.max_fee_per_gas);
                rlp.append(&self.gas);
                rlp_opt(&mut rlp, &self.to);
                rlp.append(&self.value);
                rlp.append(&self.input.as_ref());
                rlp_opt(&mut rlp, &self.access_list);
                if let Some(chain_id) = self.chain_id {
                    rlp.append(&normalize_v(self.v.as_u64(), U64::from(chain_id.as_u64())));
                }
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
            }
        }

        rlp.append(&self.r);
        rlp.append(&self.s);

        rlp.finalize_unbounded_list();

        let rlp_bytes: Bytes = rlp.out().freeze().into();
        let mut encoded = vec![];
        match self.transaction_type {
            Some(x) if x == U64::from(1) => {
                encoded.extend_from_slice(&[0x1]);
                encoded.extend_from_slice(rlp_bytes.as_ref());
                encoded.into()
            }
            Some(x) if x == U64::from(2) => {
                encoded.extend_from_slice(&[0x2]);
                encoded.extend_from_slice(rlp_bytes.as_ref());
                encoded.into()
            }
            _ => rlp_bytes,
        }
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
    /// Transaction type, Some(1) for AccessList transaction, None for Legacy
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<U64>,
    /// The price paid post-execution by the transaction (i.e. base fee + priority fee).
    /// Both fields in 1559-style transactions are *maximums* (max fee + max priority fee), the
    /// amount that's actually paid by users can only be determined post-execution
    #[serde(rename = "effectiveGasPrice", default, skip_serializing_if = "Option::is_none")]
    pub effective_gas_price: Option<U256>,
}

#[cfg(test)]
#[cfg(not(feature = "celo"))]
mod tests {
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
            max_priority_fee_per_gas: None
        };
        assert_eq!(
            tx.rlp(),
            Bytes::from(
                hex::decode("f8aa808512ec276caf83010e2b94dac17f958d2ee523a2206206994597c13d831ec780b844a9059cbb000000000000000000000000fdae129ecc2c27d166a3131098bc05d143fa258e0000000000000000000000000000000000000000000000000000000002faf08025a0c81e70f9e49e0d3b854720143e86d172fecc9e76ef8a8666f2fdc017017c5141a01dd3410180f6a6ca3e25ad3058789cd0df3321ed76b5b4dbe0a2bb2dc28ae274").unwrap()
            )
        );
    }
}
