//! AWS KMS-based signer.

use super::Signer;
use aws_sdk_kms::{
    error::SdkError,
    operation::{
        get_public_key::{GetPublicKeyError, GetPublicKeyOutput},
        sign::{SignError, SignOutput},
    },
    primitives::Blob,
    types::{MessageType, SigningAlgorithmSpec},
    Client,
};
use ethers_core::{
    k256::ecdsa::{Error as K256Error, Signature as KSig, VerifyingKey},
    types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Address, Signature as EthSig, H256,
    },
    utils::hash_message,
};
use std::fmt;
use tracing::{debug, instrument, trace};

mod utils;

/// An Ethers signer that uses keys held in Amazon Web Services Key Management Service (AWS KMS).
///
/// The AWS Signer passes signing requests to the cloud service. AWS KMS keys
/// are identified by a UUID, the `key_id`.
///
/// Because the public key is unknown, we retrieve it on instantiation of the
/// signer. This means that the new function is `async` and must be called
/// within some runtime.
///
/// ```no_run
/// # async fn test() {
/// use aws_config::BehaviorVersion;
/// use ethers_signers::{AwsSigner, Signer};
///
/// let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
/// let client = aws_sdk_kms::Client::new(&config);
///
/// let key_id = "...";
/// let chain_id = 1;
/// let signer = AwsSigner::new(client, key_id, chain_id).await.unwrap();
///
/// let message = vec![0, 1, 2, 3];
///
/// let sig = signer.sign_message(&message).await.unwrap();
/// sig.verify(message, signer.address()).expect("valid sig");
/// # }
/// ```
#[derive(Clone)]
pub struct AwsSigner {
    kms: Client,
    chain_id: u64,
    key_id: String,
    pubkey: VerifyingKey,
    address: Address,
}

impl fmt::Debug for AwsSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AwsSigner")
            .field("key_id", &self.key_id)
            .field("chain_id", &self.chain_id)
            .field("pubkey", &hex::encode(self.pubkey.to_sec1_bytes()))
            .field("address", &self.address)
            .finish()
    }
}

impl fmt::Display for AwsSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// Errors thrown by [`AwsSigner`].
#[derive(thiserror::Error, Debug)]
pub enum AwsSignerError {
    #[error(transparent)]
    SignError(#[from] SdkError<SignError>),
    #[error(transparent)]
    GetPublicKeyError(#[from] SdkError<GetPublicKeyError>),
    #[error(transparent)]
    K256(#[from] K256Error),
    #[error(transparent)]
    Spki(#[from] spki::Error),
    /// Error when converting from a hex string
    #[error(transparent)]
    HexError(#[from] hex::FromHexError),
    /// Error type from Eip712Error message
    #[error("failed encoding eip712 struct: {0:?}")]
    Eip712Error(String),
    #[error("{0}")]
    Other(String),
}

impl From<String> for AwsSignerError {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl AwsSigner {
    /// Instantiate a new signer from an existing `Client` and key ID.
    ///
    /// This function retrieves the public key from AWS and calculates the
    /// Etheruem address. It is therefore `async`.
    #[instrument(err, skip_all, fields(key_id = %key_id.as_ref()))]
    pub async fn new<T: AsRef<str>>(
        kms: Client,
        key_id: T,
        chain_id: u64,
    ) -> Result<AwsSigner, AwsSignerError> {
        let key_id = key_id.as_ref();
        let resp = request_get_pubkey(&kms, key_id).await?;
        let pubkey = decode_pubkey(resp)?;
        let address = ethers_core::utils::public_key_to_address(&pubkey);

        debug!(
            "Instantiated AWS signer with pubkey 0x{} and address {address:?}",
            hex::encode(pubkey.to_sec1_bytes()),
        );

        Ok(Self { kms, chain_id, key_id: key_id.into(), pubkey, address })
    }

    /// Fetch the pubkey associated with a key ID.
    pub async fn get_pubkey_for_key<T>(&self, key_id: T) -> Result<VerifyingKey, AwsSignerError>
    where
        T: AsRef<str>,
    {
        request_get_pubkey(&self.kms, key_id.as_ref()).await.and_then(decode_pubkey)
    }

    /// Fetch the pubkey associated with this signer's key ID.
    pub async fn get_pubkey(&self) -> Result<VerifyingKey, AwsSignerError> {
        self.get_pubkey_for_key(&self.key_id).await
    }

    /// Sign a digest with the key associated with a key ID.
    pub async fn sign_digest_with_key<T: AsRef<str>>(
        &self,
        key_id: T,
        digest: [u8; 32],
    ) -> Result<KSig, AwsSignerError> {
        request_sign_digest(&self.kms, key_id.as_ref(), digest).await.and_then(decode_signature)
    }

    /// Sign a digest with this signer's key
    pub async fn sign_digest(&self, digest: [u8; 32]) -> Result<KSig, AwsSignerError> {
        self.sign_digest_with_key(self.key_id.clone(), digest).await
    }

    /// Sign a digest with this signer's key and add the eip155 `v` value
    /// corresponding to the input chain_id
    #[instrument(err, skip(digest), fields(digest = %hex::encode(digest)))]
    async fn sign_digest_with_eip155(
        &self,
        digest: H256,
        chain_id: u64,
    ) -> Result<EthSig, AwsSignerError> {
        let sig = self.sign_digest(digest.into()).await?;
        let mut sig =
            utils::sig_from_digest_bytes_trial_recovery(&sig, digest.into(), &self.pubkey);
        utils::apply_eip155(&mut sig, chain_id);
        Ok(sig)
    }
}

#[async_trait::async_trait]
impl Signer for AwsSigner {
    type Error = AwsSignerError;

    #[instrument(err, skip(message))]
    #[allow(clippy::blocks_in_conditions)]
    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<EthSig, Self::Error> {
        let message = message.as_ref();
        let message_hash = hash_message(message);
        trace!(?message_hash, ?message);

        self.sign_digest_with_eip155(message_hash, self.chain_id).await
    }

    #[instrument(err)]
    #[allow(clippy::blocks_in_conditions)]
    async fn sign_transaction(&self, tx: &TypedTransaction) -> Result<EthSig, Self::Error> {
        let mut tx_with_chain = tx.clone();
        let chain_id = tx_with_chain.chain_id().map(|id| id.as_u64()).unwrap_or(self.chain_id);
        tx_with_chain.set_chain_id(chain_id);

        let sighash = tx_with_chain.sighash();
        self.sign_digest_with_eip155(sighash, chain_id).await
    }

    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<EthSig, Self::Error> {
        let digest =
            payload.encode_eip712().map_err(|e| Self::Error::Eip712Error(e.to_string()))?;

        let sig = self.sign_digest(digest).await?;
        let sig = utils::sig_from_digest_bytes_trial_recovery(&sig, digest, &self.pubkey);

        Ok(sig)
    }

    fn address(&self) -> Address {
        self.address
    }

    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    fn with_chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.chain_id = chain_id.into();
        self
    }
}

#[instrument(err, skip(kms))]
async fn request_get_pubkey(
    kms: &Client,
    key_id: &str,
) -> Result<GetPublicKeyOutput, AwsSignerError> {
    kms.get_public_key().key_id(key_id).send().await.map_err(Into::into)
}

#[instrument(err, skip(kms, digest), fields(digest = %hex::encode(digest)))]
async fn request_sign_digest(
    kms: &Client,
    key_id: &str,
    digest: [u8; 32],
) -> Result<SignOutput, AwsSignerError> {
    kms.sign()
        .key_id(key_id)
        .message(Blob::new(digest))
        .message_type(MessageType::Digest)
        .signing_algorithm(SigningAlgorithmSpec::EcdsaSha256)
        .send()
        .await
        .map_err(Into::into)
}

/// Decode an AWS KMS Pubkey response.
fn decode_pubkey(resp: GetPublicKeyOutput) -> Result<VerifyingKey, AwsSignerError> {
    let raw = resp
        .public_key
        .as_ref()
        .ok_or_else(|| AwsSignerError::from("Pubkey not found in response".to_owned()))?;

    let spki = spki::SubjectPublicKeyInfoRef::try_from(raw.as_ref())?;
    let key = VerifyingKey::from_sec1_bytes(spki.subject_public_key.raw_bytes())?;

    Ok(key)
}

/// Decode an AWS KMS Signature response.
fn decode_signature(resp: SignOutput) -> Result<KSig, AwsSignerError> {
    let raw = resp
        .signature
        .as_ref()
        .ok_or_else(|| AwsSignerError::from("Signature not found in response".to_owned()))?;

    let sig = KSig::from_der(raw.as_ref())?;
    Ok(sig.normalize_s().unwrap_or(sig))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_config::BehaviorVersion;

    #[tokio::test]
    async fn sign_message() {
        let Ok(key_id) = std::env::var("AWS_KEY_ID") else { return };
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = aws_sdk_kms::Client::new(&config);

        let chain_id = 1;
        let signer = AwsSigner::new(client, key_id, chain_id).await.unwrap();

        let message = vec![0, 1, 2, 3];

        let sig = signer.sign_message(&message).await.unwrap();
        sig.verify(message, signer.address()).expect("valid sig");
    }
}
