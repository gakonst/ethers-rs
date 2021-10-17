use crate::AbiError;
use ethers_core::types::Bytes;

/// Trait for ABI encoding
pub trait AbiEncode {
    /// ABI encode the type
    fn encode(self) -> Result<Bytes, AbiError>;
}

/// Trait for ABI decoding
pub trait AbiDecode: Sized {
    /// Decodes the ABI encoded data
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError>;
}
