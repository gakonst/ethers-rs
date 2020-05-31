// Code adapted from: https://github.com/tomusdrw/rust-web3/blob/master/src/api/accounts.rs
use crate::{utils::hash_message, Address, PublicKey, H256};

use rustc_hex::ToHex;
use secp256k1::{
    recovery::{RecoverableSignature, RecoveryId},
    Error as Secp256k1Error, Message, Secp256k1,
};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};
use thiserror::Error;

/// An error involving a signature.
#[derive(Clone, Debug, Error)]
pub enum SignatureError {
    /// Internal error inside the recovery
    #[error(transparent)]
    Secp256k1Error(#[from] Secp256k1Error),
    /// Invalid length, secp256k1 signatures are 65 bytes
    #[error("invalid signature length, got {0}, expected 65")]
    InvalidLength(usize),
}

/// Recovery message data.
///
/// The message data can either be a binary message that is first hashed
/// according to EIP-191 and then recovered based on the signature or a
/// precomputed hash.
#[derive(Clone, Debug, PartialEq)]
pub enum RecoveryMessage {
    /// Message bytes
    Data(Vec<u8>),
    /// Message hash
    Hash(H256),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// An ECDSA signature
pub struct Signature {
    /// R value
    pub r: H256,
    /// S Value
    pub s: H256,
    /// V value in 'Electrum' notation.
    pub v: u8,
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sig = <[u8; 65]>::from(self);
        write!(f, "{}", sig.to_hex::<String>())
    }
}

impl Signature {
    /// Recovers the Ethereum address which was used to sign the given message.
    ///
    /// Recovery signature data uses 'Electrum' notation, this means the `v`
    /// value is expected to be either `27` or `28`.
    pub fn recover<M>(&self, message: M) -> Result<Address, SignatureError>
    where
        M: Into<RecoveryMessage>,
    {
        let message = message.into();
        let message_hash = match message {
            RecoveryMessage::Data(ref message) => hash_message(message),
            RecoveryMessage::Hash(hash) => hash,
        };
        let signature = self.as_signature()?;

        let message = Message::from_slice(message_hash.as_bytes())?;
        let public_key = Secp256k1::verification_only().recover(&message, &signature)?;

        Ok(PublicKey::from(public_key).into())
    }

    /// Retrieves the recovery signature.
    fn as_signature(&self) -> Result<RecoverableSignature, SignatureError> {
        let recovery_id = self.recovery_id()?;
        let signature = {
            let mut sig = [0u8; 64];
            sig[..32].copy_from_slice(self.r.as_bytes());
            sig[32..].copy_from_slice(self.s.as_bytes());
            sig
        };

        Ok(RecoverableSignature::from_compact(&signature, recovery_id)?)
    }

    /// Retrieve the recovery ID.
    fn recovery_id(&self) -> Result<RecoveryId, SignatureError> {
        let standard_v = match self.v {
            27 => 0,
            28 => 1,
            v if v >= 35 => ((v - 1) % 2) as _,
            _ => 4,
        };

        Ok(RecoveryId::from_i32(standard_v)?)
    }

    /// Copies and serializes `self` into a new `Vec` with the recovery id included
    pub fn to_vec(&self) -> Vec<u8> {
        self.into()
    }
}

impl<'a> TryFrom<&'a [u8]> for Signature {
    type Error = SignatureError;

    /// Parses a raw signature which is expected to be 65 bytes long where
    /// the first 32 bytes is the `r` value, the second 32 bytes the `s` value
    /// and the final byte is the `v` value in 'Electrum' notation.
    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 65 {
            return Err(SignatureError::InvalidLength(bytes.len()));
        }

        let v = bytes[64];
        let r = H256::from_slice(&bytes[0..32]);
        let s = H256::from_slice(&bytes[32..64]);

        Ok(Signature { r, s, v })
    }
}

impl From<&Signature> for [u8; 65] {
    fn from(src: &Signature) -> [u8; 65] {
        let mut sig = [0u8; 65];
        sig[..32].copy_from_slice(src.r.as_bytes());
        sig[32..64].copy_from_slice(src.s.as_bytes());
        sig[64] = src.v;
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
    use crate::PrivateKey;

    #[test]
    fn recover_signature_from_message() {
        let message = "Some data";
        let hash = hash_message(message);
        let key = PrivateKey::new(&mut rand::thread_rng());
        let address = Address::from(key);

        // sign a message
        let signature = key.sign(message);

        // ecrecover via the message will hash internally
        let recovered = signature.recover(message).unwrap();

        // if provided with a hash, it will skip hashing
        let recovered2 = signature.recover(hash).unwrap();

        assert_eq!(recovered, address);
        assert_eq!(recovered2, address);
    }

    #[test]
    fn to_vec() {
        let message = "Some data";
        let key = PrivateKey::new(&mut rand::thread_rng());
        let signature = key.sign(message);
        let serialized = signature.to_vec();
        let de = Signature::try_from(&serialized[..]).unwrap();
        assert_eq!(signature, de);
    }
}
