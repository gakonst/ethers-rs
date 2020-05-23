use crate::{
    jsonrpc::ClientError,
    providers::{Provider, ProviderTrait},
    types::{Address, PrivateKey, PublicKey, Signature, U64},
    types::{Transaction, UnsignedTransaction},
};
use rand::Rng;
use std::{marker::PhantomData, str::FromStr};

use thiserror::Error;

/// A keypair
#[derive(Clone, Debug)]
pub struct Wallet<N> {
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
    pub address: Address,
    network: PhantomData<N>,
}

pub trait Network {
    const CHAIN_ID: Option<U64>;

    // TODO: Default providers?
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Mainnet;

impl Network for Mainnet {
    const CHAIN_ID: Option<U64> = Some(U64([1]));
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AnyNet;

impl Network for AnyNet {
    const CHAIN_ID: Option<U64> = None;
}

// No EIP-155 used
pub type AnyWallet = Wallet<AnyNet>;

impl<N: Network> Wallet<N> {
    /// Creates a new keypair
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

    /// Connects to a provider and returns a signer
    pub fn connect<'a>(self, provider: &'a Provider) -> Signer<Wallet<N>> {
        Signer {
            inner: self,
            provider: Some(provider),
        }
    }
}

impl<N: Network> FromStr for Wallet<N> {
    type Err = secp256k1::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(PrivateKey::from_str(src)?.into())
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

#[derive(Clone, Debug)]
pub struct Signer<'a, S> {
    pub provider: Option<&'a Provider>,
    pub inner: S,
}

#[derive(Error, Debug)]
pub enum SignerError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error("no provider was found")]
    NoProvider,
}

impl<'a, N: Network> Signer<'a, Wallet<N>> {
    /// Generates a random signer with no provider. Should be combined with the
    /// `connect` method like `Signer::random(rng).connect(provider)`
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        Signer {
            provider: None,
            inner: Wallet::new(rng),
        }
    }

    pub async fn send_transaction(
        &self,
        tx: UnsignedTransaction,
    ) -> Result<Transaction, SignerError> {
        // TODO: Is there any nicer way to do this?
        let provider = self.ensure_provider()?;

        let signed_tx = self.sign_transaction(tx.clone());

        provider.send_raw_transaction(&signed_tx.rlp()).await?;

        Ok(signed_tx)
    }
}

impl<'a, S> Signer<'a, S> {
    /// Sets the provider for the signer
    pub fn connect(mut self, provider: &'a Provider) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn ensure_provider(&self) -> Result<&Provider, SignerError> {
        if let Some(provider) = self.provider {
            Ok(provider)
        } else {
            Err(SignerError::NoProvider)
        }
    }
}

trait SignerC {
    /// Signs the hash of the provided message after prefixing it
    fn sign_message<S: AsRef<[u8]>>(&self, message: S) -> Signature;
    fn sign_transaction(&self, message: UnsignedTransaction) -> Transaction;
}

impl<'a, N: Network> SignerC for Signer<'a, Wallet<N>> {
    fn sign_message<S: AsRef<[u8]>>(&self, message: S) -> Signature {
        self.inner.private_key.sign(message)
    }

    fn sign_transaction(&self, tx: UnsignedTransaction) -> Transaction {
        self.inner.private_key.sign_transaction(tx, N::CHAIN_ID)
    }
}
