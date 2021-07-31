//! Helpers for interacting with the Ethereum Ledger App
//! [Official Docs](https://github.com/LedgerHQ/app-ethereum/blob/master/doc/ethapp.asc)
use std::fmt;
use thiserror::Error;

#[derive(Clone, Debug)]
/// Ledger wallet type
pub enum DerivationType {
    /// Ledger Live-generated HD path
    LedgerLive(usize),
    /// Legacy generated HD Path
    Legacy(usize),
    /// Any other path
    Other(String),
}

impl fmt::Display for DerivationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                DerivationType::Legacy(index) => format!("m/44'/60'/0'/{}", index),
                DerivationType::LedgerLive(index) => format!("m/44'/60'/{}'/0/0", index),
                DerivationType::Other(inner) => inner.to_owned(),
            }
        )
    }
}

#[derive(Error, Debug)]
/// Error when using the Ledger transport
pub enum LedgerError {
    /// Underlying ledger transport error
    #[error(transparent)]
    LedgerError(#[from] coins_ledger::errors::LedgerError),
    /// Device response was unexpectedly none
    #[error("Received unexpected response from device. Expected data in response, found none.")]
    UnexpectedNullResponse,

    #[error(transparent)]
    /// Error when converting from a hex string
    HexError(#[from] hex::FromHexError),
}

pub const P1_FIRST: u8 = 0x00;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum INS {
    GET_PUBLIC_KEY = 0x02,
    SIGN = 0x04,
    GET_APP_CONFIGURATION = 0x06,
    SIGN_PERSONAL_MESSAGE = 0x08,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum P1 {
    NON_CONFIRM = 0x00,
    MORE = 0x80,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum P2 {
    NO_CHAINCODE = 0x00,
}
