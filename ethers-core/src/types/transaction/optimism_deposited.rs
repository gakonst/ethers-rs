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

        let source_hash: Option<H256> = rlp.val_at(*offset)?;
        *offset += 1;
        tx.from = rlp.val_at(*offset)?;
        *offset += 1;
        tx.to = rlp.val_at(*offset)?;
        *offset += 1;
        let mint: Option<U256> = rlp.val_at(*offset)?;
        *offset += 1;
        tx.value = rlp.val_at(*offset)?;
        *offset += 1;
        tx.gas = rlp.val_at(*offset)?;
        *offset += 1;
        let is_system_tx: Option<bool> = rlp.val_at(*offset)?;
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
        let mut txn = Self::decode_base_rlp(rlp, &mut offset)?;
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
