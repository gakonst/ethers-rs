pub type Selector = [u8; 4];

// Re-export common ethereum datatypes with more specific names

/// A transaction Hash
pub use ethabi::ethereum_types::H256 as TxHash;

pub use ethabi::ethereum_types::{Address, Bloom, H160, H256, U128, U256, U64};

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

mod i256;
pub use i256::I256;

mod bytes;
pub use self::bytes::Bytes;

mod block;
pub use block::{Block, BlockId, BlockNumber};

#[cfg(feature = "celo")]
pub use block::Randomness;

mod log;
pub use log::{Filter, FilterBlockOption, Log, ValueOrArray};

mod ens;
pub use ens::NameOrAddress;

mod signature;
pub use signature::*;

mod txpool;
pub use txpool::*;

mod trace;
pub use trace::*;
