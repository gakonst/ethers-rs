//! Ethereum data types.

pub type Selector = [u8; 4];

// Re-export common ethereum datatypes with more specific names

/// A transaction Hash
pub use ethabi::ethereum_types::H256 as TxHash;

pub use ethabi::ethereum_types::{
    Address, BigEndianHash, Bloom, H128, H160, H256, H32, H512, H64, U128, U256, U512, U64,
};

pub mod transaction;
pub use transaction::{
    eip1559::Eip1559TransactionRequest,
    eip2930::Eip2930TransactionRequest,
    request::TransactionRequest,
    response::{Transaction, TransactionReceipt},
};

mod address_or_bytes;
pub use address_or_bytes::AddressOrBytes;

mod path_or_string;
pub use path_or_string::PathOrString;

mod u256;
pub use u256::*;

mod uint8;
pub use uint8::*;

mod i256;
pub use i256::{ParseI256Error, Sign, I256};

mod bytes;
pub use self::bytes::{deserialize_bytes, serialize_bytes, Bytes, ParseBytesError};

mod block;
pub use block::{Block, BlockId, BlockNumber, TimeError};

#[cfg(feature = "celo")]
pub use block::Randomness;

mod log;
pub use log::Log;

mod filter;
pub use filter::*;

mod ens;
pub use ens::NameOrAddress;

mod signature;
pub use signature::*;

mod txpool;
pub use txpool::*;

mod trace;
pub use trace::*;

mod chain;
pub use chain::*;

mod proof;

pub use proof::*;

mod fee;
pub use fee::*;

mod other;
pub use other::OtherFields;

pub mod serde_helpers;

mod syncing;
pub use syncing::{SyncProgress, SyncingStatus};

mod opcode;
pub use opcode::Opcode;

mod withdrawal;
pub use withdrawal::Withdrawal;
