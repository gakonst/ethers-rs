use crate::Address;
use rlp::{Encodable, RlpStream};
use serde::{ser::Error as SerializationError, Deserialize, Deserializer, Serialize, Serializer};

/// ENS name or Ethereum Address. Not RLP encoded/serialized if it's a name
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NameOrAddress {
    Name(String),
    Address(Address),
}

// Only RLP encode the Address variant since it doesn't make sense to ever RLP encode
// an ENS name
impl Encodable for &NameOrAddress {
    fn rlp_append(&self, s: &mut RlpStream) {
        if let NameOrAddress::Address(inner) = self {
            inner.rlp_append(s);
        }
    }
}

impl From<&str> for NameOrAddress {
    fn from(s: &str) -> Self {
        NameOrAddress::Name(s.to_owned())
    }
}

impl From<Address> for NameOrAddress {
    fn from(s: Address) -> Self {
        NameOrAddress::Address(s)
    }
}

// Only serialize the Address variant since it doesn't make sense to ever serialize
// an ENS name
impl Serialize for NameOrAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            NameOrAddress::Address(addr) => addr.serialize(serializer),
            NameOrAddress::Name(name) => Err(SerializationError::custom(format!(
                "cannot serialize ENS name {}, must be address",
                name
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for NameOrAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = Address::deserialize(deserializer)?;

        Ok(NameOrAddress::Address(inner))
    }
}
