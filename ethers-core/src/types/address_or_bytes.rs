use crate::types::{Address, Bytes};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AddressOrBytes {
    Address(Address),
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
