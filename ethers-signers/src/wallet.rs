use crate::Signer;

use ethers_core::{
    k256::elliptic_curve::error::Error as K256Error,
    rand::{CryptoRng, Rng},
    types::{Address, PrivateKey, PublicKey, Signature, TransactionRequest},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// An Ethereum private-public key pair which can be used for signing messages.
///
/// # Examples
///
/// ## Signing and Verifying a message
///
/// The wallet can be used to produce ECDSA [`Signature`] objects, which can be
/// then verified. Note that this uses [`hash_message`] under the hood which will
/// prefix the message being hashed with the `Ethereum Signed Message` domain separator.
///
/// ```
/// use ethers_core::rand::thread_rng;
/// use ethers_signers::{Wallet, Signer};
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let wallet = Wallet::new(&mut thread_rng());
///
/// // Optionally, the wallet's chain id can be set, in order to use EIP-155
/// // replay protection with different chains
/// let wallet = wallet.set_chain_id(1337u64);
///
/// // The wallet can be used to sign messages
/// let message = b"hello";
/// let signature = wallet.sign_message(message).await?;
/// assert_eq!(signature.recover(&message[..]).unwrap(), wallet.address());
/// # Ok(())
/// # }
/// ```
///
/// [`Signature`]: ethers_core::types::Signature
/// [`hash_message`]: fn@ethers_core::utils::hash_message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wallet {
    /// The Wallet's private Key
    private_key: PrivateKey,
    /// The Wallet's public Key
    public_key: PublicKey,
    /// The wallet's address
    address: Address,
    /// The wallet's chain id (for EIP-155), signs w/o replay protection if left unset
    chain_id: Option<u64>,
}

#[async_trait(?Send)]
impl Signer for Wallet {
    type Error = std::convert::Infallible;

    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        Ok(self.private_key.sign(message))
    }

    async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<Signature, Self::Error> {
        Ok(self.private_key.sign_transaction(tx, self.chain_id))
    }

    fn address(&self) -> Address {
        self.address
    }
}

impl Wallet {
    // TODO: Add support for mnemonic and encrypted JSON

    /// Creates a new random keypair seeded with the provided RNG
    pub fn new<R: Rng + CryptoRng>(rng: &mut R) -> Self {
        let private_key = PrivateKey::new(rng);
        let public_key = PublicKey::from(&private_key);
        let address = Address::from(&private_key);

        Self {
            private_key,
            public_key,
            address,
            chain_id: None,
        }
    }

    /// Sets the wallet's chain_id, used in conjunction with EIP-155 signing
    pub fn set_chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.chain_id = Some(chain_id.into());
        self
    }

    /// Gets the wallet's public key
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Gets the wallet's private key
    pub fn private_key(&self) -> &PrivateKey {
        &self.private_key
    }

    /// Gets the wallet's chain id
    pub fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }

    /// Returns the wallet's address
    // (we duplicate this method
    pub fn address(&self) -> Address {
        self.address
    }
}

impl From<PrivateKey> for Wallet {
    fn from(private_key: PrivateKey) -> Self {
        let public_key = PublicKey::from(&private_key);
        let address = Address::from(&private_key);

        Self {
            private_key,
            public_key,
            address,
            chain_id: None,
        }
    }
}

impl FromStr for Wallet {
    type Err = K256Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(PrivateKey::from_str(src)?.into())
    }
}
