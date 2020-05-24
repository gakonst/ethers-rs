//! Various Ethereum Related Datatypes

// Re-export common ethereum datatypes with more specific names
pub use ethereum_types::H256 as TxHash;
pub use ethereum_types::{Address, H256, U256, U64};

mod transaction;
// TODO: Figure out some more intuitive way instead of having 3 similarly named structs
// with the same fields
pub use transaction::{Transaction, TransactionRequest, UnsignedTransaction};

mod keys;
pub use keys::{PrivateKey, PublicKey};

mod signature;
pub use signature::Signature;

mod bytes;
pub use bytes::Bytes;

mod block;
pub use block::BlockNumber;
