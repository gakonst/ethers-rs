use crate::{Client, Signer};

use ethers_providers::{networks::Network, JsonRpcClient, Provider};

use ethers_core::{
    rand::Rng,
    secp256k1,
    types::{Address, PrivateKey, PublicKey, Signature, Transaction, TransactionRequest, TxError},
};

use std::{marker::PhantomData, str::FromStr};

/// An Ethereum keypair
#[derive(Clone, Debug)]
pub struct Wallet<N> {
    /// The Wallet's private Key
    pub private_key: PrivateKey,
    /// The Wallet's public Key
    pub public_key: PublicKey,
    /// The wallet's address
    pub address: Address,
    network: PhantomData<N>,
}

impl<'a, N: Network> Signer for Wallet<N> {
    type Error = TxError;

    fn sign_message<S: AsRef<[u8]>>(&self, message: S) -> Signature {
        self.private_key.sign(message)
    }

    fn sign_transaction(&self, tx: TransactionRequest) -> Result<Transaction, Self::Error> {
        self.private_key.sign_transaction(tx, N::CHAIN_ID)
    }

    fn address(&self) -> Address {
        self.address
    }
}

impl<N: Network> Wallet<N> {
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
            network: PhantomData,
        }
    }

    /// Connects to a provider and returns a client
    pub fn connect<P: JsonRpcClient>(self, provider: &Provider<P, N>) -> Client<P, N, Wallet<N>> {
        Client {
            signer: Some(self),
            provider,
        }
    }
}

impl<N: Network> From<PrivateKey> for Wallet<N> {
    fn from(private_key: PrivateKey) -> Self {
        let public_key = PublicKey::from(&private_key);
        let address = Address::from(&private_key);

        Self {
            private_key,
            public_key,
            address,
            network: PhantomData,
        }
    }
}

impl<N: Network> FromStr for Wallet<N> {
    type Err = secp256k1::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(PrivateKey::from_str(src)?.into())
    }
}
