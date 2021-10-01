/// The Eip712 trait provides helper methods for computing
/// the typed data hash used in `eth_signTypedData`.
///
/// The ethers-rs `derive_eip712` crate provides a derive macro to
/// implement the trait for a given struct. See documentation
/// for `derive_eip712` for more information and example usage.
///
/// For those who wish to manually implement this trait, see:
/// https://eips.ethereum.org/EIPS/eip-712
///
/// Any rust struct implementing Eip712 must also have a corresponding
/// struct in the verifying ethereum contract that matches its signature.
///
/// NOTE: Due to limitations of the derive macro not supporting return types of
/// [u8; 32] or Vec<u8>, all methods should return the hex encoded values of the keccak256
/// byte array.
pub trait Eip712 {
    /// User defined error type;
    type Error: std::error::Error + Send + Sync + std::fmt::Debug;

    /// The eip712 domain is the same for all Eip712 implementations,
    /// This method does not need to be manually implemented, but may be overridden
    /// if needed.
    fn eip712_domain_type_hash() -> String {
        hex::encode(crate::utils::keccak256(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
        ))
    }

    /// The domain separator depends on the contract and unique domain
    /// for which the user is targeting. In the derive macro, these attributes
    /// are passed in as arguments to the macro. When manually deriving, the user
    /// will need to know the name of the domain, version of the contract, chain ID of
    /// where the contract lives and the address of the verifying contract.
    fn domain_separator() -> String;

    /// This method is used for calculating the hash of the type signature of the
    /// struct. The field types of the struct must map to primitive
    /// ethereum types or custom types defined in the contract.
    fn type_hash() -> String;

    /// When using the derive macro, this is the primary method used for computing the final
    /// EIP-712 encoded payload. This method relies on the aforementioned methods for computing
    /// the final encoded payload.
    fn encode_eip712(&self) -> Result<String, Self::Error>;
}
