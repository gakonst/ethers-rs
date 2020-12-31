//! Specific helper functions for loading an offline K256 Private Key stored on disk
use super::Wallet;

use ethers_core::{
    k256::{
        ecdsa::SigningKey, elliptic_curve::error::Error as K256Error, EncodedPoint as K256PublicKey,
    },
    rand::{CryptoRng, Rng},
    types::Address,
    utils::keccak256,
};
use std::str::FromStr;

impl Clone for Wallet<SigningKey> {
    fn clone(&self) -> Self {
        Self {
            // TODO: Can we have a better way to clone here?
            signer: SigningKey::from_bytes(&*self.signer.to_bytes()).unwrap(),
            address: self.address,
            chain_id: self.chain_id,
        }
    }
}

impl Wallet<SigningKey> {
    // TODO: Add support for mnemonic and encrypted JSON

    /// Creates a new random keypair seeded with the provided RNG
    pub fn new<R: Rng + CryptoRng>(rng: &mut R) -> Self {
        let signer = SigningKey::random(rng);
        let address = key_to_address(&signer);
        Self {
            signer,
            address,
            chain_id: None,
        }
    }
}

fn key_to_address(secret_key: &SigningKey) -> Address {
    // TODO: Can we do this in a better way?
    let uncompressed_pub_key = K256PublicKey::from(&secret_key.verify_key()).decompress();
    let public_key = uncompressed_pub_key.unwrap().to_bytes();
    debug_assert_eq!(public_key[0], 0x04);
    let hash = keccak256(&public_key[1..]);
    Address::from_slice(&hash[12..])
}

impl PartialEq for Wallet<SigningKey> {
    fn eq(&self, other: &Self) -> bool {
        self.signer.to_bytes().eq(&other.signer.to_bytes())
            && self.address == other.address
            && self.chain_id == other.chain_id
    }
}

impl From<SigningKey> for Wallet<SigningKey> {
    fn from(signer: SigningKey) -> Self {
        let address = key_to_address(&signer);

        Self {
            signer,
            address,
            chain_id: None,
        }
    }
}

use ethers_core::k256::SecretKey as K256SecretKey;

impl From<K256SecretKey> for Wallet<SigningKey> {
    fn from(key: K256SecretKey) -> Self {
        let signer = SigningKey::from_bytes(&*key.to_bytes())
            .expect("private key should always be convertible to signing key");
        let address = key_to_address(&signer);

        Self {
            signer,
            address,
            chain_id: None,
        }
    }
}

impl FromStr for Wallet<SigningKey> {
    type Err = K256Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let src = hex::decode(src).expect("invalid hex when reading PrivateKey");
        let sk = SigningKey::from_bytes(&src).unwrap(); // TODO
        Ok(sk.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Signer;

    #[tokio::test]
    async fn signs_msg() {
        let message = "Some data";
        let hash = ethers_core::utils::hash_message(message);
        let key = Wallet::<SigningKey>::new(&mut rand::thread_rng());
        let address = key.address;

        // sign a message
        let signature = key.sign_message(message).await.unwrap();

        // ecrecover via the message will hash internally
        let recovered = signature.recover(message).unwrap();

        // if provided with a hash, it will skip hashing
        let recovered2 = signature.recover(hash).unwrap();

        // verifies the signature is produced by `address`
        signature.verify(message, address).unwrap();

        assert_eq!(recovered, address);
        assert_eq!(recovered2, address);
    }

    #[tokio::test]
    #[cfg(not(feature = "celo"))]
    async fn signs_tx() {
        use ethers_core::types::TransactionRequest;
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
        let chain_id = 1u64;

        let wallet: Wallet<SigningKey> =
            "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
                .parse()
                .unwrap();
        let wallet = wallet.set_chain_id(chain_id);

        let sig = wallet.sign_transaction(&tx).await.unwrap();
        let sighash = tx.sighash(Some(chain_id));
        assert!(sig.verify(sighash, wallet.address).is_ok());
    }

    #[test]
    fn key_to_address() {
        let wallet: Wallet<SigningKey> =
            "0000000000000000000000000000000000000000000000000000000000000001"
                .parse()
                .unwrap();
        assert_eq!(
            wallet.address,
            Address::from_str("7E5F4552091A69125d5DfCb7b8C2659029395Bdf").expect("Decoding failed")
        );

        let wallet: Wallet<SigningKey> =
            "0000000000000000000000000000000000000000000000000000000000000002"
                .parse()
                .unwrap();
        assert_eq!(
            wallet.address,
            Address::from_str("2B5AD5c4795c026514f8317c7a215E218DcCD6cF").expect("Decoding failed")
        );

        let wallet: Wallet<SigningKey> =
            "0000000000000000000000000000000000000000000000000000000000000003"
                .parse()
                .unwrap();
        assert_eq!(
            wallet.address,
            Address::from_str("6813Eb9362372EEF6200f3b1dbC3f819671cBA69").expect("Decoding failed")
        );
    }
}
