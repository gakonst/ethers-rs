//! AWS KMS-based Signer

use ethers_core::{
    k256::ecdsa::{Error as K256Error, Signature as KSig, VerifyingKey},
    types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Address, Signature as EthSig, H256,
    },
    utils::hash_message,
};
use rusoto_core::RusotoError;
use rusoto_kms::{
    GetPublicKeyError, GetPublicKeyRequest, Kms, KmsClient, SignError, SignRequest, SignResponse,
};
use tracing::{debug, instrument, trace};

mod utils;
use utils::{apply_eip155, verifying_key_to_address};

/// An ethers Signer that uses keys held in Amazon AWS KMS.
///
/// The AWS Signer passes signing requests to the cloud service. AWS KMS keys
/// are identified by a UUID, the `key_id`.
///
/// Because the public key is unknown, we retrieve it on instantiation of the
/// signer. This means that the new function is `async` and must be called
/// within some runtime.
///
/// ```compile_fail
/// use rusoto_core::Client;
/// use rusoto_kms::{Kms, KmsClient};
///
/// user ethers_signers::Signer;
///
/// let client = Client::new_with(
///     EnvironmentProvider::default(),
///     HttpClient::new().unwrap()
/// );
/// let kms_client = KmsClient::new_with_client(client, Region::UsWest1);
/// let key_id = "...";
/// let chain_id = 1;
///
/// let signer = AwsSigner::new(kms_client, key_id, chain_id).await?;
/// let sig = signer.sign_message(H256::zero()).await?;
/// ```
#[derive(Clone)]
pub struct AwsSigner {
    kms: KmsClient,
    chain_id: u64,
    key_id: String,
    pubkey: VerifyingKey,
    address: Address,
}

impl std::fmt::Debug for AwsSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AwsSigner")
            .field("key_id", &self.key_id)
            .field("chain_id", &self.chain_id)
            .field("pubkey", &hex::encode(self.pubkey.to_sec1_bytes()))
            .field("address", &self.address)
            .finish()
    }
}

impl std::fmt::Display for AwsSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AwsSigner {{ address: {}, chain_id: {}, key_id: {} }}",
            self.address, self.chain_id, self.key_id
        )
    }
}

/// Errors produced by the AwsSigner
#[derive(thiserror::Error, Debug)]
pub enum AwsSignerError {
    #[error("{0}")]
    SignError(#[from] RusotoError<SignError>),
    #[error("{0}")]
    GetPublicKeyError(#[from] RusotoError<GetPublicKeyError>),
    #[error("{0}")]
    K256(#[from] K256Error),
    #[error("{0}")]
    Spki(spki::Error),
    #[error("{0}")]
    Other(String),
    #[error(transparent)]
    /// Error when converting from a hex string
    HexError(#[from] hex::FromHexError),
    /// Error type from Eip712Error message
    #[error("error encoding eip712 struct: {0:?}")]
    Eip712Error(String),
}

impl From<String> for AwsSignerError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

impl From<spki::Error> for AwsSignerError {
    fn from(e: spki::Error) -> Self {
        Self::Spki(e)
    }
}

#[instrument(err, skip(kms, key_id), fields(key_id = %key_id.as_ref()))]
async fn request_get_pubkey<T>(
    kms: &KmsClient,
    key_id: T,
) -> Result<rusoto_kms::GetPublicKeyResponse, RusotoError<GetPublicKeyError>>
where
    T: AsRef<str>,
{
    debug!("Dispatching get_public_key");

    let req = GetPublicKeyRequest { grant_tokens: None, key_id: key_id.as_ref().to_owned() };
    trace!("{:?}", &req);
    let resp = kms.get_public_key(req).await;
    trace!("{:?}", &resp);
    resp
}

#[instrument(err, skip(kms, digest, key_id), fields(digest = %hex::encode(digest), key_id = %key_id.as_ref()))]
async fn request_sign_digest<T>(
    kms: &KmsClient,
    key_id: T,
    digest: [u8; 32],
) -> Result<SignResponse, RusotoError<SignError>>
where
    T: AsRef<str>,
{
    debug!("Dispatching sign");
    let req = SignRequest {
        grant_tokens: None,
        key_id: key_id.as_ref().to_owned(),
        message: digest.to_vec().into(),
        message_type: Some("DIGEST".to_owned()),
        signing_algorithm: "ECDSA_SHA_256".to_owned(),
    };
    trace!("{:?}", &req);
    let resp = kms.sign(req).await;
    trace!("{:?}", &resp);
    resp
}

impl AwsSigner {
    /// Instantiate a new signer from an existing `KmsClient` and Key ID.
    ///
    /// This function retrieves the public key from AWS and calculates the
    /// Etheruem address. It is therefore `async`.
    #[instrument(err, skip(kms, key_id, chain_id), fields(key_id = %key_id.as_ref()))]
    pub async fn new<T>(
        kms: KmsClient,
        key_id: T,
        chain_id: u64,
    ) -> Result<AwsSigner, AwsSignerError>
    where
        T: AsRef<str>,
    {
        let pubkey = request_get_pubkey(&kms, &key_id).await.map(utils::decode_pubkey)??;
        let address = verifying_key_to_address(&pubkey);

        debug!(
            "Instantiated AWS signer with pubkey 0x{} and address 0x{}",
            hex::encode(pubkey.to_sec1_bytes()),
            hex::encode(address)
        );

        Ok(Self { kms, chain_id, key_id: key_id.as_ref().to_owned(), pubkey, address })
    }

    /// Fetch the pubkey associated with a key id
    pub async fn get_pubkey_for_key<T>(&self, key_id: T) -> Result<VerifyingKey, AwsSignerError>
    where
        T: AsRef<str>,
    {
        request_get_pubkey(&self.kms, key_id).await.map(utils::decode_pubkey)?
    }

    /// Fetch the pubkey associated with this signer's key ID
    pub async fn get_pubkey(&self) -> Result<VerifyingKey, AwsSignerError> {
        self.get_pubkey_for_key(&self.key_id).await
    }

    /// Sign a digest with the key associated with a key id
    pub async fn sign_digest_with_key<T>(
        &self,
        key_id: T,
        digest: [u8; 32],
    ) -> Result<KSig, AwsSignerError>
    where
        T: AsRef<str>,
    {
        request_sign_digest(&self.kms, key_id, digest).await.map(utils::decode_signature)?
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
        apply_eip155(&mut sig, chain_id);
        Ok(sig)
    }
}

#[async_trait::async_trait]
impl super::Signer for AwsSigner {
    type Error = AwsSignerError;

    #[instrument(err, skip(message))]
    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<EthSig, Self::Error> {
        let message = message.as_ref();
        let message_hash = hash_message(message);
        trace!("{:?}", message_hash);
        trace!("{:?}", message);

        self.sign_digest_with_eip155(message_hash, self.chain_id).await
    }

    #[instrument(err)]
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

    /// Returns the signer's chain id
    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Sets the signer's chain id
    fn with_chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.chain_id = chain_id.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Signer;
    use rusoto_core::{
        credential::{EnvironmentProvider, StaticProvider},
        Client, HttpClient, Region,
    };

    #[allow(dead_code)]
    fn static_client() -> KmsClient {
        let access_key = "".to_owned();
        let secret_access_key = "".to_owned();

        let client = Client::new_with(
            StaticProvider::new(access_key, secret_access_key, None, None),
            HttpClient::new().unwrap(),
        );
        KmsClient::new_with_client(client, Region::UsWest1)
    }

    #[allow(dead_code)]
    fn env_client() -> KmsClient {
        let client = Client::new_with(EnvironmentProvider::default(), HttpClient::new().unwrap());
        KmsClient::new_with_client(client, Region::UsWest1)
    }

    #[tokio::test]
    async fn it_signs_messages() {
        let chain_id = 1;
        let key_id = match std::env::var("AWS_KEY_ID") {
            Ok(id) => id,
            _ => return,
        };
        let client = env_client();
        let signer = AwsSigner::new(client, key_id, chain_id).await.unwrap();

        let message = vec![0, 1, 2, 3];

        let sig = signer.sign_message(&message).await.unwrap();
        sig.verify(message, signer.address).expect("valid sig");
    }
}
