use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};
use thiserror::Error;

use std::{
    clone::Clone,
    fmt::{Debug, Display, Formatter, LowerHex, Result as FmtResult},
    str::FromStr,
};

/// Wrapper type around Bytes to deserialize/serialize "0x" prefixed ethereum hex strings
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub struct Bytes(
    #[serde(serialize_with = "serialize_bytes", deserialize_with = "deserialize_bytes")]
    pub  bytes::Bytes,
);

fn bytes_to_hex(b: &Bytes) -> String {
    hex::encode(b.0.as_ref())
}

impl Display for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "0x{}", bytes_to_hex(self))
    }
}

impl LowerHex for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "0x{}", bytes_to_hex(self))
    }
}

impl Bytes {
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<bytes::Bytes> for Bytes {
    fn from(src: bytes::Bytes) -> Self {
        Self(src)
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(src: Vec<u8>) -> Self {
        Self(src.into())
    }
}

impl<const N: usize> From<[u8; N]> for Bytes {
    fn from(src: [u8; N]) -> Self {
        src.to_vec().into()
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for Bytes {
    fn from(src: &'a [u8; N]) -> Self {
        src.to_vec().into()
    }
}

#[derive(Debug, Clone, Error)]
#[error("Failed to parse bytes: {0}")]
pub struct ParseBytesError(String);

impl FromStr for Bytes {
    type Err = ParseBytesError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() >= 2 && &value[0..2] == "0x" {
            let bytes: Vec<u8> = hex::decode(&value[2..])
                .map_err(|e| ParseBytesError(format!("Invalid hex: {}", e)))?;
            Ok(bytes.into())
        } else {
            Err(ParseBytesError("Doesn't start with 0x".to_string()))
        }
    }
}

pub fn serialize_bytes<S, T>(x: T, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    s.serialize_str(&format!("0x{}", hex::encode(x.as_ref())))
}

pub fn deserialize_bytes<'de, D>(d: D) -> Result<bytes::Bytes, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(d)?;
    if value.len() >= 2 && &value[0..2] == "0x" {
        let bytes: Vec<u8> =
            hex::decode(&value[2..]).map_err(|e| Error::custom(format!("Invalid hex: {}", e)))?;
        Ok(bytes.into())
    } else {
        Err(Error::invalid_value(Unexpected::Str(&value), &"0x prefix"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_formatting() {
        let b = Bytes::from(vec![1, 35, 69, 103, 137, 171, 205, 239]);
        let expected = String::from("0x0123456789abcdef");
        assert_eq!(format!("{:x}", b), expected);
        assert_eq!(format!("{}", b), expected);
    }

    #[test]
    fn test_from_str() {
        let b = Bytes::from_str("0x1213");
        assert!(b.is_ok());
        let b = b.unwrap();
        assert_eq!(b.as_ref(), hex::decode("1213").unwrap());

        let b = Bytes::from_str("1213");
        assert!(b.is_err());
        assert_eq!(b.err().unwrap().0, "Doesn't start with 0x");
    }
}
