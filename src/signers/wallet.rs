use crate::{
    providers::{JsonRpcClient, Provider},
    signers::{Client, Network, Signer},
    types::{Address, PrivateKey, PublicKey, Signature, Transaction, UnsignedTransaction},
};

use rand::Rng;
use std::{marker::PhantomData, str::FromStr};

/// A keypair
#[derive(Clone, Debug)]
pub struct Wallet<N> {
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
    pub address: Address,
    network: PhantomData<N>,
}

impl<'a, N: Network> Signer for Wallet<N> {
    fn sign_message<S: AsRef<[u8]>>(&self, message: S) -> Signature {
        self.private_key.sign(message)
    }

    fn sign_transaction(&self, tx: UnsignedTransaction) -> Transaction {
        self.private_key.sign_transaction(tx, N::CHAIN_ID)
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
    pub fn connect<P: JsonRpcClient>(self, provider: &Provider<P>) -> Client<Wallet<N>, P> {
        Client {
            signer: self,
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
