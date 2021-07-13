//! Mod of types for ethereum logs
use ethers_core::abi::Error;
use ethers_core::abi::RawLog;

/// A trait for types (events) that can be decoded from a `RawLog`
pub trait EthLogDecode: Send + Sync {
    /// decode from a `RawLog`
    fn decode_log(log: &RawLog) -> Result<Self, Error>
    where
        Self: Sized;
}

/// Decodes a series of logs into a vector
pub fn decode_logs<T: EthLogDecode>(logs: &[RawLog]) -> Result<Vec<T>, Error> {
    logs.iter().map(T::decode_log).collect()
}
