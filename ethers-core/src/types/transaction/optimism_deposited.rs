use super::rlp_opt;
use crate::types::{Bytes, Signature, Transaction, TransactionRequest, H256, U256};
use rlp::{Decodable, RlpStream};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const NUM_TX_FIELDS: usize = 8;

/// An error involving an OptimismDeposited transaction request.
#[derive(Debug, Error)]
pub enum OptimismDepositedRequestError {
    /// When decoding a transaction request from RLP
    #[error(transparent)]
    DecodingError(#[from] rlp::DecoderError),
}

/// Parameters for sending a transaction
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct OptimismDepositedTransactionRequest {
    #[serde(flatten)]
    pub tx: TransactionRequest,

    /// The source hash which uniquely identifies the origin of the deposit
    #[serde(rename = "sourceHash", skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<H256>,

    /// The ETH value to mint on L2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mint: Option<U256>,

    /// If true, the transaction does not interact with the L2 block gas pool.
    /// Note: boolean is disabled (enforced to be false) starting from the Regolith upgrade.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_system_tx: Option<bool>,
}

impl OptimismDepositedTransactionRequest {
    pub fn new(
        tx: TransactionRequest,
        source_hash: Option<H256>,
        mint: Option<U256>,
        is_system_tx: Option<bool>,
    ) -> Self {
        Self { tx, source_hash, mint, is_system_tx }
    }

    pub fn rlp(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_TX_FIELDS);

        rlp_opt(&mut rlp, &self.source_hash);
        rlp.append(&self.tx.from);
        rlp_opt(&mut rlp, &self.tx.to);
        rlp_opt(&mut rlp, &self.mint);
        rlp.append(&self.tx.value);
        rlp.append(&self.tx.gas);
        rlp_opt(&mut rlp, &self.is_system_tx);
        rlp_opt(&mut rlp, &self.tx.data.as_deref());

        rlp.out().freeze().into()
    }

    /// Decodes fields based on the RLP offset passed.
    fn decode_base_rlp(rlp: &rlp::Rlp, offset: &mut usize) -> Result<Self, rlp::DecoderError> {
        let mut tx = TransactionRequest::new();

        let source_hash: Option<H256> = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.from = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.to = Some(rlp.val_at(*offset)?);
        *offset += 1;
        let mint: Option<U256> = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.value = Some(rlp.val_at(*offset)?);
        *offset += 1;
        tx.gas = Some(rlp.val_at(*offset)?);
        *offset += 1;
        let is_system_tx: Option<bool> = Some(rlp.val_at(*offset)?);
        *offset += 1;
        let data = rlp::Rlp::new(rlp.at(*offset)?.as_raw()).data()?;
        tx.data = match data.len() {
            0 => None,
            _ => Some(Bytes::from(data.to_vec())),
        };
        *offset += 1;

        Ok(Self { tx, source_hash, mint, is_system_tx })
    }

    /// Decodes the given RLP into a transaction
    /// Note: this transaction does not have a signature
    pub fn decode_signed_rlp(
        rlp: &rlp::Rlp,
    ) -> Result<(Self, Signature), OptimismDepositedRequestError> {
        let mut offset = 0;
        let txn = Self::decode_base_rlp(rlp, &mut offset)?;
        let sig = Signature { r: 0.into(), s: 0.into(), v: 0 };

        Ok((txn, sig))
    }
}

/// Get a Eip2930TransactionRequest from a rlp encoded byte stream
impl Decodable for OptimismDepositedTransactionRequest {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        Self::decode_base_rlp(rlp, &mut 0)
    }
}

/// Get an OptimismDeposited transaction request from a Transaction
impl From<&Transaction> for OptimismDepositedTransactionRequest {
    fn from(tx: &Transaction) -> OptimismDepositedTransactionRequest {
        OptimismDepositedTransactionRequest {
            tx: tx.into(),
            source_hash: tx.source_hash,
            mint: tx.mint,
            is_system_tx: tx.is_system_tx,
        }
    }
}

#[cfg(feature = "optimism")]
#[cfg(not(feature = "celo"))]
#[cfg(test)]
mod test {

    use crate::types::{
        transaction::eip2718::TypedTransaction, Address, Bytes, NameOrAddress, Transaction, H256,
        U256, U64,
    };
    use rlp::Decodable;
    use std::str::FromStr;

    #[test]
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
            transaction_type: Some(U64::from(0x7E)),
            access_list: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            chain_id: None,
        };

        let rlp = deposited_tx.rlp();

        let expected_rlp = Bytes::from(hex::decode("7ef90159a0a8157ccf61bcdfbcb74a84ec1262e62644dd1e7e3614abcbd8db0c99a60049fc94deaddeaddeaddeaddeaddeaddeaddeaddead00019442000000000000000000000000000000000000158080830f424080b90104015d8eb90000000000000000000000000000000000000000000000000000000000878c1c00000000000000000000000000000000000000000000000000000000644662bc0000000000000000000000000000000000000000000000000000001ee24fba17b7e19cc10812911dfa8a438e0a81a9933f843aa5b528899b8d9e221b649ae0df00000000000000000000000000000000000000000000000000000000000000060000000000000000000000007431310e026b69bfc676c0013e12a1a11411eec9000000000000000000000000000000000000000000000000000000000000083400000000000000000000000000000000000000000000000000000000000f4240").unwrap());

        assert_eq!(rlp, expected_rlp);
    }

    #[test]
    fn test_rlp_decode_optimism_tx() {
        let encoded = Bytes::from(hex::decode("7ef90159a0a8157ccf61bcdfbcb74a84ec1262e62644dd1e7e3614abcbd8db0c99a60049fc94deaddeaddeaddeaddeaddeaddeaddeaddead00019442000000000000000000000000000000000000158080830f424080b90104015d8eb90000000000000000000000000000000000000000000000000000000000878c1c00000000000000000000000000000000000000000000000000000000644662bc0000000000000000000000000000000000000000000000000000001ee24fba17b7e19cc10812911dfa8a438e0a81a9933f843aa5b528899b8d9e221b649ae0df00000000000000000000000000000000000000000000000000000000000000060000000000000000000000007431310e026b69bfc676c0013e12a1a11411eec9000000000000000000000000000000000000000000000000000000000000083400000000000000000000000000000000000000000000000000000000000f4240").unwrap());
        let tx = TypedTransaction::decode(&rlp::Rlp::new(&encoded)).unwrap();

        assert!(matches!(tx, TypedTransaction::OptimismDeposited(_)));

        assert_eq!(tx.gas(), Some(&U256::from(1000000u64)));
        assert_eq!(tx.gas_price(), None);
        assert_eq!(tx.value(), Some(&U256::zero()));
        assert_eq!(tx.nonce(), None);
        assert_eq!(
            tx.to(),
            Some(&NameOrAddress::Address(
                Address::from_str("0x4200000000000000000000000000000000000015").unwrap()
            ))
        );
    }
}
