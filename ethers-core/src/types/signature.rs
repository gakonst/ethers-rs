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

/// An ERC-2098 Compact Signature Representation
/// [ERC-2098](https://eips.ethereum.org/EIPS/eip-2098)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub struct CompactSignature {
    /// R value
    pub r: U256,
    /// yParity and s value
    pub y_parity_and_s: U256,
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(hex::Buffer::<65, false>::new().format(&self.into()))
    }
}

impl fmt::Display for CompactSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(hex::Buffer::<64, false>::new().format(&self.into()))
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
        let message = message.into();
        let message_hash = match message {
            RecoveryMessage::Data(ref message) => hash_message(message),
            RecoveryMessage::Hash(hash) => hash,
        };

        let (recoverable_sig, recovery_id) = self.as_signature()?;
        let verify_key = VerifyingKey::recover_from_prehash(
            message_hash.as_ref(),
            &recoverable_sig,
            recovery_id,
        )?;

        let public_key = K256PublicKey::from(&verify_key);
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

fn normalize_recovery_id(v: u64) -> u8 {
    match v {
        0 => 0,
        1 => 1,
        27 => 0,
        28 => 1,
        v if v >= 35 => ((v - 1) % 2) as _,
        _ => 4,
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

impl<'a> TryFrom<&'a [u8]> for CompactSignature {
    type Error = SignatureError;

    /// Parses a raw compact signature which is expected to be 64 bytes long where
    /// the first 32 bytes is the `r` value, the second 32 bytes the `y_parity_and_s` value
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() != 64 {
            return Err(SignatureError::InvalidLength(value.len()))
        }

        let r = U256::from_big_endian(&value[0..32]);
        let y_parity_and_s = U256::from_big_endian(&value[32..64]);

        Ok(CompactSignature { r, y_parity_and_s })
    }
}

impl FromStr for CompactSignature {
    type Err = SignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CompactSignature::try_from(&hex::decode(s)?[..])
    }
}

impl From<&CompactSignature> for [u8; 64] {
    fn from(src: &CompactSignature) -> [u8; 64] {
        let mut sig: [u8; 64] = [0u8; 64];
        let mut r_bytes = [0u8; 32];
        let mut y_parity_and_s_bytes = [0u8; 32];
        src.r.to_big_endian(&mut r_bytes);
        src.y_parity_and_s.to_big_endian(&mut y_parity_and_s_bytes);
        sig[..32].copy_from_slice(&r_bytes);
        sig[32..64].copy_from_slice(&y_parity_and_s_bytes);
        sig
    }
}

impl From<CompactSignature> for [u8; 64] {
    fn from(src: CompactSignature) -> [u8; 64] {
        <[u8; 64]>::from(&src)
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

impl From<Signature> for CompactSignature {
    fn from(src: Signature) -> CompactSignature {
        let mut r_bytes: [u8; 32] = [0u8; 32];
        let mut y_parity_and_s_bytes: [u8; 32] = [0u8; 32];

        src.r.to_big_endian(&mut r_bytes);
        src.s.to_big_endian(&mut y_parity_and_s_bytes);

        if src.v == 28 {
            y_parity_and_s_bytes[0] |= 0x80;
        }

        CompactSignature {
            r: U256::from_big_endian(&r_bytes),
            y_parity_and_s: U256::from_big_endian(&y_parity_and_s_bytes),
        }
    }
}

impl From<CompactSignature> for Signature {
    fn from(src: CompactSignature) -> Signature {
        let mut s_bytes: [u8; 32] = [0u8; 32];
        let mut r_bytes: [u8; 32] = [0u8; 32];

        src.y_parity_and_s.to_big_endian(&mut s_bytes);
        src.r.to_big_endian(&mut r_bytes);

        let v: U256 = src.y_parity_and_s >> 255;
        if v == U256::one() {
            s_bytes[0] &= 0x7f;
        }
        Signature {
            r: U256::from_big_endian(&r_bytes),
            s: U256::from_big_endian(&s_bytes),
            v: 27 + v.as_u64(),
        }
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

    #[test]
    fn compact_signature_from_str() {
        let compact1 = CompactSignature::from_str(
            "0x68a020a209d3d56c46f38cc50a33f704f4a9a10a59377f8dd762ac66910e9b907e865ad05c4035ab5792787d4a0297a43617ae897930a6fe4d822b8faea52064"
        ).expect("could not parse 0x-prefixed compact signature");

        let compact2 = CompactSignature::from_str(
            "68a020a209d3d56c46f38cc50a33f704f4a9a10a59377f8dd762ac66910e9b907e865ad05c4035ab5792787d4a0297a43617ae897930a6fe4d822b8faea52064"
        ).expect("could not parse non-prefixed compact signature");

        assert_eq!(compact1, compact2);
    }

    #[test]
    fn compact_signature_from_signature() {
        let s0 = Signature {
            r: U256::from_str("0x68a020a209d3d56c46f38cc50a33f704f4a9a10a59377f8dd762ac66910e9b90")
                .unwrap(),
            s: U256::from_str("0x7e865ad05c4035ab5792787d4a0297a43617ae897930a6fe4d822b8faea52064")
                .unwrap(),
            v: 27,
        };
        let s1 = Signature {
            r: U256::from_str("0x9328da16089fcba9bececa81663203989f2df5fe1faa6291a45381c81bd17f76")
                .unwrap(),
            s: U256::from_str("0x139c6d6b623b42da56557e5e734a43dc83345ddfadec52cbe24d0cc64f550793")
                .unwrap(),
            v: 28,
        };

        let c0 = CompactSignature::from(s0);
        let c1 = CompactSignature::from(s1);

        assert_eq!(c0.r, s0.r);
        assert_eq!(c0.y_parity_and_s, s0.s);

        assert_eq!(c1.r, s1.r);
        assert_eq!(
            c1.y_parity_and_s,
            U256::from_str("0x939c6d6b623b42da56557e5e734a43dc83345ddfadec52cbe24d0cc64f550793")
                .unwrap()
        );
    }

    #[test]
    fn signature_from_compact_signature() {
        let c0 = CompactSignature {
            r: U256::from_str("0x68a020a209d3d56c46f38cc50a33f704f4a9a10a59377f8dd762ac66910e9b90")
                .unwrap(),
            y_parity_and_s: U256::from_str(
                "0x7e865ad05c4035ab5792787d4a0297a43617ae897930a6fe4d822b8faea52064",
            )
            .unwrap(),
        };
        let c1 = CompactSignature {
            r: U256::from_str("0x9328da16089fcba9bececa81663203989f2df5fe1faa6291a45381c81bd17f76")
                .unwrap(),
            y_parity_and_s: U256::from_str(
                "0x939c6d6b623b42da56557e5e734a43dc83345ddfadec52cbe24d0cc64f550793",
            )
            .unwrap(),
        };

        let s0 = Signature::from(c0);
        let s1 = Signature::from(c1);

        assert_eq!(s0.r, c0.r);
        assert_eq!(s0.s, c0.y_parity_and_s);
        assert_eq!(s0.v, 27);

        assert_eq!(s1.r, c1.r);
        assert_eq!(
            s1.s,
            U256::from_str("0x139c6d6b623b42da56557e5e734a43dc83345ddfadec52cbe24d0cc64f550793")
                .unwrap()
        );
        assert_eq!(s1.v, 28);
    }
}
