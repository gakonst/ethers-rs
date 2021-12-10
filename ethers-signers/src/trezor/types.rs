#![allow(clippy::upper_case_acronyms)]
//! Helpers for interacting with the Ethereum Trezor App
//! [Official Docs](https://github.com/TrezorHQ/app-ethereum/blob/master/doc/ethapp.asc)
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug)]
/// Trezor wallet type
pub enum DerivationType {
    /// Trezor Live-generated HD path
    TrezorLive(usize),
    /// Any other path. Attention! Trezor by default forbids custom derivation paths
    /// Run trezorctl set safety-checks prompt, to allow it
    Other(String),
}

impl fmt::Display for DerivationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                DerivationType::TrezorLive(index) => format!("m/44'/60'/{}'/0/0", index),
                DerivationType::Other(inner) => inner.to_owned(),
            }
        )
    }
}

#[derive(Error, Debug)]
/// Error when using the Trezor transport
pub enum TrezorError {
    /// Underlying Trezor transport error
    #[error(transparent)]
    TrezorError(#[from] trezor_client::error::Error),
    #[error(transparent)]
    /// Error when converting from a hex string
    HexError(#[from] hex::FromHexError),
    #[error(transparent)]
    /// Error when converting a semver requirement
    SemVerError(#[from] semver::Error),
    /// Error when signing EIP712 struct with not compatible Trezor ETH app
    #[error("Trezor ethereum app requires at least version: {0:?}")]
    UnsupportedAppVersion(String),
}
