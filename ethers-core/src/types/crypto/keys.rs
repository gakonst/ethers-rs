use crate::{
    types::{Address, Signature, TransactionRequest, H256},
    utils::{hash_message, keccak256},
};

use rand::Rng;
use rustc_hex::FromHex;
use secp256k1::{
    self as Secp256k1,
    util::{COMPRESSED_PUBLIC_KEY_SIZE, SECRET_KEY_SIZE},
    Error as SecpError, Message, PublicKey as PubKey, RecoveryId, SecretKey,
};
use serde::{
    de::Error as DeserializeError,
    de::{SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{fmt, ops::Deref, str::FromStr};

/// A private key on Secp256k1
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivateKey(pub(super) SecretKey);

impl Serialize for PrivateKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(SECRET_KEY_SIZE)?;
        for e in &self.0.serialize() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for PrivateKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = <[u8; SECRET_KEY_SIZE]>::deserialize(deserializer)?;
        Ok(PrivateKey(
            SecretKey::parse(&bytes).map_err(DeserializeError::custom)?,
        ))
    }
}

impl FromStr for PrivateKey {
    type Err = SecpError;

    fn from_str(src: &str) -> Result<PrivateKey, Self::Err> {
        let src = src
            .from_hex::<Vec<u8>>()
            .expect("invalid hex when reading PrivateKey");
        let sk = SecretKey::parse_slice(&src)?;
        Ok(PrivateKey(sk))
    }
}

impl PrivateKey {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        PrivateKey(SecretKey::random(rng))
    }

    /// Sign arbitrary string data.
    ///
    /// The data is UTF-8 encoded and enveloped the same way as with
    /// `hash_message`. The returned signed data's signature is in 'Electrum'
    /// notation, that is the recovery value `v` is either `27` or `28` (as
    /// opposed to the standard notation where `v` is either `0` or `1`). This
    /// is important to consider when using this signature with other crates.
    pub fn sign<S>(&self, message: S) -> Signature
    where
        S: AsRef<[u8]>,
    {
        let message = message.as_ref();
        let message_hash = hash_message(message);

        let sig_message =
            Message::parse_slice(message_hash.as_bytes()).expect("hash is non-zero 32-bytes; qed");
        self.sign_with_eip155(&sig_message, None)
    }

    /// RLP encodes and then signs the stransaction.
    ///
    /// If no chain_id is provided, then EIP-155 is not used.
    ///
    /// This will return an error if called if any of the `nonce`, `gas_price` or `gas`
    /// fields are not populated.
    ///
    /// # Panics
    ///
    /// If `tx.to` is an ENS name. The caller MUST take care of name resolution before
    /// calling this function.
    pub fn sign_transaction(&self, tx: &TransactionRequest, chain_id: Option<u64>) -> Signature {
        // Get the transaction's sighash
        let sighash = tx.sighash(chain_id);
        let message =
            Message::parse_slice(sighash.as_bytes()).expect("hash is non-zero 32-bytes; qed");
        // Sign it (with replay protection if applicable)
        self.sign_with_eip155(&message, chain_id)
    }

    fn sign_with_eip155(&self, message: &Message, chain_id: Option<u64>) -> Signature {
        let (signature, recovery_id) = Secp256k1::sign(message, &self.0);

        let v = to_eip155_v(recovery_id, chain_id);
        let r = H256::from_slice(&signature.r.b32());
        let s = H256::from_slice(&signature.s.b32());

        Signature { v, r, s }
    }
}

/// Applies [EIP155](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md)
fn to_eip155_v(recovery_id: RecoveryId, chain_id: Option<u64>) -> u64 {
    let standard_v = recovery_id.serialize() as u64;
    if let Some(chain_id) = chain_id {
        // When signing with a chain ID, add chain replay protection.
        standard_v + 35 + chain_id * 2
    } else {
        // Otherwise, convert to 'Electrum' notation.
        standard_v + 27
    }
}

impl Deref for PrivateKey {
    type Target = SecretKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A secp256k1 Public Key
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicKey(pub(super) PubKey);

impl From<PubKey> for PublicKey {
    /// Gets the public address of a private key.
    fn from(src: PubKey) -> PublicKey {
        PublicKey(src)
    }
}

impl From<&PrivateKey> for PublicKey {
    /// Gets the public address of a private key.
    fn from(src: &PrivateKey) -> PublicKey {
        let public_key = PubKey::from_secret_key(src);
        PublicKey(public_key)
    }
}

/// Gets the address of a public key.
///
/// The public address is defined as the low 20 bytes of the keccak hash of
/// the public key. Note that the public key returned from the `secp256k1`
/// crate is 65 bytes long, that is because it is prefixed by `0x04` to
/// indicate an uncompressed public key; this first byte is ignored when
/// computing the hash.
impl From<&PublicKey> for Address {
    fn from(src: &PublicKey) -> Address {
        let public_key = src.0.serialize();

        debug_assert_eq!(public_key[0], 0x04);
        let hash = keccak256(&public_key[1..]);

        Address::from_slice(&hash[12..])
    }
}

impl From<PublicKey> for Address {
    fn from(src: PublicKey) -> Address {
        Address::from(&src)
    }
}

impl From<&PrivateKey> for Address {
    fn from(src: &PrivateKey) -> Address {
        let public_key = PublicKey::from(src);
        Address::from(&public_key)
    }
}

impl From<PrivateKey> for Address {
    fn from(src: PrivateKey) -> Address {
        Address::from(&src)
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(COMPRESSED_PUBLIC_KEY_SIZE)?;
        for e in self.0.serialize_compressed().iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArrayVisitor;

        impl<'de> Visitor<'de> for ArrayVisitor {
            type Value = PublicKey;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid proof")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<PublicKey, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let mut bytes = [0u8; COMPRESSED_PUBLIC_KEY_SIZE];
                for b in &mut bytes[..] {
                    *b = seq
                        .next_element()?
                        .ok_or_else(|| DeserializeError::custom("could not read bytes"))?;
                }

                Ok(PublicKey(
                    PubKey::parse_compressed(&bytes).map_err(DeserializeError::custom)?,
                ))
            }
        }

        deserializer.deserialize_tuple(COMPRESSED_PUBLIC_KEY_SIZE, ArrayVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hex::FromHex;

    #[test]
    fn serde() {
        for _ in 0..10 {
            let key = PrivateKey::new(&mut rand::thread_rng());
            let serialized = bincode::serialize(&key).unwrap();
            assert_eq!(serialized, &key.0.serialize());
            let de: PrivateKey = bincode::deserialize(&serialized).unwrap();
            assert_eq!(key, de);

            let public = PublicKey::from(&key);
            let serialized = bincode::serialize(&public).unwrap();
            assert_eq!(&serialized[..], public.0.serialize_compressed().as_ref());
            let de: PublicKey = bincode::deserialize(&serialized).unwrap();
            assert_eq!(public, de);
        }
    }

    #[test]
    fn signs_data() {
        // test vector taken from:
        // https://web3js.readthedocs.io/en/v1.2.2/web3-eth-accounts.html#sign

        let key: PrivateKey = "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
            .parse()
            .unwrap();
        let sign = key.sign("Some data");

        assert_eq!(
            sign.to_vec(),
            "b91467e570a6466aa9e9876cbcd013baba02900b8979d43fe208a4a4f339f5fd6007e74cd82e037b800186422fc2da167c747ef045e5d18a5f5d4300f8e1a0291c"
            .from_hex::<Vec<u8>>()
            .unwrap()
        );
    }
}
