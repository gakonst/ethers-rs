use crate::types::{Address, Bytes};

#[derive(Clone, Debug, PartialEq, Eq)]
/// A type that can either be an `Address` or `Bytes`.
pub enum AddressOrBytes {
    /// An address type
    Address(Address),
    /// A bytes type
    Bytes(Bytes),
}

impl From<Address> for AddressOrBytes {
    fn from(s: Address) -> Self {
        Self::Address(s)
    }
}

impl From<Bytes> for AddressOrBytes {
    fn from(s: Bytes) -> Self {
        Self::Bytes(s)
    }
}
