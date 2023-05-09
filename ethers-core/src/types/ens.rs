use crate::types::Address;
use rlp::{Decodable, Encodable, RlpStream};
use serde::{ser::Error as SerializationError, Deserialize, Deserializer, Serialize, Serializer};
use std::{cmp::Ordering, str::FromStr};

/// ENS name or Ethereum Address. Not RLP encoded/serialized if it's a name.
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
        if let Self::Address(inner) = self {
            inner.rlp_append(s);
        }
    }
}

impl Decodable for NameOrAddress {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        // An address (H160) is 20 bytes, so let's only accept 20 byte rlp string encodings.
        if !rlp.is_data() {
            return Err(rlp::DecoderError::RlpExpectedToBeData)
        }

        // the data needs to be 20 bytes long
        match rlp.size().cmp(&20usize) {
            Ordering::Less => Err(rlp::DecoderError::RlpIsTooShort),
            Ordering::Greater => Err(rlp::DecoderError::RlpIsTooBig),
            Ordering::Equal => {
                let rlp_data = rlp.data()?;
                Ok(Self::Address(Address::from_slice(rlp_data)))
            }
        }
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
            Self::Address(addr) => addr.serialize(serializer),
            Self::Name(name) => Err(SerializationError::custom(format!(
                "cannot serialize ENS name {name}, must be address"
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
        Ok(Self::Address(inner))
    }
}

impl From<&str> for NameOrAddress {
    fn from(s: &str) -> Self {
        Self::from_str(s).unwrap()
    }
}

impl From<String> for NameOrAddress {
    fn from(s: String) -> Self {
        Self::Name(s)
    }
}

impl From<&String> for NameOrAddress {
    fn from(s: &String) -> Self {
        Self::Name(s.clone())
    }
}

impl From<Address> for NameOrAddress {
    fn from(s: Address) -> Self {
        Self::Address(s)
    }
}

impl FromStr for NameOrAddress {
    type Err = <Address as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("0x") {
            s.parse().map(Self::Address)
        } else {
            Ok(Self::Name(s.to_string()))
        }
    }
}

impl NameOrAddress {
    /// Maps Address(a) to Some(a) and Name to None.
    pub fn as_address(&self) -> Option<&Address> {
        match self {
            Self::Address(a) => Some(a),
            Self::Name(_) => None,
        }
    }

    /// Maps Name(n) to Some(n) and Address to None.
    pub fn as_name(&self) -> Option<&str> {
        match self {
            Self::Address(_) => None,
            Self::Name(n) => Some(n),
        }
    }
}

#[cfg(test)]
mod tests {
    use rlp::Rlp;

    use super::*;

    #[test]
    fn rlp_name_not_serialized() {
        let name = NameOrAddress::Name("ens.eth".to_string());

        let mut rlp = RlpStream::new();
        name.rlp_append(&mut rlp);
        assert!(rlp.is_empty());

        let mut rlp = RlpStream::new();
        name.rlp_append(&mut rlp);
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
        union.rlp_append(&mut rlp);
        assert_eq!(rlp.as_raw(), expected.as_raw());
    }

    #[test]
    fn rlp_address_deserialized() {
        let addr = "3dd6f334b732d23b51dfbee2070b40bbd1a97a8f".parse().unwrap();
        let expected = NameOrAddress::Address(addr);

        let mut rlp = RlpStream::new();
        rlp.append(&addr);
        let rlp_bytes = &rlp.out().freeze()[..];
        let data = Rlp::new(rlp_bytes);
        let name = NameOrAddress::decode(&data).unwrap();

        assert_eq!(name, expected);
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
