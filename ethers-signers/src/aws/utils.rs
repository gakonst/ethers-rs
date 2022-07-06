//! These utils are NOT meant for general usage. They are ONLY meant for use
//! within this module. They DO NOT perform basic safety checks and may panic
//! if used incorrectly.

use std::convert::TryFrom;

use ethers_core::{
    k256::{
        ecdsa::{
            recoverable::{Id, Signature as RSig},
            Signature as KSig, VerifyingKey,
        },
        elliptic_curve::sec1::ToEncodedPoint,
        FieldBytes,
    },
    types::{Address, Signature as EthSig, U256},
    utils::keccak256,
};
use rusoto_kms::{GetPublicKeyResponse, SignResponse};

use crate::aws::AwsSignerError;

/// Converts a recoverable signature to an ethers signature
pub(super) fn rsig_to_ethsig(sig: &RSig) -> EthSig {
    let v: u8 = sig.recovery_id().into();
    let v = (v + 27) as u64;
    let r_bytes: FieldBytes = sig.r().into();
    let s_bytes: FieldBytes = sig.s().into();
    let r = U256::from_big_endian(r_bytes.as_slice());
    let s = U256::from_big_endian(s_bytes.as_slice());
    EthSig { r, s, v }
}

/// Makes a trial recovery to check whether an RSig corresponds to a known
/// `VerifyingKey`
fn check_candidate(sig: &RSig, digest: [u8; 32], vk: &VerifyingKey) -> bool {
    if let Ok(key) = sig.recover_verifying_key_from_digest_bytes(digest.as_ref().into()) {
        key == *vk
    } else {
        false
    }
}

/// Recover an rsig from a signature under a known key by trial/error
pub(super) fn rsig_from_digest_bytes_trial_recovery(
    sig: &KSig,
    digest: [u8; 32],
    vk: &VerifyingKey,
) -> RSig {
    let sig_0 = RSig::new(sig, Id::new(0).unwrap()).unwrap();
    let sig_1 = RSig::new(sig, Id::new(1).unwrap()).unwrap();

    if check_candidate(&sig_0, digest, vk) {
        sig_0
    } else if check_candidate(&sig_1, digest, vk) {
        sig_1
    } else {
        panic!("bad sig");
    }
}

/// Modify the v value of a signature to conform to eip155
pub(super) fn apply_eip155(sig: &mut EthSig, chain_id: u64) {
    let v = (chain_id * 2 + 35) + ((sig.v - 1) % 2);
    sig.v = v;
}

/// Convert a verifying key to an ethereum address
pub(super) fn verifying_key_to_address(key: &VerifyingKey) -> Address {
    // false for uncompressed
    let uncompressed_pub_key = key.to_encoded_point(false);
    let public_key = uncompressed_pub_key.to_bytes();
    debug_assert_eq!(public_key[0], 0x04);
    let hash = keccak256(&public_key[1..]);
    Address::from_slice(&hash[12..])
}

/// Decode an AWS KMS Pubkey response
pub(super) fn decode_pubkey(resp: GetPublicKeyResponse) -> Result<VerifyingKey, AwsSignerError> {
    let raw = resp
        .public_key
        .ok_or_else(|| AwsSignerError::from("Pubkey not found in response".to_owned()))?;

    let spk = spki::SubjectPublicKeyInfo::try_from(raw.as_ref())?;
    let key = VerifyingKey::from_sec1_bytes(spk.subject_public_key)?;

    Ok(key)
}

/// Decode an AWS KMS Signature response
pub(super) fn decode_signature(resp: SignResponse) -> Result<KSig, AwsSignerError> {
    let raw = resp
        .signature
        .ok_or_else(|| AwsSignerError::from("Signature not found in response".to_owned()))?;

    let sig = KSig::from_der(&raw)?;
    Ok(sig.normalize_s().unwrap_or(sig))
}
