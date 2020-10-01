use super::hash::Sha256Proxy;
use crate::{
    types::{Address, Signature, TransactionRequest, H256},
    utils::{hash_message, keccak256},
};

use rand::{CryptoRng, Rng};
use rustc_hex::FromHex;
use serde::{
    de::Error as DeserializeError,
    de::{SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{fmt, ops::Deref, str::FromStr};

use k256::{
    ecdsa::{
        recoverable::{Id as RecoveryId, Signature as RecoverableSignature},
        signature::DigestSigner,
        SigningKey,
    },
    elliptic_curve::{error::Error as EllipticCurveError, FieldBytes},
    EncodedPoint as K256PublicKey, Secp256k1, SecretKey as K256SecretKey,
};

const SECRET_KEY_SIZE: usize = 32;
const COMPRESSED_PUBLIC_KEY_SIZE: usize = 33;

/// A private key on Secp256k1
#[derive(Clone, Debug)]
pub struct PrivateKey(pub(super) K256SecretKey);

impl PartialEq for PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bytes().eq(&other.0.to_bytes())
    }
}

impl Serialize for PrivateKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(SECRET_KEY_SIZE)?;
        for e in self.0.to_bytes() {
            seq.serialize_element(&e)?;
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
            K256SecretKey::from_bytes(&bytes).map_err(DeserializeError::custom)?,
        ))
    }
}

impl FromStr for PrivateKey {
    type Err = EllipticCurveError;

    fn from_str(src: &str) -> Result<PrivateKey, Self::Err> {
        let src = src
            .from_hex::<Vec<u8>>()
            .expect("invalid hex when reading PrivateKey");
        let sk = K256SecretKey::from_bytes(&src)?;
        Ok(PrivateKey(sk))
    }
}

impl PrivateKey {
    pub fn new<R: Rng + CryptoRng>(rng: &mut R) -> Self {
        PrivateKey(K256SecretKey::random(rng))
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

        self.sign_hash_with_eip155(message_hash, None)
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
        let sighash = tx.sighash(chain_id);
        self.sign_hash_with_eip155(sighash, chain_id)
    }

    fn sign_hash_with_eip155(&self, hash: H256, chain_id: Option<u64>) -> Signature {
        let signing_key = SigningKey::new(&self.0.to_bytes()).expect("invalid secret key");

        let recoverable_sig: RecoverableSignature =
            signing_key.sign_digest(Sha256Proxy::from(hash));

        let v = to_eip155_v(recoverable_sig.recovery_id(), chain_id);

        let r_bytes: FieldBytes<Secp256k1> = recoverable_sig.r().into();
        let s_bytes: FieldBytes<Secp256k1> = recoverable_sig.s().into();
        let r = H256::from_slice(&r_bytes.as_slice());
        let s = H256::from_slice(&s_bytes.as_slice());

        Signature { r, s, v }
    }
}

/// Applies [EIP155](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md)
fn to_eip155_v(recovery_id: RecoveryId, chain_id: Option<u64>) -> u64 {
    let standard_v: u8 = recovery_id.into();
    if let Some(chain_id) = chain_id {
        // When signing with a chain ID, add chain replay protection.
        (standard_v as u64) + 35 + chain_id * 2
    } else {
        // Otherwise, convert to 'Electrum' notation.
        (standard_v as u64) + 27
    }
}

impl Deref for PrivateKey {
    type Target = K256SecretKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A secp256k1 Public Key
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicKey(pub(super) K256PublicKey);

impl From<K256PublicKey> for PublicKey {
    /// Gets the public address of a private key.
    fn from(src: K256PublicKey) -> PublicKey {
        PublicKey(src)
    }
}

impl From<&PrivateKey> for PublicKey {
    /// Gets the public address of a private key.
    fn from(src: &PrivateKey) -> PublicKey {
        let public_key = K256PublicKey::from_secret_key(src, false);
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
        let public_key = src.0.as_bytes();

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

        for e in self.0.compress().as_bytes().iter() {
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

                let pub_key = K256PublicKey::from_bytes(&bytes[..])
                    .map_err(|_| DeserializeError::custom("parse pub key"))?;

                let uncompressed_pub_key = pub_key.decompress();
                if uncompressed_pub_key.is_some().into() {
                    Ok(PublicKey(uncompressed_pub_key.unwrap()))
                } else {
                    Err(DeserializeError::custom("parse pub key"))
                }
            }
        }

        deserializer.deserialize_tuple(COMPRESSED_PUBLIC_KEY_SIZE, ArrayVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde() {
        for _ in 0..10 {
            let key = PrivateKey::new(&mut rand::thread_rng());
            let serialized = bincode::serialize(&key).unwrap();
            assert_eq!(serialized.as_slice(), key.0.to_bytes().as_slice());
            let de: PrivateKey = bincode::deserialize(&serialized).unwrap();
            assert_eq!(key, de);

            let public = PublicKey::from(&key);
            println!("public = {:?}", public);

            let serialized = bincode::serialize(&public).unwrap();
            let de: PublicKey = bincode::deserialize(&serialized).unwrap();
            assert_eq!(public, de);
        }
    }

    #[test]
    #[cfg(not(feature = "celo"))]
    fn signs_tx() {
        use crate::types::Address;
        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        let tx = TransactionRequest {
            from: None,
            to: Some(
                "F0109fC8DF283027b6285cc889F5aA624EaC1F55"
                    .parse::<Address>()
                    .unwrap()
                    .into(),
            ),
            value: Some(1_000_000_000.into()),
            gas: Some(2_000_000.into()),
            nonce: Some(0.into()),
            gas_price: Some(21_000_000_000u128.into()),
            data: None,
        };
        let chain_id = 1;

        let key: PrivateKey = "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
            .parse()
            .unwrap();

        let sig = key.sign_transaction(&tx, Some(chain_id));
        let sighash = tx.sighash(Some(chain_id));
        assert!(sig.verify(sighash, Address::from(key)).is_ok());
    }

    #[test]
    fn key_to_address() {
        let priv_key: PrivateKey =
            "0000000000000000000000000000000000000000000000000000000000000001"
                .parse()
                .unwrap();
        let addr: Address = priv_key.into();
        assert_eq!(
            addr,
            Address::from_str("7E5F4552091A69125d5DfCb7b8C2659029395Bdf").expect("Decoding failed")
        );

        let priv_key: PrivateKey =
            "0000000000000000000000000000000000000000000000000000000000000002"
                .parse()
                .unwrap();
        let addr: Address = priv_key.into();
        assert_eq!(
            addr,
            Address::from_str("2B5AD5c4795c026514f8317c7a215E218DcCD6cF").expect("Decoding failed")
        );

        let priv_key: PrivateKey =
            "0000000000000000000000000000000000000000000000000000000000000003"
                .parse()
                .unwrap();
        let addr: Address = priv_key.into();
        assert_eq!(
            addr,
            Address::from_str("6813Eb9362372EEF6200f3b1dbC3f819671cBA69").expect("Decoding failed")
        );
    }
}
