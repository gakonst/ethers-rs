use crate::{Client, ClientError, Signer};

use ethers_providers::{JsonRpcClient, Provider};

use ethers_core::{
    rand::Rng,
    secp256k1,
    types::{
        Address, PrivateKey, PublicKey, Signature, Transaction, TransactionRequest, TxError,
    },
};

use std::str::FromStr;

/// An Ethereum keypair
#[derive(Clone, Debug)]
pub struct Wallet {
    /// The Wallet's private Key
    pub private_key: PrivateKey,
    /// The Wallet's public Key
    pub public_key: PublicKey,
    /// The wallet's address
    pub address: Address,
    /// The wallet's chain id (for EIP-155), signs w/o replay protection if left unset
    pub chain_id: u64,
}

impl Signer for Wallet {
    type Error = TxError;

    fn sign_message<S: AsRef<[u8]>>(&self, message: S) -> Signature {
        self.private_key.sign(message)
    }

    fn sign_transaction(&self, tx: TransactionRequest) -> Result<Transaction, Self::Error> {
        self.private_key.sign_transaction(tx, Some(self.chain_id))
    }

    fn address(&self) -> Address {
        self.address
    }
}

impl From<TxError> for ClientError {
    fn from(src: TxError) -> Self {
        ClientError::SignerError(Box::new(src))
    }
}

impl Wallet {
    // TODO: Add support for mnemonic and encrypted JSON

    /// Creates a new random keypair seeded with the provided RNG
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let private_key = PrivateKey::new(rng);
        let public_key = PublicKey::from(&private_key);
        let address = Address::from(&private_key);

        Self {
            private_key,
            public_key,
            address,
            chain_id: 1,
        }
    }

    /// Connects to a provider and returns a client
    pub fn connect<P: JsonRpcClient>(self, provider: Provider<P>) -> Client<P, Wallet> {
        Client {
            signer: Some(self),
            provider,
        }
    }

    /// Sets the wallet's chain_id
    pub fn chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.chain_id = chain_id.into();
        self
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
            chain_id: 1,
        }
    }
}

impl FromStr for Wallet {
    type Err = secp256k1::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(PrivateKey::from_str(src)?.into())
    }
}
