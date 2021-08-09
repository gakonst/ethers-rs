use super::{normalize_v, request::TransactionRequest};
use crate::types::{Address, Bytes, Signature, H256, U256, U64};

use rlp::RlpStream;
use rlp_derive::{RlpEncodable, RlpEncodableWrapper};
use serde::{Deserialize, Serialize};

const NUM_EIP2930_FIELDS: usize = 8;

/// Access list
// NB: Need to use `RlpEncodableWrapper` else we get an extra [] in the output
// https://github.com/gakonst/ethers-rs/pull/353#discussion_r680683869
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, RlpEncodableWrapper)]
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
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, RlpEncodable)]
#[serde(rename_all = "camelCase")]
pub struct AccessListItem {
    /// Accessed address
    pub address: Address,
    /// Accessed storage keys
    pub storage_keys: Vec<H256>,
}

/// An EIP-2930 transaction is a legacy transaction including an [`AccessList`].
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Eip2930TransactionRequest {
    #[serde(flatten)]
    pub tx: TransactionRequest,
    pub access_list: AccessList,
}

impl Eip2930TransactionRequest {
    pub fn new(tx: TransactionRequest, access_list: AccessList) -> Self {
        Self { tx, access_list }
    }

    pub fn rlp<T: Into<U64>>(&self, chain_id: T) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_EIP2930_FIELDS);
        rlp.append(&chain_id.into());
        self.tx.rlp_base(&mut rlp);
        // append the access list in addition to the base rlp encoding
        rlp.append(&self.access_list);

        rlp.out().freeze().into()
    }

    /// Produces the RLP encoding of the transaction with the provided signature
    pub fn rlp_signed<T: Into<U64>>(&self, chain_id: T, signature: &Signature) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_EIP2930_FIELDS + 3);

        let chain_id = chain_id.into();
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{transaction::eip2718::TypedTransaction, U256};

    #[test]
    #[cfg_attr(feature = "celo", ignore)]
    // https://github.com/ethereum/go-ethereum/blob/c503f98f6d5e80e079c1d8a3601d188af2a899da/core/types/transaction_test.go#L59-L67
    fn rlp() {
        let tx: TypedTransaction = TransactionRequest::new()
            .nonce(3)
            .gas_price(1)
            .gas(25000)
            .to("b94f5374fce5edbc8e2a8697c15331677e6ebf0b"
                .parse::<Address>()
                .unwrap())
            .value(10)
            .data(vec![0x55, 0x44])
            .with_access_list(vec![])
            .into();

        let hash = tx.sighash(1);
        let sig: Signature = "c9519f4f2b30335884581971573fadf60c6204f59a911df35ee8a540456b266032f1e8e2c5dd761f9e4f88f41c8310aeaba26a8bfcdacfedfa12ec3862d3752101".parse().unwrap();
        assert_eq!(
            hash,
            "49b486f0ec0a60dfbbca2d30cb07c9e8ffb2a2ff41f29a1ab6737475f6ff69f3"
                .parse()
                .unwrap()
        );

        let enc = rlp::encode(&tx.rlp_signed(1, &sig).as_ref());
        let expected = "b86601f8630103018261a894b94f5374fce5edbc8e2a8697c15331677e6ebf0b0a825544c001a0c9519f4f2b30335884581971573fadf60c6204f59a911df35ee8a540456b2660a032f1e8e2c5dd761f9e4f88f41c8310aeaba26a8bfcdacfedfa12ec3862d37521";
        assert_eq!(hex::encode(enc.to_vec()), expected);
    }

    #[test]
    fn serde_eip2930_tx() {
        let access_list = vec![AccessListItem {
            address: Address::zero(),
            storage_keys: vec![H256::zero()],
        }];
        let tx = TransactionRequest::new()
            .to(Address::zero())
            .value(U256::from(100))
            .with_access_list(access_list);
        let tx: TypedTransaction = tx.into();
        let serialized = serde_json::to_string(&tx).unwrap();
        dbg!(&serialized);

        // deserializes to either the envelope type or the inner type
        let de: TypedTransaction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tx, de);

        let de: Eip2930TransactionRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tx, TypedTransaction::Eip2930(de));
    }
}
