//! Various Ethereum Related Datatypes

// Re-export common ethereum datatypes with more specific names
pub use ethereum_types::H256 as TxHash;
pub use ethereum_types::{Address, H256, U256, U64};

mod transaction;
pub use transaction::{Transaction, TransactionRequest};

mod keys;
pub use keys::{PrivateKey, PublicKey, TxError};

mod signature;
pub use signature::Signature;

mod bytes;
pub use bytes::Bytes;

mod block_number;
pub use block_number::BlockNumber;
