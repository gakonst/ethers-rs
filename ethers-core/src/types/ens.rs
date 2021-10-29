use crate::types::Address;
use rlp::{Encodable, RlpStream};
use serde::{ser::Error as SerializationError, Deserialize, Deserializer, Serialize, Serializer};

/// ENS name or Ethereum Address. Not RLP encoded/serialized if it's a name
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NameOrAddress {
    /// An ENS Name (format does not get checked)
    Name(String),
    /// An Ethereum Address
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

impl Encodable for NameOrAddress {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rlp_name_not_serialized() {
        let name = NameOrAddress::Name("ens.eth".to_string());

        let mut rlp = RlpStream::new();
        name.rlp_append(&mut rlp);
        assert!(rlp.is_empty());

        let mut rlp = RlpStream::new();
        (&name).rlp_append(&mut rlp);
        assert!(rlp.is_empty());
    }

    #[test]
    fn rlp_address_serialized() {
        let addr = "f02c1c8e6114b1dbe8937a39260b5b0a374432bb".parse().unwrap();
        let union = NameOrAddress::Address(addr);

        let mut expected = RlpStream::new();
        addr.rlp_append(&mut expected);

        let mut rlp = RlpStream::new();
        union.rlp_append(&mut rlp);
        assert_eq!(rlp.as_raw(), expected.as_raw());

        let mut rlp = RlpStream::new();
        (&union).rlp_append(&mut rlp);
        assert_eq!(rlp.as_raw(), expected.as_raw());
    }

    #[test]
    fn serde_name_not_serialized() {
        let name = NameOrAddress::Name("ens.eth".to_string());
        bincode::serialize(&name).unwrap_err();
    }

    #[test]
    fn serde_address_serialized() {
        let addr = "f02c1c8e6114b1dbe8937a39260b5b0a374432bb".parse().unwrap();
        let union = NameOrAddress::Address(addr);

        assert_eq!(bincode::serialize(&addr).unwrap(), bincode::serialize(&union).unwrap(),);
    }
}
