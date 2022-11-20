//! Specific helper functions for loading an offline K256 Private Key stored on disk
use super::Wallet;

use crate::wallet::mnemonic::MnemonicBuilderError;
use coins_bip32::Bip32Error;
use coins_bip39::MnemonicError;
#[cfg(not(target_arch = "wasm32"))]
use elliptic_curve::rand_core;
#[cfg(not(target_arch = "wasm32"))]
use eth_keystore::KeystoreError;
use ethers_core::{
    k256::ecdsa::{self, SigningKey},
    rand::{CryptoRng, Rng},
    utils::secret_key_to_address,
};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
/// Error thrown by the Wallet module
pub enum WalletError {
    /// Error propagated from the BIP-32 crate
    #[error(transparent)]
    Bip32Error(#[from] Bip32Error),
    /// Error propagated from the BIP-39 crate
    #[error(transparent)]
    Bip39Error(#[from] MnemonicError),
    /// Underlying eth keystore error
    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    EthKeystoreError(#[from] KeystoreError),
    /// Error propagated from k256's ECDSA module
    #[error(transparent)]
    EcdsaError(#[from] ecdsa::Error),
    /// Error propagated from the hex crate.
    #[error(transparent)]
    HexError(#[from] hex::FromHexError),
    /// Error propagated by IO operations
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    /// Error propagated from the mnemonic builder module.
    #[error(transparent)]
    MnemonicBuilderError(#[from] MnemonicBuilderError),
    /// Error type from Eip712Error message
    #[error("error encoding eip712 struct: {0:?}")]
    Eip712Error(String),
}

impl Wallet<SigningKey> {
    /// Creates a new random encrypted JSON with the provided password and stores it in the
    /// provided directory. Returns a tuple (Wallet, String) of the wallet instance for the
    /// keystore with its random UUID. Accepts an optional name for the keystore file. If `None`,
    /// the keystore is stored as the stringified UUID.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_keystore<P, R, S>(
        dir: P,
        rng: &mut R,
        password: S,
        name: Option<&str>,
    ) -> Result<(Self, String), WalletError>
    where
        P: AsRef<Path>,
        R: Rng + CryptoRng + rand_core::CryptoRng,
        S: AsRef<[u8]>,
    {
        let (secret, uuid) = eth_keystore::new(dir, rng, password, name)?;
        let signer = SigningKey::from_bytes(secret.as_slice())?;
        let address = secret_key_to_address(&signer);
        Ok((Self { signer, address, chain_id: 1 }, uuid))
    }

    /// Decrypts an encrypted JSON from the provided path to construct a Wallet instance
    #[cfg(not(target_arch = "wasm32"))]
    pub fn decrypt_keystore<P, S>(keypath: P, password: S) -> Result<Self, WalletError>
    where
        P: AsRef<Path>,
        S: AsRef<[u8]>,
    {
        let secret = eth_keystore::decrypt_key(keypath, password)?;
        let signer = SigningKey::from_bytes(secret.as_slice())?;
        let address = secret_key_to_address(&signer);
        Ok(Self { signer, address, chain_id: 1 })
    }

    /// Creates a new random keypair seeded with the provided RNG
    pub fn new<R: Rng + CryptoRng>(rng: &mut R) -> Self {
        let signer = SigningKey::random(rng);
        let address = secret_key_to_address(&signer);
        Self { signer, address, chain_id: 1 }
    }
}

impl PartialEq for Wallet<SigningKey> {
    fn eq(&self, other: &Self) -> bool {
        self.signer.to_bytes().eq(&other.signer.to_bytes()) &&
            self.address == other.address &&
            self.chain_id == other.chain_id
    }
}

impl From<SigningKey> for Wallet<SigningKey> {
    fn from(signer: SigningKey) -> Self {
        let address = secret_key_to_address(&signer);

        Self { signer, address, chain_id: 1 }
    }
}

use ethers_core::k256::SecretKey as K256SecretKey;

impl From<K256SecretKey> for Wallet<SigningKey> {
    fn from(key: K256SecretKey) -> Self {
        let signer = key.into();
        let address = secret_key_to_address(&signer);

        Self { signer, address, chain_id: 1 }
    }
}

impl FromStr for Wallet<SigningKey> {
    type Err = WalletError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let src = hex::decode(src)?;
        let sk = SigningKey::from_bytes(&src)?;
        Ok(sk.into())
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use crate::Signer;
    use ethers_core::types::Address;
    use tempfile::tempdir;

    #[tokio::test]
    async fn encrypted_json_keystore() {
        // create and store a random encrypted JSON keystore in this directory
        let dir = tempdir().unwrap();
        let mut rng = rand::thread_rng();
        let (key, uuid) =
            Wallet::<SigningKey>::new_keystore(&dir, &mut rng, "randpsswd", None).unwrap();

        // sign a message using the above key
        let message = "Some data";
        let signature = key.sign_message(message).await.unwrap();

        // read from the encrypted JSON keystore and decrypt it, while validating that the
        // signatures produced by both the keys should match
        let path = Path::new(dir.path()).join(uuid);
        let key2 = Wallet::<SigningKey>::decrypt_keystore(path.clone(), "randpsswd").unwrap();
        let signature2 = key2.sign_message(message).await.unwrap();
        assert_eq!(signature, signature2);
        std::fs::remove_file(&path).unwrap();
    }

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
        use crate::TypedTransaction;
        use ethers_core::types::{TransactionRequest, U64};
        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        let tx: TypedTransaction = TransactionRequest {
            from: None,
            to: Some("F0109fC8DF283027b6285cc889F5aA624EaC1F55".parse::<Address>().unwrap().into()),
            value: Some(1_000_000_000.into()),
            gas: Some(2_000_000.into()),
            nonce: Some(0.into()),
            gas_price: Some(21_000_000_000u128.into()),
            data: None,
            chain_id: Some(U64::one()),
        }
        .into();
        let wallet: Wallet<SigningKey> =
            "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318".parse().unwrap();
        let wallet = wallet.with_chain_id(tx.chain_id().unwrap().as_u64());

        let sig = wallet.sign_transaction(&tx).await.unwrap();
        let sighash = tx.sighash();
        sig.verify(sighash, wallet.address).unwrap();
    }

    #[tokio::test]
    #[cfg(not(feature = "celo"))]
    async fn signs_tx_empty_chain_id() {
        use crate::TypedTransaction;
        use ethers_core::types::TransactionRequest;
        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        let tx: TypedTransaction = TransactionRequest {
            from: None,
            to: Some("F0109fC8DF283027b6285cc889F5aA624EaC1F55".parse::<Address>().unwrap().into()),
            value: Some(1_000_000_000.into()),
            gas: Some(2_000_000.into()),
            nonce: Some(0.into()),
            gas_price: Some(21_000_000_000u128.into()),
            data: None,
            chain_id: None,
        }
        .into();
        let wallet: Wallet<SigningKey> =
            "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318".parse().unwrap();
        let wallet = wallet.with_chain_id(1u64);

        // this should populate the tx chain_id as the signer's chain_id (1) before signing
        let sig = wallet.sign_transaction(&tx).await.unwrap();

        // since we initialize with None we need to re-set the chain_id for the sighash to be
        // correct
        let mut tx = tx;
        tx.set_chain_id(1);
        let sighash = tx.sighash();
        sig.verify(sighash, wallet.address).unwrap();
    }

    #[test]
    #[cfg(not(feature = "celo"))]
    fn signs_tx_empty_chain_id_sync() {
        use crate::TypedTransaction;
        use ethers_core::types::TransactionRequest;

        let chain_id = 1337u64;
        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        let tx: TypedTransaction = TransactionRequest {
            from: None,
            to: Some("F0109fC8DF283027b6285cc889F5aA624EaC1F55".parse::<Address>().unwrap().into()),
            value: Some(1_000_000_000u64.into()),
            gas: Some(2_000_000u64.into()),
            nonce: Some(0u64.into()),
            gas_price: Some(21_000_000_000u128.into()),
            data: None,
            chain_id: None,
        }
        .into();
        let wallet: Wallet<SigningKey> =
            "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318".parse().unwrap();
        let wallet = wallet.with_chain_id(chain_id);

        // this should populate the tx chain_id as the signer's chain_id (1337) before signing and
        // normalize the v
        let sig = wallet.sign_transaction_sync(&tx);

        // ensure correct v given the chain - first extract recid
        let recid = (sig.v - 35) % 2;
        // eip155 check
        assert_eq!(sig.v, chain_id * 2 + 35 + recid);

        // since we initialize with None we need to re-set the chain_id for the sighash to be
        // correct
        let mut tx = tx;
        tx.set_chain_id(chain_id);
        let sighash = tx.sighash();
        sig.verify(sighash, wallet.address).unwrap();
    }

    #[test]
    fn key_to_address() {
        let wallet: Wallet<SigningKey> =
            "0000000000000000000000000000000000000000000000000000000000000001".parse().unwrap();
        assert_eq!(
            wallet.address,
            Address::from_str("7E5F4552091A69125d5DfCb7b8C2659029395Bdf").expect("Decoding failed")
        );

        let wallet: Wallet<SigningKey> =
            "0000000000000000000000000000000000000000000000000000000000000002".parse().unwrap();
        assert_eq!(
            wallet.address,
            Address::from_str("2B5AD5c4795c026514f8317c7a215E218DcCD6cF").expect("Decoding failed")
        );

        let wallet: Wallet<SigningKey> =
            "0000000000000000000000000000000000000000000000000000000000000003".parse().unwrap();
        assert_eq!(
            wallet.address,
            Address::from_str("6813Eb9362372EEF6200f3b1dbC3f819671cBA69").expect("Decoding failed")
        );
    }
}
