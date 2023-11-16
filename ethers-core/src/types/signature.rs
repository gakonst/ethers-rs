// Code adapted from: https://github.com/tomusdrw/rust-web3/blob/master/src/api/accounts.rs
use crate::{
    types::{Address, H256, U256},
    utils::hash_message,
};
use elliptic_curve::{consts::U32, sec1::ToEncodedPoint};
use generic_array::GenericArray;
use k256::{
    ecdsa::{
        Error as K256SignatureError, RecoveryId, Signature as RecoverableSignature,
        Signature as K256Signature, VerifyingKey,
    },
    PublicKey as K256PublicKey,
};
use open_fastrlp::Decodable;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, str::FromStr};
use thiserror::Error;

/// An error involving a signature.
#[derive(Debug, Error)]
pub enum SignatureError {
    /// Invalid length, secp256k1 signatures are 65 bytes
    #[error("invalid signature length, got {0}, expected 65")]
    InvalidLength(usize),
    /// When parsing a signature from string to hex
    #[error(transparent)]
    DecodingError(#[from] hex::FromHexError),
    /// Thrown when signature verification failed (i.e. when the address that
    /// produced the signature did not match the expected address)
    #[error("Signature verification failed. Expected {0}, got {1}")]
    VerificationError(Address, Address),
    /// Internal error during signature recovery
    #[error(transparent)]
    K256Error(#[from] K256SignatureError),
    /// Error in recovering public key from signature
    #[error("Public key recovery error")]
    RecoveryError,
}

/// Recovery message data.
///
/// The message data can either be a binary message that is first hashed
/// according to EIP-191 and then recovered based on the signature or a
/// precomputed hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryMessage {
    /// Message bytes
    Data(Vec<u8>),
    /// Message hash
    Hash(H256),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy, Hash)]
/// An ECDSA signature
pub struct Signature {
    /// R value
    pub r: U256,
    /// S Value
    pub s: U256,
    /// V value
    pub v: u64,
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(hex::Buffer::<65, false>::new().format(&self.into()))
    }
}

impl Signature {
    /// Verifies that signature on `message` was produced by `address`
    pub fn verify<M, A>(&self, message: M, address: A) -> Result<(), SignatureError>
    where
        M: Into<RecoveryMessage>,
        A: Into<Address>,
    {
        let address = address.into();
        let recovered = self.recover(message)?;
        if recovered != address {
            return Err(SignatureError::VerificationError(address, recovered))
        }

        Ok(())
    }

    /// Recovers the Ethereum address which was used to sign the given message.
    ///
    /// Recovery signature data uses 'Electrum' notation, this means the `v`
    /// value is expected to be either `27` or `28`.
    pub fn recover<M>(&self, message: M) -> Result<Address, SignatureError>
    where
        M: Into<RecoveryMessage>,
    {
        let message_hash = match message.into() {
            RecoveryMessage::Data(ref message) => hash_message(message),
            RecoveryMessage::Hash(hash) => hash,
        };

        let (recoverable_sig, recovery_id) = self.as_signature()?;
        let verifying_key = VerifyingKey::recover_from_prehash(
            message_hash.as_ref(),
            &recoverable_sig,
            recovery_id,
        )?;

        let public_key = K256PublicKey::from(&verifying_key);
        let public_key = public_key.to_encoded_point(/* compress = */ false);
        let public_key = public_key.as_bytes();
        debug_assert_eq!(public_key[0], 0x04);
        let hash = crate::utils::keccak256(&public_key[1..]);
        Ok(Address::from_slice(&hash[12..]))
    }

    /// Recovers the ethereum address which was used to sign a given EIP712
    /// typed data payload.
    ///
    /// Recovery signature data uses 'Electrum' notation, this means the `v`
    /// value is expected to be either `27` or `28`.
    pub fn recover_typed_data<T>(&self, payload: &T) -> Result<Address, SignatureError>
    where
        T: super::transaction::eip712::Eip712,
    {
        let encoded = payload.encode_eip712().map_err(|_| SignatureError::RecoveryError)?;
        self.recover(encoded)
    }

    /// Retrieves the recovery signature.
    fn as_signature(&self) -> Result<(RecoverableSignature, RecoveryId), SignatureError> {
        let recovery_id = self.recovery_id()?;
        let signature = {
            let mut r_bytes = [0u8; 32];
            let mut s_bytes = [0u8; 32];
            self.r.to_big_endian(&mut r_bytes);
            self.s.to_big_endian(&mut s_bytes);
            let gar: &GenericArray<u8, U32> = GenericArray::from_slice(&r_bytes);
            let gas: &GenericArray<u8, U32> = GenericArray::from_slice(&s_bytes);
            K256Signature::from_scalars(*gar, *gas)?
        };

        Ok((signature, recovery_id))
    }

    /// Retrieve the recovery ID.
    pub fn recovery_id(&self) -> Result<RecoveryId, SignatureError> {
        let standard_v = normalize_recovery_id(self.v);
        Ok(RecoveryId::from_byte(standard_v).expect("normalized recovery id always valid"))
    }

    /// Copies and serializes `self` into a new `Vec` with the recovery id included
    #[allow(clippy::wrong_self_convention)]
    pub fn to_vec(&self) -> Vec<u8> {
        self.into()
    }

    /// Decodes a signature from RLP bytes, assuming no RLP header
    pub(crate) fn decode_signature(buf: &mut &[u8]) -> Result<Self, open_fastrlp::DecodeError> {
        let v = u64::decode(buf)?;
        Ok(Self { r: U256::decode(buf)?, s: U256::decode(buf)?, v })
    }
}

impl open_fastrlp::Decodable for Signature {
    fn decode(buf: &mut &[u8]) -> Result<Self, open_fastrlp::DecodeError> {
        Self::decode_signature(buf)
    }
}

impl open_fastrlp::Encodable for Signature {
    fn length(&self) -> usize {
        self.r.length() + self.s.length() + self.v.length()
    }
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        self.v.encode(out);
        self.r.encode(out);
        self.s.encode(out);
    }
}

/// Normalize a V value, respecting raw, legacy, and EIP-155 values.
///
/// This function covers the entire u64 range, producing v-values as follows:
/// - 0-26 - raw/bare. 0-3 are legal. In order to ensure that all values are covered, we also handle
///   4-26 here by returning v % 4.
/// - 27-34 - legacy. 27-30 are legal. By legacy bitcoin convention range 27-30 signals uncompressed
///   pubkeys, while 31-34 signals compressed pubkeys. We do not respect the compression convention.
///   All Ethereum keys are uncompressed.
/// - 35+ - EIP-155. By EIP-155 convention, `v = 35 + CHAIN_ID * 2 + 0/1` We return (v-1 % 2) here.
///
/// NB: raw and legacy support values 2, and 3, while EIP-155 does not.
/// Recovery values of 2 and 3 are unlikely to occur in practice. In the vanishingly unlikely event
/// that you encounter an EIP-155 signature with a recovery value of 2 or 3, you should normalize
/// out of band.
fn normalize_recovery_id(v: u64) -> u8 {
    match v {
        // Case 0: raw/bare
        v @ 0..=26 => (v % 4) as u8,
        // Case 2: non-eip155 v value
        v @ 27..=34 => ((v - 27) % 4) as u8,
        // Case 3: eip155 V value
        v @ 35.. => ((v - 1) % 2) as u8,
    }
}

impl<'a> TryFrom<&'a [u8]> for Signature {
    type Error = SignatureError;

    /// Parses a raw signature which is expected to be 65 bytes long where
    /// the first 32 bytes is the `r` value, the second 32 bytes the `s` value
    /// and the final byte is the `v` value in 'Electrum' notation.
    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 65 {
            return Err(SignatureError::InvalidLength(bytes.len()))
        }

        let v = bytes[64];
        let r = U256::from_big_endian(&bytes[0..32]);
        let s = U256::from_big_endian(&bytes[32..64]);

        Ok(Signature { r, s, v: v.into() })
    }
}

impl FromStr for Signature {
    type Err = SignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Signature::try_from(&hex::decode(s)?[..])
    }
}

impl From<&Signature> for [u8; 65] {
    fn from(src: &Signature) -> [u8; 65] {
        let mut sig = [0u8; 65];
        let mut r_bytes = [0u8; 32];
        let mut s_bytes = [0u8; 32];
        src.r.to_big_endian(&mut r_bytes);
        src.s.to_big_endian(&mut s_bytes);
        sig[..32].copy_from_slice(&r_bytes);
        sig[32..64].copy_from_slice(&s_bytes);
        // TODO: What if we try to serialize a signature where
        // the `v` is not normalized?

        // The u64 to u8 cast is safe because `sig.v` can only ever be 27 or 28
        // here. Regarding EIP-155, the modification to `v` happens during tx
        // creation only _after_ the transaction is signed using
        // `ethers_signers::to_eip155_v`.
        sig[64] = src.v as u8;
        sig
    }
}

impl From<Signature> for [u8; 65] {
    fn from(src: Signature) -> [u8; 65] {
        <[u8; 65]>::from(&src)
    }
}

impl From<&Signature> for Vec<u8> {
    fn from(src: &Signature) -> Vec<u8> {
        <[u8; 65]>::from(src).to_vec()
    }
}

impl From<Signature> for Vec<u8> {
    fn from(src: Signature) -> Vec<u8> {
        <[u8; 65]>::from(&src).to_vec()
    }
}

impl From<&[u8]> for RecoveryMessage {
    fn from(s: &[u8]) -> Self {
        s.to_owned().into()
    }
}

impl From<Vec<u8>> for RecoveryMessage {
    fn from(s: Vec<u8>) -> Self {
        RecoveryMessage::Data(s)
    }
}

impl From<&str> for RecoveryMessage {
    fn from(s: &str) -> Self {
        s.as_bytes().to_owned().into()
    }
}

impl From<String> for RecoveryMessage {
    fn from(s: String) -> Self {
        RecoveryMessage::Data(s.into_bytes())
    }
}

impl From<[u8; 32]> for RecoveryMessage {
    fn from(hash: [u8; 32]) -> Self {
        H256(hash).into()
    }
}

impl From<H256> for RecoveryMessage {
    fn from(hash: H256) -> Self {
        RecoveryMessage::Hash(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Transaction;

    #[test]
    fn can_recover_tx_sender() {
        // random mainnet tx: https://etherscan.io/tx/0x86718885c4b4218c6af87d3d0b0d83e3cc465df2a05c048aa4db9f1a6f9de91f
        let tx_rlp = hex::decode("02f872018307910d808507204d2cb1827d0094388c818ca8b9251b393131c08a736a67ccb19297880320d04823e2701c80c001a0cf024f4815304df2867a1a74e9d2707b6abda0337d2d54a4438d453f4160f190a07ac0e6b3bc9395b5b9c8b9e6d77204a236577a5b18467b9175c01de4faa208d9").unwrap();
        let tx: Transaction = rlp::decode(&tx_rlp).unwrap();
        assert_eq!(tx.rlp(), tx_rlp);
        assert_eq!(
            tx.hash,
            "0x86718885c4b4218c6af87d3d0b0d83e3cc465df2a05c048aa4db9f1a6f9de91f".parse().unwrap()
        );
        assert_eq!(tx.transaction_type, Some(2.into()));
        let expected = Address::from_str("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5").unwrap();
        assert_eq!(tx.recover_from().unwrap(), expected);
    }

    #[test]
    fn recover_web3_signature() {
        // test vector taken from:
        // https://web3js.readthedocs.io/en/v1.2.2/web3-eth-accounts.html#sign
        let signature = Signature::from_str(
            "b91467e570a6466aa9e9876cbcd013baba02900b8979d43fe208a4a4f339f5fd6007e74cd82e037b800186422fc2da167c747ef045e5d18a5f5d4300f8e1a0291c"
        ).expect("could not parse signature");
        assert_eq!(
            signature.recover("Some data").unwrap(),
            Address::from_str("2c7536E3605D9C16a7a3D7b1898e529396a65c23").unwrap()
        );
    }

    #[test]
    fn signature_from_str() {
        let s1 = Signature::from_str(
            "0xaa231fbe0ed2b5418e6ba7c19bee2522852955ec50996c02a2fe3e71d30ddaf1645baf4823fea7cb4fcc7150842493847cfb6a6d63ab93e8ee928ee3f61f503500"
        ).expect("could not parse 0x-prefixed signature");

        let s2 = Signature::from_str(
            "aa231fbe0ed2b5418e6ba7c19bee2522852955ec50996c02a2fe3e71d30ddaf1645baf4823fea7cb4fcc7150842493847cfb6a6d63ab93e8ee928ee3f61f503500"
        ).expect("could not parse non-prefixed signature");

        assert_eq!(s1, s2);
    }
}
