use super::{eip2718::TypedTransaction, normalize_v};
use crate::types::{
    Address, Bytes, Signature, SignatureError, Transaction, TransactionRequest, H256, U256, U64,
};
use open_fastrlp::{
    RlpDecodable as FastRlpDecodable, RlpDecodableWrapper as FastRlpDecodableWrapper,
    RlpEncodable as FastRlpEncodable, RlpEncodableWrapper as FastRlpEncodableWrapper,
};
use rlp::{
    Decodable, RlpDecodable, RlpDecodableWrapper, RlpEncodable, RlpEncodableWrapper, RlpStream,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const NUM_EIP2930_FIELDS: usize = 8;

/// Access list
// NB: Need to use `RlpEncodableWrapper` else we get an extra [] in the output
// https://github.com/gakonst/ethers-rs/pull/353#discussion_r680683869
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    RlpEncodableWrapper,
    RlpDecodableWrapper,
    FastRlpEncodableWrapper,
    FastRlpDecodableWrapper,
)]
pub struct AccessList(pub Vec<AccessListItem>);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccessListWithGasUsed {
    pub access_list: AccessList,
    pub gas_used: U256,
}

impl From<Vec<AccessListItem>> for AccessList {
    fn from(src: Vec<AccessListItem>) -> AccessList {
        AccessList(src)
    }
}

impl TransactionRequest {
    /// Sets the `access_list` field in the transaction (converts the [`TransactionRequest`] to
    /// an [`Eip2930TransactionRequest`])
    pub fn with_access_list<T: Into<AccessList>>(
        self,
        access_list: T,
    ) -> Eip2930TransactionRequest {
        Eip2930TransactionRequest::new(self, access_list.into())
    }
}

/// Access list item
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    RlpEncodable,
    RlpDecodable,
    FastRlpEncodable,
    FastRlpDecodable,
)]
#[serde(rename_all = "camelCase")]
pub struct AccessListItem {
    /// Accessed address
    pub address: Address,
    /// Accessed storage keys
    pub storage_keys: Vec<H256>,
}

/// An error involving an EIP2930 transaction request.
#[derive(Debug, Error)]
pub enum Eip2930RequestError {
    /// When decoding a transaction request from RLP
    #[error(transparent)]
    DecodingError(#[from] rlp::DecoderError),
    /// When recovering the address from a signature
    #[error(transparent)]
    RecoveryError(#[from] SignatureError),
}

/// An EIP-2930 transaction is a legacy transaction including an [`AccessList`].
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Eip2930TransactionRequest {
    #[serde(flatten)]
    pub tx: TransactionRequest,
    #[serde(rename = "accessList")]
    pub access_list: AccessList,
}

impl Eip2930TransactionRequest {
    pub fn new(tx: TransactionRequest, access_list: AccessList) -> Self {
        Self { tx, access_list }
    }

    pub fn rlp(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_EIP2930_FIELDS);

        let chain_id = self.tx.chain_id.unwrap_or_else(U64::one);
        rlp.append(&chain_id);
        self.tx.rlp_base(&mut rlp);
        // append the access list in addition to the base rlp encoding
        rlp.append(&self.access_list);

        rlp.out().freeze().into()
    }

    /// Produces the RLP encoding of the transaction with the provided signature
    pub fn rlp_signed(&self, signature: &Signature) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_EIP2930_FIELDS + 3);

        let chain_id = self.tx.chain_id.unwrap_or_else(U64::one);
        rlp.append(&chain_id);
        self.tx.rlp_base(&mut rlp);
        // append the access list in addition to the base rlp encoding
        rlp.append(&self.access_list);

        // append the signature
        let v = normalize_v(signature.v, chain_id);
        rlp.append(&v);
        rlp.append(&signature.r);
        rlp.append(&signature.s);
        rlp.out().freeze().into()
    }

    /// Decodes fields based on the RLP offset passed.
    fn decode_base_rlp(rlp: &rlp::Rlp, offset: &mut usize) -> Result<Self, rlp::DecoderError> {
        let chain_id: u64 = rlp.val_at(*offset)?;
        *offset += 1;

        let mut request = TransactionRequest::decode_unsigned_rlp_base(rlp, offset)?;
        request.chain_id = Some(U64::from(chain_id));

        let al = rlp::Rlp::new(rlp.at(*offset)?.as_raw()).data()?;
        let access_list = match al.len() {
            0 => AccessList(vec![]),
            _ => rlp.val_at(*offset)?,
        };
        *offset += 1;

        Ok(Self { tx: request, access_list })
    }

    /// Decodes the given RLP into a transaction, attempting to decode its signature as well.
    pub fn decode_signed_rlp(rlp: &rlp::Rlp) -> Result<(Self, Signature), Eip2930RequestError> {
        let mut offset = 0;
        let mut txn = Self::decode_base_rlp(rlp, &mut offset)?;

        let v = rlp.val_at(offset)?;
        offset += 1;
        let r = rlp.val_at(offset)?;
        offset += 1;
        let s = rlp.val_at(offset)?;

        let sig = Signature { r, s, v };
        txn.tx.from = Some(sig.recover(TypedTransaction::Eip2930(txn.clone()).sighash())?);
        Ok((txn, sig))
    }
}

/// Get a Eip2930TransactionRequest from a rlp encoded byte stream
impl Decodable for Eip2930TransactionRequest {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        Self::decode_base_rlp(rlp, &mut 0)
    }
}

impl From<&Transaction> for Eip2930TransactionRequest {
    fn from(tx: &Transaction) -> Eip2930TransactionRequest {
        Eip2930TransactionRequest {
            tx: tx.into(),
            access_list: tx.access_list.clone().unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::types::{transaction::eip2718::TypedTransaction, U256};
    use std::str::FromStr;

    #[test]
    #[cfg_attr(feature = "celo", ignore)]
    // https://github.com/ethereum/go-ethereum/blob/c503f98f6d5e80e079c1d8a3601d188af2a899da/core/types/transaction_test.go#L59-L67
    fn rlp() {
        let tx: TypedTransaction = TransactionRequest::new()
            .nonce(3)
            .gas_price(1)
            .gas(25000)
            .to("b94f5374fce5edbc8e2a8697c15331677e6ebf0b".parse::<Address>().unwrap())
            .value(10)
            .data(vec![0x55, 0x44])
            .with_access_list(vec![])
            .into();

        let hash = tx.sighash();
        let sig: Signature = "c9519f4f2b30335884581971573fadf60c6204f59a911df35ee8a540456b266032f1e8e2c5dd761f9e4f88f41c8310aeaba26a8bfcdacfedfa12ec3862d3752101".parse().unwrap();
        assert_eq!(
            hash,
            "49b486f0ec0a60dfbbca2d30cb07c9e8ffb2a2ff41f29a1ab6737475f6ff69f3".parse().unwrap()
        );

        let enc = rlp::encode(&tx.rlp_signed(&sig).as_ref());
        let expected = "b86601f8630103018261a894b94f5374fce5edbc8e2a8697c15331677e6ebf0b0a825544c001a0c9519f4f2b30335884581971573fadf60c6204f59a911df35ee8a540456b2660a032f1e8e2c5dd761f9e4f88f41c8310aeaba26a8bfcdacfedfa12ec3862d37521";
        assert_eq!(hex::encode(&enc), expected);
    }

    #[test]
    #[cfg_attr(feature = "legacy", ignore)]
    fn serde_eip2930_tx() {
        let access_list =
            vec![AccessListItem { address: Address::zero(), storage_keys: vec![H256::zero()] }];
        let tx = TransactionRequest::new()
            .to(Address::zero())
            .value(U256::from(100))
            .with_access_list(access_list);
        let tx: TypedTransaction = tx.into();
        let serialized = serde_json::to_string(&tx).unwrap();

        // deserializes to either the envelope type or the inner type
        let de: TypedTransaction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tx, de);

        let de: Eip2930TransactionRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tx, TypedTransaction::Eip2930(de));
    }

    #[test]
    #[cfg_attr(feature = "celo", ignore)]
    fn decoding_eip2930_signed() {
        let raw_tx = hex::decode("01f901ef018209068508d8f9fc0083124f8094f5b4f13bdbe12709bd3ea280ebf4b936e99b20f280b90184c5d404940000000000000000000000000000000000000000000000000c4d67a76e15d8190000000000000000000000000000000000000000000000000029d9d8fb7440000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000020000000000000000000000007b73644935b8e68019ac6356c40661e1bc315860000000000000000000000000761d38e5ddf6ccf6cf7c55759d5210750b5d60f30000000000000000000000000000000000000000000000000000000000000000000000000000000000000000381fe4eb128db1621647ca00965da3f9e09f4fac000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000000000000000000000000000000000000000000ac001a0881e7f5298290794bcaa0294986db5c375cbf135dd3c21456b159c470568b687a061fc5f52abab723053fbedf29e1c60b89006416d6c86e1c54ef85a3e84f2dc6e").unwrap();
        let expected_tx = TransactionRequest::new()
            .chain_id(1u64)
            .nonce(2310u64)
            .gas_price(38_000_000_000u64)
            .gas(1_200_000u64)
            .to(Address::from_str("0xf5b4f13bdbe12709bd3ea280ebf4b936e99b20f2").unwrap())
            .value(0u64)
            .data(hex::decode("c5d404940000000000000000000000000000000000000000000000000c4d67a76e15d8190000000000000000000000000000000000000000000000000029d9d8fb7440000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000020000000000000000000000007b73644935b8e68019ac6356c40661e1bc315860000000000000000000000000761d38e5ddf6ccf6cf7c55759d5210750b5d60f30000000000000000000000000000000000000000000000000000000000000000000000000000000000000000381fe4eb128db1621647ca00965da3f9e09f4fac000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000000000000000000000000000000000000000000a").unwrap())
            .from(Address::from_str("0x82a33964706683db62b85a59128ce2fc07c91658").unwrap())
            .with_access_list(AccessList(vec![]));
        let r =
            U256::from_str("0x881e7f5298290794bcaa0294986db5c375cbf135dd3c21456b159c470568b687")
                .unwrap();
        let s =
            U256::from_str("0x61fc5f52abab723053fbedf29e1c60b89006416d6c86e1c54ef85a3e84f2dc6e")
                .unwrap();
        let v = 1;
        let expected_sig = Signature { r, s, v };

        let raw_tx_rlp = rlp::Rlp::new(&raw_tx[..]);

        let (real_tx, real_sig) = TypedTransaction::decode_signed(&raw_tx_rlp).unwrap();
        let real_tx = match real_tx {
            TypedTransaction::Eip2930(tx) => tx,
            _ => panic!("The raw bytes should decode to an EIP2930 tranaction"),
        };

        assert_eq!(expected_tx, real_tx);
        assert_eq!(expected_sig, real_sig);
    }

    #[test]
    #[cfg_attr(feature = "celo", ignore)]
    fn decoding_eip2930_with_access_list() {
        let raw_tx = hex::decode("01f90126018223ff850a02ffee00830f4240940000000000a8fb09af944ab3baf7a9b3e1ab29d880b876200200001525000000000b69ffb300000000557b933a7c2c45672b610f8954a3deb39a51a8cae53ec727dbdeb9e2d5456c3be40cff031ab40a55724d5c9c618a2152e99a45649a3b8cf198321f46720b722f4ec38f99ba3bb1303258d2e816e6a95b25647e01bd0967c1b9599fa3521939871d1d0888f845d694724d5c9c618a2152e99a45649a3b8cf198321f46c0d694720b722f4ec38f99ba3bb1303258d2e816e6a95bc0d69425647e01bd0967c1b9599fa3521939871d1d0888c001a08323efae7b9993bd31a58da7924359d24b5504aa2b33194fcc5ae206e65d2e62a054ce201e3b4b5cd38eb17c56ee2f9111b2e164efcd57b3e70fa308a0a51f7014").unwrap();
        let expected_tx = TransactionRequest::new()
            .chain_id(1u64)
            .nonce(9215u64)
            .gas_price(43_000_000_000u64)
            .gas(1_000_000)
            .to(Address::from_str("0x0000000000a8fb09af944ab3baf7a9b3e1ab29d8").unwrap())
            .value(0)
            .data(Bytes::from_str("0x200200001525000000000b69ffb300000000557b933a7c2c45672b610f8954a3deb39a51a8cae53ec727dbdeb9e2d5456c3be40cff031ab40a55724d5c9c618a2152e99a45649a3b8cf198321f46720b722f4ec38f99ba3bb1303258d2e816e6a95b25647e01bd0967c1b9599fa3521939871d1d0888").unwrap())
            .from(Address::from_str("0xe9c790e8fde820ded558a4771b72eec916c04763").unwrap())
            .with_access_list(AccessList(vec![
                AccessListItem {
                    address: Address::from_str("0x724d5c9c618a2152e99a45649a3b8cf198321f46").unwrap(),
                    storage_keys: vec![],
                },
                AccessListItem {
                    address: Address::from_str("0x720b722f4ec38f99ba3bb1303258d2e816e6a95b").unwrap(),
                    storage_keys: vec![],
                },
                AccessListItem {
                    address: Address::from_str("0x25647e01bd0967c1b9599fa3521939871d1d0888").unwrap(),
                    storage_keys: vec![],
                },
            ]));
        let expected_sig = Signature {
            r: "0x8323efae7b9993bd31a58da7924359d24b5504aa2b33194fcc5ae206e65d2e62".into(),
            s: "0x54ce201e3b4b5cd38eb17c56ee2f9111b2e164efcd57b3e70fa308a0a51f7014".into(),
            v: 1u64,
        };

        let raw_tx_rlp = rlp::Rlp::new(&raw_tx[..]);

        let (real_tx, real_sig) = TypedTransaction::decode_signed(&raw_tx_rlp).unwrap();
        let real_tx = match real_tx {
            TypedTransaction::Eip2930(tx) => tx,
            _ => panic!("The raw bytes should decode to an EIP2930 tranaction"),
        };

        assert_eq!(expected_tx, real_tx);
        assert_eq!(expected_sig, real_sig);
    }
}
