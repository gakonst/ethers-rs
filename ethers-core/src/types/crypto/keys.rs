use crate::{
    types::{Address, NameOrAddress, Signature, Transaction, TransactionRequest, H256, U256},
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
use thiserror::Error;

use k256::{
    ecdsa::{
        recoverable::{Id as RecoveryId, Signature as RecoverableSignature},
        signature::Signer,
        SigningKey,
    },
    elliptic_curve::{error::Error as EllipticCurveError, FieldBytes},
    EncodedPoint as K256PublicKey, SecretKey as K256SecretKey, Secp256k1,
};

const SECRET_KEY_SIZE: usize = 32;
const FULL_PUBLIC_KEY_SIZE: usize = 65;
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

/// An error which may be thrown when attempting to sign a transaction with
/// missing fields
#[derive(Clone, Debug, Error)]
pub enum TxError {
    /// Thrown if the `nonce` field is missing
    #[error("no nonce was specified")]
    NonceMissing,
    /// Thrown if the `gas_price` field is missing
    #[error("no gas price was specified")]
    GasPriceMissing,
    /// Thrown if the `gas` field is missing
    #[error("no gas was specified")]
    GasMissing,
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

        self.sign_with_eip155(message_hash.as_bytes(), None)
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
    pub fn sign_transaction(
        &self,
        tx: TransactionRequest,
        chain_id: Option<u64>,
    ) -> Result<Transaction, TxError> {
        // The nonce, gas and gasprice fields must already be populated
        let nonce = tx.nonce.ok_or(TxError::NonceMissing)?;
        let gas_price = tx.gas_price.ok_or(TxError::GasPriceMissing)?;
        let gas = tx.gas.ok_or(TxError::GasMissing)?;

        // Get the transaction's sighash
        let sighash = tx.sighash(chain_id);

        // Sign it (with replay protection if applicable)
        let signature = self.sign_with_eip155(sighash.as_bytes(), chain_id);

        // Get the actual transaction hash
        let rlp = tx.rlp_signed(&signature);
        let hash = keccak256(&rlp.0);

        // This function should not be called with ENS names
        let to = tx.to.map(|to| match to {
            NameOrAddress::Address(inner) => inner,
            NameOrAddress::Name(_) => {
                panic!("Expected `to` to be an Ethereum Address, not an ENS name")
            }
        });

        Ok(Transaction {
            hash: hash.into(),
            nonce,
            from: self.into(),
            to,
            value: tx.value.unwrap_or_default(),
            gas_price,
            gas,
            input: tx.data.unwrap_or_default(),
            v: signature.v.into(),
            r: U256::from_big_endian(signature.r.as_bytes()),
            s: U256::from_big_endian(signature.s.as_bytes()),

            // Leave these empty as they're only used for included transactions
            block_hash: None,
            block_number: None,
            transaction_index: None,

            // Celo support
            #[cfg(feature = "celo")]
            fee_currency: tx.fee_currency,
            #[cfg(feature = "celo")]
            gateway_fee: tx.gateway_fee,
            #[cfg(feature = "celo")]
            gateway_fee_recipient: tx.gateway_fee_recipient,
        })
    }

    fn sign_with_eip155(&self, message: &[u8], chain_id: Option<u64>) -> Signature {
        let signing_key = SigningKey::new(&self.0.to_bytes()).expect("invalid secret key");

        let recoverable_sig: RecoverableSignature = signing_key.sign(message);

        let v = to_eip155_v(recoverable_sig.recovery_id(), chain_id);

        let r_bytes: FieldBytes<Secp256k1> = recoverable_sig.r().into();
        let s_bytes: FieldBytes<Secp256k1> = recoverable_sig.s().into();
        let r = H256::from_slice(&r_bytes.as_slice());
        let s = H256::from_slice(&s_bytes.as_slice());

        Signature { r: r, s: s, v: v }
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

        for e in self.0.as_bytes().iter() {
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

                match K256PublicKey::from_bytes(&bytes[..]) {
                    Ok(public_key) => Ok(PublicKey(public_key)),
                    _ => Err(DeserializeError::custom("could not parse public key from bytes"))
                }
            }
        }

        deserializer.deserialize_tuple(FULL_PUBLIC_KEY_SIZE, ArrayVisitor)
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
            assert_eq!(serialized.as_slice(), key.0.as_bytes().as_slice());
            let de: PrivateKey = bincode::deserialize(&serialized).unwrap();
            assert_eq!(key, de);

            let public = PublicKey::from(&key);

            println!("public = {:?}", public);

            let serialized = bincode::serialize(&public).unwrap();
            assert_eq!(serialized.as_slice(), public.0.as_bytes());
            let de: PublicKey = bincode::deserialize(&serialized).unwrap();
            assert_eq!(public, de);
        }
    }

    #[test]
    #[cfg(not(feature = "celo"))]
    fn signs_tx() {
        use crate::types::{Address, Bytes};
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

        let tx = key.sign_transaction(tx, Some(chain_id)).unwrap();

        assert_eq!(
            tx.hash,
            "de8db924885b0803d2edc335f745b2b8750c8848744905684c20b987443a9593"
                .parse()
                .unwrap()
        );

        let expected_rlp = Bytes("f869808504e3b29200831e848094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca008025a0c9cf86333bcb065d140032ecaab5d9281bde80f21b9687b3e94161de42d51895a0727a108a0b8d101465414033c3f705a9c7b826e596766046ee1183dbc8aeaa68".from_hex().unwrap());
        assert_eq!(tx.rlp(), expected_rlp);
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
